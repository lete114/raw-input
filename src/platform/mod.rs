// #[cfg(target_os = "linux")]
// mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use crate::{CoreError, Event, MonitorInfo, SubscriptionHandle};

pub(crate) struct PlatformCore;
pub(crate) struct PlatformListen;
pub(crate) struct PlatformGrab;
pub(crate) struct PlatformSimulate;
pub(crate) struct PlatformDisplay;

pub(crate) trait CoreImpl {
    /// Starts the core engine.
    fn start() -> Result<(), CoreError>;

    /// Checks if the core engine is runing.
    fn is_runing() -> bool;

    /// Pauses the core engine.
    fn pause();

    /// Resumes the core engine.
    fn resume();

    /// Stops the core engine.
    fn stop();
}

pub(crate) trait ListenImpl {
    /// Starts listening for events.
    fn start();

    /// Checks if listening for events is active.
    fn is_runing() -> bool;

    /// Pauses listening for events.
    fn pause();

    /// Resumes listening for events.
    fn resume();

    /// Stops listening for events.
    fn stop();

    /// Mouse move listening
    fn mouse_move(enable: bool);

    /// Mouse wheel listening
    fn mouse_wheel(enable: bool);

    /// Mouse button listening
    fn mouse_button(enable: bool);

    /// Keyboard listening
    fn keyboard(enable: bool);

    /// Subscribe to input events
    fn subscribe<F>(callback: F) -> SubscriptionHandle
    where
        F: Fn(Event) + Send + Sync + 'static;

    /// Unsubscribe all listeners
    fn unsubscribe_all();
}

pub(crate) trait GrabImpl {
    /// Starts grabbing (blocking) events.
    fn start();

    /// Checks if grabbing is active.
    fn is_runing() -> bool;

    /// Pauses grabbing events.
    fn pause();

    /// Resumes grabbing events.
    fn resume();

    /// Stops grabbing events.
    fn stop();

    /// Mouse move grab
    fn mouse_move(enable: bool);

    /// Mouse wheel grab
    fn mouse_wheel(enable: bool);

    /// Mouse button grab
    fn mouse_button(enable: bool);

    /// Keyboard grab
    fn keyboard(enable: bool);
}

pub(crate) trait SimulateImpl {
    /// Simulates an input event.
    fn simulate(event: Event);

    /// Simulates mouse movement by a delta.
    fn mouse_move(delta_x: f64, delta_y: f64);

    /// Simulates moving the mouse to an absolute position.
    fn mouse_move_to(x: f64, y: f64);

    /// Simulates mouse wheel scrolling.
    fn mouse_wheel(delta_x: f64, delta_y: f64);

    /// Simulates mouse button press or release.
    fn mouse_button(button: crate::MouseButton, down: bool);

    /// Simulates key up or down.
    fn keyboard(key: crate::Key, down: bool);
}

pub(crate) trait DisplayImpl {
    /// Returns the UI scale factor of the primary monitor.
    fn get_scale_factor() -> f64;

    /// Returns the current cursor position on the screen.
    fn get_cursor_position() -> Option<(f64, f64)>;

    /// Returns the size of the primary monitor.
    fn get_primary_screen_size() -> (f64, f64);

    /// Returns the size of the virtual screen (all monitors combined).
    fn get_virtual_screen_size() -> (f64, f64);

    /// Returns the bounds of the virtual screen (x, y, width, height) in logical units.
    fn get_virtual_screen_bounds() -> (f64, f64, f64, f64);

    /// Returns a list of all connected monitors.
    fn get_available_monitors() -> Vec<MonitorInfo>;

    /// Returns information about the primary monitor.
    fn get_primary_monitor() -> Option<MonitorInfo>;

    /// Returns information about the monitor where the cursor currently is.
    fn get_current_monitor() -> Option<MonitorInfo>;

    /// Returns information about the monitor at the given point (x, y).
    fn get_monitor_from_point(x: f64, y: f64) -> Option<MonitorInfo>;
}
