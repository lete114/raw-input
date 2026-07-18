//! # raw-input
//!
//! A cross-platform library for capturing and simulating global input events (keyboard and mouse).
//!
//! ## Core Components
//!
//! - **[`Core`]**: Manages the platform-specific event loop. Must be started to enable other features.
//! - **[`Listen`]**: Provides a way to subscribe to global input events without blocking them.
//! - **[`Simulate`]**: Allows programmatic injection of keyboard and mouse events.
//! - **[`Grab`]**: Enables intercepting and optionally blocking input events from reaching other applications.
//! - **[`Display`]**: Utilities for querying monitor information and cursor positions.
//!
//! ## Example
//!
//! ```no_run
//! use std::thread;
//! use std::time::Duration;
//!
//! use raw_input::{Core, Listen, Event};
//!
//! // 1. Start the core engine in a background thread
//! // (Crucial for processing Windows message loops)
//! thread::spawn(|| {
//!     Core::start().expect("Failed to start raw-input core");
//! });
//!
//! // 2. Subscribe to global events
//! Listen::start();
//! let handle = Listen::subscribe(|event| {
//!     match event {
//!         Event::KeyDown { key } => println!("Key pressed: {:?}", key),
//!         Event::MouseMove { delta } => println!("Mouse moved by: {}, {}", delta.x, delta.y),
//!         _ => {},
//!     }
//! });
//!
//! // 3. Manage the subscription lifecycle
//! thread::sleep(Duration::from_secs(5));
//! handle.pause();    // Stop receiving events temporarily
//!
//! thread::sleep(Duration::from_secs(2));
//! handle.resume();   // Start receiving events again
//!
//! thread::sleep(Duration::from_secs(2));
//! handle.unsubscribe(); // Permanently remove the listener
//! Listen::stop(); // Stop Listen
//! Core::stop();
//! ```

mod dispatcher;
mod event;
mod key;
mod platform;
mod subscription;

#[rustfmt::skip]
use crate::platform::{
    PlatformCore, CoreImpl, 
    PlatformDisplay, DisplayImpl, 
    PlatformGrab, GrabImpl,
    PlatformListen, ListenImpl,
    PlatformSimulate, SimulateImpl,
};

pub use crate::event::{Event, MouseButton, Point};
pub use crate::key::{Key, KeyCode};
pub use crate::subscription::SubscriptionHandle;

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// Information about a connected physical monitor.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct MonitorInfo {
    /// The OS-assigned name or identifier of the monitor.
    pub name: String,
    /// Indicates whether this is the primary monitor of the system.
    pub is_primary: bool,
    /// The starting coordinates (x, y) of the monitor in the global physical coordinate system.
    pub offset: (f64, f64),
    /// The physical resolution (width, height) of the monitor in pixels.
    pub size: (f64, f64),
    /// The UI scale factor (e.g., 1.0, 1.5, 2.0) for High-DPI support.
    pub scale_factor: f64,
}

impl MonitorInfo {
    /// Returns the width of the monitor.
    pub fn width(&self) -> f64 {
        self.size.0
    }

    /// Returns the height of the monitor.
    pub fn height(&self) -> f64 {
        self.size.1
    }
}

/// Errors that occur when trying to capture OS events.
///
/// # Note on macOS
/// On macOS, failing to set Accessibility permissions does not necessarily
/// trigger an error; the system may simply ignore events or return empty data.
#[derive(Debug)]
#[non_exhaustive]
pub enum CoreError {
    /// macOS: Failed to create an event tap.
    MacEventTapError,
    /// macOS: Failed to add the event tap to the run loop.
    MacLoopSourceError,
    /// Linux: Could not open the X11 display.
    LinuxMissingDisplayError,
    /// Linux: Keyboard-related error in the X11 or Wayland backend.
    LinuxKeyboardError,
    /// Linux: Failed to enable the XRecord context.
    LinuxRecordContextEnablingError,
    /// Linux: General XRecord context error.
    LinuxRecordContextError,
    /// Linux: The XRecord extension is missing or incompatible.
    LinuxXRecordExtensionError,
    /// Windows: Failed to set a low-level keyboard hook.
    WindowsKeyHookError(String),
    /// Windows: Failed to set a low-level mouse hook.
    WindowsMouseHookError(String),
    /// Windows: Failed to register for Raw Input devices.
    WindowsRegisterRawInputError(String),
}

/// The system background engine manager.
///
/// `Core` handles the lifecycle of the platform's native event loop.
///
/// # Example
/// ```no_run
/// use raw_input::Core;
///
/// // Initialize and start the background event loop
/// std::thread::spawn(|| {
///     if let Err(err) = Core::start() {
///         eprintln!("Failed to start core: {:?}", err);
///     }
/// });
///
/// Listen::start();
/// // ... your application logic ...
/// ```
pub struct Core;

