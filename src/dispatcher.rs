use crate::event::Event;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};

/// Represents the current lifecycle state of a subscriber.
#[derive(Debug, PartialEq)]
pub enum Status {
    /// Subscriber will receive events.
    Active,
    /// Subscriber is temporarily ignored.
    Paused,
}

/// Internal container for a subscription callback.
pub(crate) struct Subscriber {
    pub(crate) status: Status,
    pub(crate) callback: Box<dyn Fn(Event) + Send + Sync + 'static>,
}

/// Global counter to generate unique subscription IDs.
pub(crate) static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Thread-safe global map storing all active event subscribers.
pub(crate) static CALLBACKS: Lazy<DashMap<u64, Subscriber>> = Lazy::new(DashMap::new);

/// Dispatches an event to all active subscribers.
///
/// This function iterates through all registered callbacks and executes them
/// if their status is set to `Active`.
pub(crate) fn dispatch(event: Event) {
    for guard in CALLBACKS.iter() {
        if guard.status == Status::Active {
            (guard.callback)(event);
        }
    }
}

/// Clears all subscribers and resets the ID counter.
pub(crate) fn remove_all() {
    CALLBACKS.clear();
    NEXT_ID.store(0, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;
    use crate::event::Event;
    use crate::key::Key;

    fn dummy_event() -> Event {
        Event::KeyDown { key: Key::Escape, code: None }
    }

    #[serial]
    #[test]
    fn test_dispatch_calls_active_callbacks() {
        remove_all();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        CALLBACKS.insert(id, Subscriber {
            status: Status::Active,
            callback: Box::new(move |_| { called_clone.store(true, Ordering::SeqCst); }),
        });
        dispatch(dummy_event());
        assert!(called.load(Ordering::SeqCst), "active callback should be called");
        remove_all();
    }

    #[serial]
    #[test]
    fn test_dispatch_skips_paused_callbacks() {
        remove_all();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        CALLBACKS.insert(id, Subscriber {
            status: Status::Paused,
            callback: Box::new(move |_| { called_clone.store(true, Ordering::SeqCst); }),
        });
        dispatch(dummy_event());
        assert!(!called.load(Ordering::SeqCst), "paused callback should not be called");
        remove_all();
    }

    #[serial]
    #[test]
    fn test_dispatch_calls_only_active_among_mixed() {
        remove_all();
        let active_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let paused_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let a = active_called.clone();
        let p = paused_called.clone();
        let id1 = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let id2 = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        CALLBACKS.insert(id1, Subscriber {
            status: Status::Active,
            callback: Box::new(move |_| { a.store(true, Ordering::SeqCst); }),
        });
        CALLBACKS.insert(id2, Subscriber {
            status: Status::Paused,
            callback: Box::new(move |_| { p.store(true, Ordering::SeqCst); }),
        });
        dispatch(dummy_event());
        assert!(active_called.load(Ordering::SeqCst), "active callback should be called");
        assert!(!paused_called.load(Ordering::SeqCst), "paused callback should not be called");
        remove_all();
    }

    #[serial]
    #[test]
    fn test_remove_all_clears_callbacks() {
        remove_all();
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        CALLBACKS.insert(id, Subscriber {
            status: Status::Active,
            callback: Box::new(|_| {}),
        });
        assert_eq!(CALLBACKS.len(), 1, "callback should exist before remove_all");
        remove_all();
        assert!(CALLBACKS.is_empty(), "callbacks should be empty after remove_all");
        assert_eq!(NEXT_ID.load(Ordering::SeqCst), 0, "NEXT_ID should reset to 0");
    }
}
