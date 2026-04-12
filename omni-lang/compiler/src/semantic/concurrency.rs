//! Phase 9: Concurrency Runtime & Tensor Acceleration
//!
//! Implements Phase 9 requirements:
//! - Work-stealing structured concurrency executor
//! - Actor model with supervision trees
//! - Tensor/SIMD module implementation
//! - Replay debugging infrastructure

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct WorkStealingScheduler {
    pub queues: Vec<Arc<Mutex<VecDeque<Task>>>>,
    pub thread_count: usize,
    pub active_workers: Arc<AtomicUsize>,
    pub shutdown: Arc<AtomicBool>,
    pub wakeup: Arc<(Mutex<bool>, Condvar)>,
    pub actor_system: Option<Arc<Mutex<ActorSystem>>>,
}

pub struct Task {
    pub id: usize,
    pub name: String,
    pub payload: TaskPayload,
    pub state: TaskState,
}

pub enum TaskPayload {
    Fn(Box<dyn FnOnce() + Send + 'static>),
    Actor(ActorMessage),
}

pub enum TaskState {
    Pending,
    Running,
    Completed,
    Failed(String),
}

impl WorkStealingScheduler {
    pub fn new(thread_count: usize) -> Self {
        let queues = (0..thread_count)
            .map(|_| Arc::new(Mutex::new(VecDeque::new())))
            .collect();
        Self {
            queues,
            thread_count,
            active_workers: Arc::new(AtomicUsize::new(0)),
            shutdown: Arc::new(AtomicBool::new(false)),
            wakeup: Arc::new((Mutex::new(false), Condvar::new())),
            actor_system: None,
        }
    }

    pub fn attach_actor_system(&mut self, actor_system: Arc<Mutex<ActorSystem>>) {
        self.actor_system = Some(actor_system);
    }

    pub fn spawn(&self, task: Task) {
        let idx = task.id % self.thread_count;
        if let Some(queue) = self.queues.get(idx) {
            let mut queue = queue.lock().unwrap();
            queue.push_back(task);
        }
        let (lock, cvar) = &*self.wakeup;
        let mut ready = lock.lock().unwrap();
        *ready = true;
        cvar.notify_one();
    }

    fn pop_task(&self, worker_idx: usize) -> Option<Task> {
        if let Some(queue) = self.queues.get(worker_idx) {
            let mut queue = queue.lock().unwrap();
            if let Some(task) = queue.pop_front() {
                return Some(task);
            }
        }

        for idx in 0..self.thread_count {
            if idx == worker_idx {
                continue;
            }
            if let Some(queue) = self.queues.get(idx) {
                let mut queue = queue.lock().unwrap();
                if let Some(task) = queue.pop_back() {
                    return Some(task);
                }
            }
        }
        None
    }

    fn any_task_available(&self) -> bool {
        self.queues
            .iter()
            .any(|queue| !queue.lock().unwrap().is_empty())
    }

    pub fn run(self: Arc<Self>) -> Vec<JoinHandle<()>> {
        let mut handles = Vec::with_capacity(self.thread_count);
        for worker_idx in 0..self.thread_count {
            let scheduler = self.clone();
            let handle = thread::spawn(move || {
                scheduler.worker_loop(worker_idx);
            });
            handles.push(handle);
        }
        handles
    }

    fn worker_loop(&self, worker_idx: usize) {
        while !self.is_shutdown() {
            if let Some(mut task) = self.pop_task(worker_idx) {
                self.active_workers.fetch_add(1, Ordering::SeqCst);
                task.state = TaskState::Running;
                self.execute_task(task);
                self.active_workers.fetch_sub(1, Ordering::SeqCst);
            } else {
                let (lock, cvar) = &*self.wakeup;
                let mut ready = lock.lock().unwrap();
                while !self.is_shutdown() && !self.any_task_available() {
                    ready = cvar.wait(ready).unwrap();
                }
            }
        }
    }