impl Core {
    /// Starts the core engine.
    ///
    /// **This is a blocking operation**
    #[inline]
    pub fn start() -> Result<(), CoreError> {
        PlatformCore::start()
    }

    /// Checks if the core engine is runing.
    #[inline]
    pub fn is_runing() -> bool {
        PlatformCore::is_runing()
    }

    /// Pauses the core engine.
    #[inline]
    pub fn pause() {
        PlatformCore::pause();
    }

    /// Resumes the core engine.
    #[inline]
    pub fn resume() {
        PlatformCore::resume();
    }

    /// Stops the core engine.
    #[inline]
    pub fn stop() {
        PlatformCore::stop();
    }
}

/// Screen and monitor information provider.
///
/// # Example
/// ```no_run
/// use raw_input::Display;
///
/// let monitors = Display::get_available_monitors();
/// for monitor in monitors {
///     println!("Monitor: {} - Size: {:?}", monitor.name, monitor.size);
/// }
///
/// let scale = Display::get_scale_factor();
/// println!("Current UI Scale: {}", scale);
/// ```
pub struct Display;

impl Display {
    /// Returns the UI scale factor of the primary monitor.
    #[inline]
    pub fn get_scale_factor() -> f64 {
        PlatformDisplay::get_scale_factor()
    }

    /// Returns the current cursor position on the screen.
    #[inline]
    pub fn get_cursor_position() -> Option<(f64, f64)> {
        PlatformDisplay::get_cursor_position()
    }

    /// Returns the size of the primary monitor.
    #[inline]
    pub fn get_primary_screen_size() -> (f64, f64) {
        PlatformDisplay::get_primary_screen_size()
    }

    /// Returns the size of the virtual screen (all monitors combined).
    #[inline]
    pub fn get_virtual_screen_size() -> (f64, f64) {
        PlatformDisplay::get_virtual_screen_size()
    }

    /// Returns the virtual screen boundary across all monitors.
    /// (x, y, width, height) in logical units
    #[inline]
    pub fn get_virtual_screen_bounds() -> (f64, f64, f64, f64) {
        PlatformDisplay::get_virtual_screen_bounds()
    }

    /// Returns a list of all connected monitors.
    #[inline]
    pub fn get_available_monitors() -> Vec<MonitorInfo> {
        PlatformDisplay::get_available_monitors()
    }

    /// Returns information about the primary monitor.
    #[inline]
    pub fn get_primary_monitor() -> Option<MonitorInfo> {
        PlatformDisplay::get_primary_monitor()
    }

    /// Returns information about the monitor where the cursor currently is.
    #[inline]
    pub fn get_current_monitor() -> Option<MonitorInfo> {
        PlatformDisplay::get_current_monitor()
    }

    /// Returns information about the monitor that contains the specified point.
    #[inline]
    pub fn get_monitor_from_point(x: f64, y: f64) -> Option<MonitorInfo> {
        PlatformDisplay::get_monitor_from_point(x, y)
    }
}

/// Input interceptor for blocking or modifying events.
///
/// `Grab` allows you to prevent specific events from reaching other applications.
///
/// # Example
/// ```no_run
/// use raw_input::Grab;
///
/// // It must be started first
/// // Core::start(); // This is a blocking operation
///
/// // Block all keyboard input
/// Grab::start();  // defaults to blocking all input
///
/// // customizable
/// // Grab::mouse_move(true);
/// // Grab::mouse_wheel(true);
/// // Grab::mouse_button(true);
/// // Grab::keyboard(true);
///
/// // Stop grabbing later
/// // Grab::stop();
/// ```
pub struct Grab;

impl Grab {
    /// Starts grabbing (blocking) events.
    #[inline]
    pub fn start() {
        PlatformGrab::start();
    }

    /// Checks if grabbing is active.
    #[inline]
    pub fn is_runing() -> bool {
        PlatformGrab::is_runing()
    }

    /// Pauses grabbing events.
    #[inline]
    pub fn pause() {
        PlatformGrab::pause();
    }

    /// Resumes grabbing events.
    #[inline]
    pub fn resume() {
        PlatformGrab::resume();
    }

    /// Stops grabbing events.
    #[inline]
    pub fn stop() {
        PlatformGrab::stop();
    }

    /// Mouse move grab
    #[inline]
    pub fn mouse_move(enable: bool) {
        PlatformGrab::mouse_move(enable);
    }

