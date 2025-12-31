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