    fn execute_task(&self, task: Task) {
        match task.payload {
            TaskPayload::Fn(handler) => {
                handler();
            }
            TaskPayload::Actor(message) => {
                if let Some(actor_system) = &self.actor_system {
                    let mut system = actor_system.lock().unwrap();
                    let _ = system.dispatch_message(&task.name, message);
                }
            }
        }
    }

    pub fn wait_for_idle(&self) {
        while self.active_workers.load(Ordering::SeqCst) > 0 || self.any_task_available() {
            thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let (_, cvar) = &*self.wakeup;
        cvar.notify_all();
    }

    pub fn create_scope(self: Arc<Self>) -> SpawnScope {
        SpawnScope {
            scheduler: self,
            pending: Arc::new(AtomicUsize::new(0)),
        }
    }
}

pub struct SpawnScope {
    pub scheduler: Arc<WorkStealingScheduler>,
    pub pending: Arc<AtomicUsize>,
}

impl SpawnScope {
    pub fn spawn<F>(&self, name: &str, func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pending.fetch_add(1, Ordering::SeqCst);
        let pending = self.pending.clone();
        let name = name.to_string();
        let task = Task {
            id: pending.load(Ordering::SeqCst),
            name,
            payload: TaskPayload::Fn(Box::new(move || {
                func();
                pending.fetch_sub(1, Ordering::SeqCst);
            })),
            state: TaskState::Pending,
        };
        self.scheduler.spawn(task);
    }

    pub fn join(&self) {
        while self.pending.load(Ordering::SeqCst) > 0 {
            thread::sleep(Duration::from_millis(1));
        }
    }
}

pub struct GlobalSpawnCap(pub usize);

impl GlobalSpawnCap {
    pub fn new(limit: usize) -> Self {
        Self(limit)
    }

    pub fn allow(&self) -> bool {
        self.0 > 0
    }
}

pub struct ActorSystem {
    pub actors: HashMap<String, Actor>,
    pub supervision_tree: HashMap<String, SupervisionRule>,
    pub mailbox: HashMap<String, VecDeque<ActorMessage>>,
}

pub struct Actor {
    pub id: String,
    pub handler: ActorHandler,
    pub state: ActorState,
}

pub enum ActorHandler {
    Stateless(fn(ActorMessage) -> ActorResponse),
    Stateful(Box<dyn ActorStatefulHandler>),
}

pub trait ActorStatefulHandler: Send + Sync {
    fn handle(&mut self, msg: ActorMessage) -> ActorResponse;
}

pub enum ActorState {
    Starting,
    Running,
    Restarting,
    Stopped,
}

pub struct ActorMessage {
    pub sender: Option<String>,
    pub payload: MessagePayload,
}

pub enum MessagePayload {
    Stop,
    Restart,
    Ping,
    Pong,
    Custom(Vec<u8>),
}

pub enum ActorResponse {
    Ok,
    Pong,
    Custom(Vec<u8>),
    Error(String),
}

pub struct SupervisionRule {
    pub actor_id: String,
    pub strategy: SupervisionStrategy,
    pub max_restarts: usize,
    pub restart_count: usize,
}

pub enum SupervisionStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}

impl ActorSystem {
    pub fn new() -> Self {
        Self {
            actors: HashMap::new(),
            supervision_tree: HashMap::new(),
            mailbox: HashMap::new(),
        }
    }

    pub fn spawn_actor(
        &mut self,
        id: String,
        handler: ActorHandler,
        strategy: SupervisionStrategy,
    ) -> Result<(), String> {
        if self.actors.contains_key(&id) {
            return Err(format!("Actor {} already exists", id));
        }

        self.actors.insert(
            id.clone(),
            Actor {
                id: id.clone(),
                handler,
                state: ActorState::Starting,
            },
        );

        self.supervision_tree.insert(
            id.clone(),
            SupervisionRule {
                actor_id: id.clone(),
                strategy,
                max_restarts: 3,
                restart_count: 0,
            },
        );

        self.mailbox.insert(id, VecDeque::new());
        Ok(())
    }

    pub fn send(&mut self, target: &str, msg: ActorMessage) -> Result<(), String> {
        if let Some(mailbox) = self.mailbox.get_mut(target) {
            mailbox.push_back(msg);
            Ok(())
        } else {
            Err(format!("Target actor {} not found", target))
        }
    }

