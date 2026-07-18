use crate::dispatcher::{CALLBACKS, Status};

/// A handle that allows control over an active event subscription.
///
/// It can be used to pause, resume, or permanently remove a callback.
pub struct SubscriptionHandle {
    pub(crate) id: u64,
}

impl SubscriptionHandle {
    /// Pauses the subscription. The callback will not be executed until `resume` is called.
    ///
    /// # Example
    /// ```no_run
    /// handle.pause();
    /// ```
    pub fn pause(&self) {
        if let Some(mut subscriber) = CALLBACKS.get_mut(&self.id) {
            subscriber.status = Status::Paused;
        }
    }

    /// Resumes a previously paused subscription.
    ///
    /// # Example
    /// ```no_run
    /// handle.resume();
    /// ```
    pub fn resume(&self) {
        if let Some(mut subscriber) = CALLBACKS.get_mut(&self.id) {
            subscriber.status = Status::Active;
        }
    }

    /// Removes the subscription from the dispatcher.
    /// The callback will be dropped and never called again.
    ///
    /// # Example
    /// ```no_run
    /// handle.unsubscribe();
    /// ```
    pub fn unsubscribe(self) {
        CALLBACKS.remove(&self.id);
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;
    use crate::dispatcher::{NEXT_ID, Subscriber, dispatch, remove_all};
    use crate::event::Event;
    use crate::key::Key;

    fn dummy_event() -> Event {
        Event::KeyDown { key: Key::Escape, code: None }
    }

    fn insert_callback() -> SubscriptionHandle {
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        CALLBACKS.insert(id, Subscriber {
            status: Status::Active,
            callback: Box::new(|_| {}),
        });
        SubscriptionHandle { id }
    }

    #[serial]
    #[test]
    fn test_subscription_pause_resume_unsubscribe() {
        remove_all();
        let handle = insert_callback();

        // Initially active
        assert_eq!(
            CALLBACKS.get(&handle.id).unwrap().status,
            Status::Active
        );

        // Pause
        handle.pause();
        assert_eq!(
            CALLBACKS.get(&handle.id).unwrap().status,
            Status::Paused
        );

        // Resume
        handle.resume();
        assert_eq!(
            CALLBACKS.get(&handle.id).unwrap().status,
            Status::Active
        );

        // Unsubscribe
        let saved_id = handle.id;
        handle.unsubscribe();
        assert!(CALLBACKS.get(&saved_id).is_none());

        remove_all();
    }

    #[serial]
    #[test]
    fn test_subscription_unsubscribe_stops_dispatch() {
        remove_all();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        CALLBACKS.insert(id, Subscriber {
            status: Status::Active,
            callback: Box::new(move |_| { called_clone.store(true, std::sync::atomic::Ordering::SeqCst); }),
        });
        let handle = SubscriptionHandle { id };

        // Unsubscribe then dispatch
        handle.unsubscribe();
        dispatch(dummy_event());
        assert!(!called.load(std::sync::atomic::Ordering::SeqCst), "unsubscribed callback should not be called");

        remove_all();
    }

    #[serial]
    #[test]
    fn test_subscription_pause_resume_toggle() {
        remove_all();
        let handle = insert_callback();

        handle.pause();
        assert_eq!(
            CALLBACKS.get(&handle.id).unwrap().status,
            Status::Paused
        );

        handle.resume();
        assert_eq!(
            CALLBACKS.get(&handle.id).unwrap().status,
            Status::Active
        );

        // Pause again
        handle.pause();
        assert_eq!(
            CALLBACKS.get(&handle.id).unwrap().status,
            Status::Paused
        );

        remove_all();
    }
}
