#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::key::{Key, KeyCode};

/// Represents the standard buttons on a mouse.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

/// A simple coordinate point using integers, typically for pixel positions.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// A coordinate point using floating-point numbers, used for precise deltas or scaling.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct FloatPoint {
    pub x: f64,
    pub y: f64,
}

/// The main event enum containing all possible input actions.
///
/// # Example
/// ```
/// use raw_input::{Event, Key};
///
/// fn handle_event(event: Event) {
///     match event {
///         Event::KeyDown { key: Key::Escape } => println!("Escape pressed!"),
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Event {
    /// Mouse movement with pixel delta.
    MouseMove { delta: Point },
    /// Mouse wheel rotation with floating point precision.
    MouseWheel { delta: FloatPoint },
    /// Mouse button press.
    MouseDown { button: MouseButton },
    /// Mouse button release.
    MouseUp { button: MouseButton },
    /// Keyboard key press.
    ///
    /// `code` is a platform-specific raw key identifier (e.g. scancode or virtual key).
    /// It is optional and may be unavailable on some platforms or synthetic events.
    KeyDown { key: Key, code: Option<KeyCode> },
    /// Keyboard key release.
    ///
    /// `code` is a platform-specific raw key identifier (e.g. scancode or virtual key).
    /// It is optional and may be unavailable on some platforms or synthetic events.
    KeyUp { key: Key, code: Option<KeyCode> },
}