    pub fn process_mailbox(&mut self, actor_id: &str) -> Result<ActorResponse, String> {
        let mailbox = self.mailbox.get_mut(actor_id).ok_or("Actor not found")?;

        if let Some(msg) = mailbox.pop_front() {
            self.dispatch_message(actor_id, msg)
        } else {
            Err("No messages".to_string())
        }
    }

    pub fn dispatch_message(&mut self, actor_id: &str, msg: ActorMessage) -> Result<ActorResponse, String> {
        let actor = self.actors.get_mut(actor_id).ok_or("Actor not found")?;
        actor.state = ActorState::Running;

        let result = match &mut actor.handler {
            ActorHandler::Stateless(handler) => handler(msg),
            ActorHandler::Stateful(handler) => handler.handle(msg),
        };

        match &result {
            ActorResponse::Error(error) => {
                actor.state = ActorState::Restarting;
                self.apply_supervision(actor_id, error);
            }
            _ => {
                actor.state = ActorState::Running;
            }
        }

        Ok(result)
    }

    pub fn process_all_mailboxes(&mut self) -> Vec<Result<ActorResponse, String>> {
        let actor_ids: Vec<String> = self.mailbox.keys().cloned().collect();
        actor_ids
            .into_iter()
            .map(|id| self.process_mailbox(&id))
            .collect()
    }

