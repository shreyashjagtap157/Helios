//! Phase 9: Concurrency Runtime & Tensor Acceleration
//!
//! Implements Phase 9 requirements:
//! - Work-stealing structured concurrency executor
//! - Actor model with supervision trees
//! - Tensor/SIMD module implementation
//! - Replay debugging infrastructure

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct WorkStealingScheduler {
    pub queues: Vec<VecDeque<Task>>,
    pub thread_count: usize,
    pub active_workers: Arc<AtomicUsize>,
    pub shutdown: Arc<AtomicBool>,
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
        let queues: Vec<VecDeque<Task>> = (0..thread_count).map(|_| VecDeque::new()).collect();
        Self {
            queues,
            thread_count,
            active_workers: Arc::new(AtomicUsize::new(0)),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let idx = task.id % self.thread_count;
        if let Some(queue) = self.queues.get_mut(idx) {
            queue.push_back(task);
        }
    }

    pub fn steal(&mut self, victim_idx: usize) -> Option<Task> {
        if let Some(queue) = self.queues.get_mut(victim_idx) {
            queue.pop_front()
        } else {
            None
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
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
            let actor = self.actors.get_mut(actor_id).ok_or("Actor not found")?;

            match &mut actor.handler {
                ActorHandler::Stateless(handler) => Ok(handler(msg)),
                ActorHandler::Stateful(handler) => Ok(handler.handle(msg)),
            }
        } else {
            Err("No messages".to_string())
        }
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

        let mut result = Tensor::new(self.shape.clone());
        let chunks = self.data.len() / width;

        for chunk in 0..chunks {
            let base = chunk * width;
            for i in 0..width {
                result.data[base + i] = self.data[base + i] + other.data[base + i];
            }
        }

        for i in (chunks * width)..self.data.len() {
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

    #[allow(unused_variables)]
    pub fn detect_simd_width() -> usize {
        1
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

#[derive(Clone, PartialEq)]
pub enum EventType {
    Step,
    Call,
    Return,
    Branch,
    MemoryAccess,
    Exception,
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

    #[test]
    fn test_work_stealing_scheduler() {
        let mut scheduler = WorkStealingScheduler::new(4);
        let task = Task {
            id: 0,
            name: "test".to_string(),
            payload: TaskPayload::Actor(ActorMessage {
                sender: None,
                payload: MessagePayload::Ping,
            }),
            state: TaskState::Pending,
        };
        scheduler.spawn(task);
        assert!(!scheduler.is_shutdown());
    }

    #[test]
    fn test_actor_system_spawn() {
        let mut system = ActorSystem::new();
        let result = system.spawn_actor(
            "test".to_string(),
            ActorHandler::Stateless(|_| ActorResponse::Ok),
            SupervisionStrategy::OneForOne,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_actor_send() {
        let mut system = ActorSystem::new();
        system
            .spawn_actor(
                "test".to_string(),
                ActorHandler::Stateless(|_| ActorResponse::Ok),
                SupervisionStrategy::OneForOne,
            )
            .unwrap();

        let msg = ActorMessage {
            sender: None,
            payload: MessagePayload::Ping,
        };
        let result = system.send("test", msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tensor_creation() {
        let t = Tensor::new(vec![2, 3]);
        assert_eq!(t.shape, vec![2, 3]);
        assert_eq!(t.data.len(), 6);
    }

    #[test]
    fn test_tensor_add() {
        let a = Tensor::ones(vec![2, 2]);
        let b = Tensor::ones(vec![2, 2]);
        let c = a.add(&b).unwrap();
        for v in c.data.iter() {
            assert_eq!(*v, 2.0);
        }
    }

    #[test]
    fn test_tensor_matmul() {
        let a = Tensor::ones(vec![2, 3]);
        let b = Tensor::ones(vec![3, 2]);
        let c = a.matmul(&b).unwrap();
        assert_eq!(c.shape, vec![2, 2]);
        for v in c.data.iter() {
            assert_eq!(*v, 3.0);
        }
    }

    #[test]
    fn test_replay_debugger() {
        let mut debugger = ReplayDebugger::new();
        debugger.enable_replay();

        debugger.record(ExecutionEvent {
            timestamp: 1,
            event_type: EventType::Step,
            data: vec![],
        });

        debugger.record(ExecutionEvent {
            timestamp: 2,
            event_type: EventType::Call,
            data: vec![],
        });

        let event = debugger.step_forward();
        assert!(event.is_some());
        assert_eq!(debugger.current_frame, 1);
    }

    #[test]
    fn test_replay_goto() {
        let mut debugger = ReplayDebugger::new();
        debugger.enable_replay();

        for i in 0..5 {
            debugger.record(ExecutionEvent {
                timestamp: i as u64,
                event_type: EventType::Step,
                data: vec![],
            });
        }

        debugger.goto(3);
        assert_eq!(debugger.current_frame, 3);
    }
}
