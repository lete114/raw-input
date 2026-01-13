#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

// #[cfg(target_os = "linux")]
// mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

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
///     Core::start().expect("Failed to start raw-input core");
/// });
/// ```
pub struct Core;

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

/// Information about a connected physical monitor.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct MonitorInfo {
    /// The OS-assigned name or identifier of the monitor.
    pub name: String,
    /// Indicates whether this is the primary monitor of the system.
    pub is_primary: bool,
    /// The starting coordinates (x, y) of the monitor in the global physical coordinate system.
    pub offset: (i32, i32),
    /// The physical resolution (width, height) of the monitor in pixels.
    pub size: (i32, i32),
    /// The UI scale factor (e.g., 1.0, 1.5, 2.0) for High-DPI support.
    pub scale_factor: f64,
}

impl MonitorInfo {
    /// Returns the width of the monitor.
    pub fn width(&self) -> i32 {
        self.size.0
    }

    /// Returns the height of the monitor.
    pub fn height(&self) -> i32 {
        self.size.1
    }
}