    pub fn apply_supervision(&mut self, failed_actor: &str, _error: &str) {
        if let Some(rule) = self.supervision_tree.get_mut(failed_actor) {
            if rule.restart_count < rule.max_restarts {
                rule.restart_count += 1;

                if let Some(actor) = self.actors.get_mut(failed_actor) {
                    actor.state = ActorState::Restarting;
                }

                match rule.strategy {
                    SupervisionStrategy::OneForOne => {
                        if let Some(actor) = self.actors.get_mut(failed_actor) {
                            actor.state = ActorState::Starting;
                        }
                    }
                    SupervisionStrategy::OneForAll => {
                        for child_id in self.supervision_tree.keys() {
                            if let Some(actor) = self.actors.get_mut(child_id) {
                                actor.state = ActorState::Starting;
                            }
                        }
                    }
                    SupervisionStrategy::RestForOne => {
                        let mut found = false;
                        for child_id in self.supervision_tree.keys() {
                            if found {
                                if let Some(actor) = self.actors.get_mut(child_id) {
                                    actor.state = ActorState::Starting;
                                }
                            }
                            if child_id == failed_actor {
                                found = true;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TensorModule {
    pub simd_enabled: bool,
    pub simd_width: usize,
}

pub struct Tensor {
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
    pub strides: Vec<usize>,
}

impl Tensor {
    pub fn new(shape: Vec<usize>) -> Self {
        let size: usize = shape.iter().product();
        let mut strides = vec![1usize; shape.len()];
        for i in (0..shape.len() - 1).rev() {
            strides[i] = strides[i + 1] * shape[i + 1];
        }
        Self {
            shape,
            data: vec![0.0; size],
            strides,
        }
    }

    pub fn zeros(shape: Vec<usize>) -> Self {
        Self::new(shape)
    }

    pub fn ones(shape: Vec<usize>) -> Self {
        let mut t = Self::new(shape);
        for v in t.data.iter_mut() {
            *v = 1.0;
        }
        t
    }

    pub fn fill(&mut self, value: f32) {
        for v in self.data.iter_mut() {
            *v = value;
        }
    }

    pub fn add(&self, other: &Tensor) -> Result<Tensor, String> {
        if self.shape != other.shape {
            return Err("Shape mismatch".to_string());
        }

        let mut result = Tensor::new(self.shape.clone());
        for (i, v) in self.data.iter().enumerate() {
            result.data[i] = v + other.data[i];
        }
        Ok(result)
    }

    pub fn mul(&self, other: &Tensor) -> Result<Tensor, String> {
        if self.shape != other.shape {
            return Err("Shape mismatch".to_string());
        }

        let mut result = Tensor::new(self.shape.clone());
        for (i, v) in self.data.iter().enumerate() {
            result.data[i] = v * other.data[i];
        }
        Ok(result)
    }

    pub fn matmul(&self, other: &Tensor) -> Result<Tensor, String> {
        if self.shape.len() != 2 || other.shape.len() != 2 {
            return Err("matmul requires 2D tensors".to_string());
        }

        let (m, k) = (self.shape[0], self.shape[1]);
        let (k2, n) = (other.shape[0], other.shape[1]);

        if k != k2 {
            return Err("Matrix dimensions incompatible".to_string());
        }

        let mut result = Tensor::new(vec![m, n]);
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0f32;
                for p in 0..k {
                    sum += self.data[i * k + p] * other.data[p * n + j];
                }
                result.data[i * n + j] = sum;
            }
        }
        Ok(result)
    }

    pub fn simd_add(&self, other: &Tensor, width: usize) -> Result<Tensor, String> {
        if self.shape != other.shape {
            return Err("Shape mismatch".to_string());
        }

        if width == 0 {
            return Err("SIMD width must be at least 1".to_string());
        }

        let mut result = Tensor::new(self.shape.clone());
        let len = self.data.len();
        let chunks = len / width;

        for chunk in 0..chunks {
            let base = chunk * width;
            for i in 0..width {
                result.data[base + i] = self.data[base + i] + other.data[base + i];
            }
        }

        for i in (chunks * width)..len {
            result.data[i] = self.data[i] + other.data[i];
        }

        Ok(result)
    }
}

impl TensorModule {
    pub fn new() -> Self {
        Self {
            simd_enabled: false,
            simd_width: 1,
        }
    }

    pub fn with_simd(width: usize) -> Self {
        Self {
            simd_enabled: width > 1,
            simd_width: width,
        }
    }

    pub fn detect_simd_width() -> usize {
        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "sse2"))]
        {
            4
        }

        #[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "sse2")))]
        {
            1
        }
    }
}

impl Default for TensorModule {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ReplayDebugger {
    pub recording: Vec<ExecutionEvent>,
    pub breakpoints: Vec<Breakpoint>,
    pub current_frame: usize,
    pub replay_enabled: bool,
}

#[derive(Clone)]
pub struct ExecutionEvent {
    pub timestamp: u64,
    pub event_type: EventType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    Step,
    Call,
    Return,
    Branch,
    MemoryAccess,
    Exception,
    ActorMessageSent,
    ActorMessageProcessed,
    TaskScheduled,
    TaskCompleted,
}

pub struct Breakpoint {
    pub id: usize,
    pub location: BreakpointLocation,
    pub enabled: bool,
    pub condition: Option<String>,
}

pub enum BreakpointLocation {
    Line(usize),
    Function(String),
}

impl ReplayDebugger {
    pub fn new() -> Self {
        Self {
            recording: Vec::new(),
            breakpoints: Vec::new(),
            current_frame: 0,
            replay_enabled: false,
        }
    }

    pub fn record(&mut self, event: ExecutionEvent) {
        if self.replay_enabled {
            self.recording.push(event);
        }
    }

    pub fn set_breakpoint(&mut self, bp: Breakpoint) {
        self.breakpoints.push(bp);
    }

    pub fn enable_replay(&mut self) {
        self.replay_enabled = true;
    }

    pub fn disable_replay(&mut self) {
        self.replay_enabled = false;
    }

    pub fn replay(&self, from_frame: usize) -> Vec<&ExecutionEvent> {
        self.recording.iter().skip(from_frame).collect()
    }

    pub fn step_forward(&mut self) -> Option<ExecutionEvent> {
        if self.current_frame < self.recording.len() {
            let event = self.recording[self.current_frame].clone();
            self.current_frame += 1;
            Some(event)
        } else {
            None
        }
    }

    pub fn step_back(&mut self) -> Option<ExecutionEvent> {
        if self.current_frame > 0 {
            self.current_frame -= 1;
            Some(self.recording[self.current_frame].clone())
        } else {
            None
        }
    }

    pub fn goto(&mut self, frame: usize) -> Option<ExecutionEvent> {
        if frame < self.recording.len() {
            self.current_frame = frame;
            Some(self.recording[frame].clone())
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.recording.clear();
        self.current_frame = 0;
    }
}

impl Default for ReplayDebugger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_work_stealing_scheduler_spawn_task() {
        let scheduler = Arc::new(WorkStealingScheduler::new(2));
        let task_completed = Arc::new(AtomicBool::new(false));
        let marker = task_completed.clone();

        let handles = scheduler.clone().run();
        scheduler.spawn(Task {
            id: 0,
            name: "task".to_string(),
            payload: TaskPayload::Fn(Box::new(move || {
                marker.store(true, Ordering::SeqCst);
            })),
            state: TaskState::Pending,
        });

        scheduler.wait_for_idle();
        scheduler.shutdown();
        for handle in handles {
            handle.join().unwrap();
        }
        assert!(task_completed.load(Ordering::SeqCst));
    }

    #[test]
    fn test_spawn_scope_completes_all_tasks() {
        let scheduler = Arc::new(WorkStealingScheduler::new(2));
        let scope = scheduler.clone().create_scope();
        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        let handles = scheduler.clone().run();
        scope.spawn("scope-task", move || {
            completed_clone.store(true, Ordering::SeqCst);
        });
        scope.join();
        scheduler.shutdown();

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(completed.load(Ordering::SeqCst));
    }

    #[test]
    fn test_actor_system_spawn_and_process_mailbox() {
        let mut system = ActorSystem::new();
        system
            .spawn_actor(
                "test".to_string(),
                ActorHandler::Stateless(|msg| match msg.payload {
                    MessagePayload::Ping => ActorResponse::Pong,
                    _ => ActorResponse::Ok,
                }),
                SupervisionStrategy::OneForOne,
            )
            .unwrap();

        system
            .send(
                "test",
                ActorMessage {
                    sender: None,
                    payload: MessagePayload::Ping,
                },
            )
            .unwrap();

        let response = system.process_mailbox("test");
        assert!(matches!(response, Ok(ActorResponse::Pong)));
    }

    #[test]
    fn test_actor_supervision_restarts_failed_actor() {
        struct Failer;
        impl ActorStatefulHandler for Failer {
            fn handle(&mut self, _msg: ActorMessage) -> ActorResponse {
                ActorResponse::Error("boom".to_string())
            }
        }

        let mut system = ActorSystem::new();
        system
            .spawn_actor(
                "failer".to_string(),
                ActorHandler::Stateful(Box::new(Failer)),
                SupervisionStrategy::OneForOne,
            )
            .unwrap();

        system
            .send(
                "failer",
                ActorMessage {
                    sender: None,
                    payload: MessagePayload::Ping,
                },
            )
            .unwrap();

        let response = system.process_mailbox("failer");
        assert!(matches!(response, Ok(ActorResponse::Error(_))));
        let rule = system.supervision_tree.get("failer").unwrap();
        assert_eq!(rule.restart_count, 1);
    }

    #[test]
    fn test_tensor_add_and_simd_width_detection() {
        let a = Tensor::ones(vec![4, 4]);
        let b = Tensor::ones(vec![4, 4]);
        let c = a.simd_add(&b, 4).unwrap();
        assert_eq!(c.data, vec![2.0; 16]);
        assert!(TensorModule::detect_simd_width() >= 1);
    }

    #[test]
    fn test_replay_debugger_record_and_replay() {
        let mut debugger = ReplayDebugger::new();
        debugger.enable_replay();

        debugger.record(ExecutionEvent {
            timestamp: 1,
            event_type: EventType::TaskScheduled,
            data: vec![],
        });
        debugger.record(ExecutionEvent {
            timestamp: 2,
            event_type: EventType::TaskCompleted,
            data: vec![],
        });

        let events = debugger.replay(0);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, EventType::TaskScheduled);
        assert_eq!(events[1].event_type, EventType::TaskCompleted);

        debugger.goto(1);
        assert_eq!(debugger.current_frame, 1);
    }
}
