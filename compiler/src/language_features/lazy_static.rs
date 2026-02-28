/// Lazy Static Implementation
/// Provides compile-time verification and runtime initialization of static variables
/// 
/// Lazy statics are evaluated once at first access, caching the result
/// Thread-safe using Once and UnsafeCell pattern

use std::sync::{Once, Mutex};
use std::cell::UnsafeCell;

/// Represents a lazily-initialized static value
/// Generic over T to support any type that can be initialized
pub struct LazyStat<T> {
    once: Once,
    value: UnsafeCell<Option<T>>,
}

impl<T> LazyStat<T> {
    /// Create a new lazy static (only available at compile time)
    pub const fn new() -> Self {
        LazyStat {
            once: Once::new(),
            value: UnsafeCell::new(None),
        }
    }

    /// Get the lazily-initialized value
    /// Calls the closure on first access, caches result on subsequent calls
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.once.call_once(|| {
            unsafe {
                *self.value.get() = Some(f());
            }
        });
        unsafe { self.value.get().as_ref().unwrap().as_ref().unwrap() }
    }

    /// Get mutable reference (requires exclusive access)
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.value.get_mut().as_mut().and_then(|opt| opt.as_mut())
    }

    /// Check if initialized without blocking
    pub fn is_initialized(&self) -> bool {
        unsafe { self.value.get().as_ref().unwrap().is_some() }
    }
}

// Safety: LazyStat is Send if T is Send
unsafe impl<T: Send> Send for LazyStat<T> {}

// Safety: LazyStat is Sync if T is Sync (Once ensures thread-safe initialization)
unsafe impl<T: Sync> Sync for LazyStat<T> {}

/// Macro for declaring lazy statics
/// 
/// # Example
/// ```omni
/// lazy_static COUNT: i32 = 42;
/// lazy_static COMPUTED: Vec<i32> = compute_expensive();
/// ```
#[macro_export]
macro_rules! lazy_static {
    ($name:ident: $ty:ty = $init:expr) => {
        static $name: $crate::LazyStat<$ty> = $crate::LazyStat::new();
        
        // Thread-safe getter
        pub fn $name() -> &'static $ty {
            $name.get_or_init(|| $init)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_lazy_static_initialization() {
        let counter = Arc::new(AtomicUsize::new(0));
        let lazy: LazyStat<i32> = LazyStat::new();
        
        let c1 = counter.clone();
        let val1 = lazy.get_or_init(|| {
            c1.fetch_add(1, Ordering::SeqCst);
            42
        });
        
        assert_eq!(*val1, 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        
        // Second call should not re-initialize
        let c2 = counter.clone();
        let val2 = lazy.get_or_init(|| {
            c2.fetch_add(1, Ordering::SeqCst);
            100
        });
        
        assert_eq!(*val2, 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Still 1!
    }

    #[test]
    fn test_lazy_static_thread_safety() {
        let lazy = Arc::new(LazyStat::new());
        let mut handles = vec![];
        
        for _ in 0..10 {
            let l = lazy.clone();
            handles.push(std::thread::spawn(move || {
                let val = l.get_or_init(|| 42);
                *val
            }));
        }
        
        for handle in handles {
            assert_eq!(handle.join().unwrap(), 42);
        }
    }
}
