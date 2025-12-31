use crate::event::Event;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};

/// Represents the current lifecycle state of a subscriber.
#[derive(PartialEq)]
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