    /// Mouse wheel grab
    #[inline]
    pub fn mouse_wheel(enable: bool) {
        PlatformGrab::mouse_wheel(enable);
    }

    /// Mouse button grab
    #[inline]
    pub fn mouse_button(enable: bool) {
        PlatformGrab::mouse_button(enable);
    }

    /// Keyboard grab
    #[inline]
    pub fn keyboard(enable: bool) {
        PlatformGrab::keyboard(enable);
    }
}

/// Global input listener for monitoring events.
///
/// This provides a way to observe keyboard and mouse actions without blocking them.
///
/// # Example
/// ```no_run
/// use raw_input::{Listen, Event};
///
/// // It must be started first
/// // Core::start(); // This is a blocking operation
///
/// // Start listening to all input
/// Listen::start();  // defaults to listening to all input
///
/// // customizable
/// // Listen::mouse_move(true);
/// // Listen::mouse_wheel(true);
/// // Listen::mouse_button(true);
/// // Listen::keyboard(true);
///
/// // Subscribe to all global input events
/// let handle = Listen::subscribe(|event| {
///     match event {
///         Event::KeyDown { key } => println!("Key pressed: {:?}", key),
///         Event::MouseMove { delta } => println!("Mouse delta: {:?}", delta),
///         _ => {},
///     }
/// });
///
/// // Use handle.pause(), handle.resume(), or handle.unsubscribe() to manage the lifecycle.
///
/// Listen::stop();
/// ```
pub struct Listen;

impl Listen {
    /// Starts listening for events.
    #[inline]
    pub fn start() {
        PlatformListen::start();
    }

    /// Checks if listening for events is active.
    #[inline]
    pub fn is_runing() -> bool {
        PlatformListen::is_runing()
    }

    /// Pauses listening for events.
    #[inline]
    pub fn pause() {
        PlatformListen::pause();
    }

    /// Resumes listening for events.
    #[inline]
    pub fn resume() {
        PlatformListen::resume();
    }

    /// Stops listening for events.
    #[inline]
    pub fn stop() {
        PlatformListen::stop();
    }

    /// Mouse move listening
    #[inline]
    pub fn mouse_move(enable: bool) {
        PlatformListen::mouse_move(enable);
    }

    /// Mouse wheel listening
    #[inline]
    pub fn mouse_wheel(enable: bool) {
        PlatformListen::mouse_wheel(enable);
    }

    /// Mouse button listening
    #[inline]
    pub fn mouse_button(enable: bool) {
        PlatformListen::mouse_button(enable);
    }

    /// Keyboard listening
    #[inline]
    pub fn keyboard(enable: bool) {
        PlatformListen::keyboard(enable);
    }

    /// Subscribe to input events
    #[inline]
    pub fn subscribe<F>(callback: F) -> SubscriptionHandle
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        PlatformListen::subscribe(callback)
    }

    /// Unsubscribe all listeners
    #[inline]
    pub fn unsubscribe_all() {
        PlatformListen::unsubscribe_all();
    }
}

/// Input simulator for synthesizing events.
///
/// Use `Simulate` to programmatically trigger keyboard and mouse actions.
///
/// # Example
/// ```no_run
/// use raw_input::{Simulate, Event, Key};
///
/// // Simulate pressing the 'A' key
/// Simulate::simulate(Event::KeyDown { key: Key::KeyA });
///
/// // Convenience methods for mouse
/// Simulate::mouse_move(100, 100);
/// ```
pub struct Simulate;

impl Simulate {
    /// Simulates an input event.
    #[inline]
    pub fn simulate(event: Event) {
        PlatformSimulate::simulate(event);
    }

    /// Simulates mouse movement by a delta.
    #[inline]
    pub fn mouse_move(delta_x: f64, delta_y: f64) {
        PlatformSimulate::mouse_move(delta_x, delta_y);
    }

    /// Simulates moving the mouse to an absolute position.
    #[inline]
    pub fn mouse_move_to(x: f64, y: f64) {
        PlatformSimulate::mouse_move_to(x, y);
    }

    /// Simulates mouse wheel scrolling.
    #[inline]
    pub fn mouse_wheel(delta_x: f64, delta_y: f64) {
        PlatformSimulate::mouse_wheel(delta_x, delta_y);
    }

    /// Simulates mouse button press or release.
    #[inline]
    pub fn mouse_button(button: MouseButton, down: bool) {
        PlatformSimulate::mouse_button(button, down);
    }

    /// Simulates key up or down.
    #[inline]
    pub fn keyboard(key: Key, down: bool) {
        PlatformSimulate::keyboard(key, down);
    }
}
