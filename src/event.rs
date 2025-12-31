#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

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

/// Platform-specific key code type.
#[cfg(not(target_os = "macos"))]
pub type KeyCode = u32;
// #[cfg(target_os = "macos")]
// pub type KeyCode = crate::CGKeyCode;

/// Represents raw hardware scan codes or virtual key codes from different OS layers.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum RawKey {
    ScanCode(KeyCode),
    WinVirtualKeycode(KeyCode),
    LinuxXorgKeycode(KeyCode),
    LinuxConsoleKeycode(KeyCode),
    MacVirtualKeycode(KeyCode),
}

#[rustfmt::skip]
/// A high-level representation of keyboard keys.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Key {
    /// Alt key on Linux and Windows (option key on macOS)
    Alt,
    AltGr,
    Backspace,
    CapsLock,
    ControlLeft,
    ControlRight,
    Delete,
    DownArrow,
    End,
    Escape,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
    Home,
    LeftArrow,
    /// Also known as "Windows", "Super", or "Command"
    MetaLeft,
    /// Also known as "Windows", "Super", or "Command"
    MetaRight,
    PageDown,
    PageUp,
    Return,
    RightArrow,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    UpArrow,
    PrintScreen,
    ScrollLock,
    Pause,
    NumLock,
    BackQuote,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    Minus,
    Equal,
    KeyQ, KeyW, KeyE, KeyR, KeyT, KeyY, KeyU, KeyI, KeyO, KeyP,
    LeftBracket,
    RightBracket,
    KeyA, KeyS, KeyD, KeyF, KeyG, KeyH, KeyJ, KeyK, KeyL,
    SemiColon,
    Quote,
    BackSlash,
    IntlBackslash,
    IntlRo,   // Brazilian /? and Japanese _ 'ro'
    IntlYen,  // Japanese Henkan (Convert) key.
    KanaMode, // Japanese Hiragana/Katakana key.
    KeyZ, KeyX, KeyC, KeyV, KeyB, KeyN, KeyM,
    Comma,
    Dot,
    Slash,
    Insert,
    KpReturn,
    KpMinus,
    KpPlus,
    KpMultiply,
    KpDivide,
    KpDecimal,
    KpEqual,
    KpComma,
    Kp0, Kp1, Kp2, Kp3, Kp4, Kp5, Kp6, Kp7, Kp8, Kp9,
    VolumeUp,
    VolumeDown,
    VolumeMute,
    Lang1, // Korean Hangul/English toggle key, and as the Kana key on the Apple Japanese keyboard.
    Lang2, // Korean Hanja conversion key, and as the Eisu key on the Apple Japanese keyboard.
    Lang3, // Japanese Katakana key.
    Lang4, // Japanese Hiragana key.
    Lang5, // Japanese Zenkaku/Hankaku (Fullwidth/halfwidth) key.
    Function,
    Apps,
    Cancel,
    Clear,
    Kana,
    Hangul,
    Junja,
    Final,
    Hanja,
    Hanji,
    Print,
    Select,
    Execute,
    Help,
    Sleep,
    Separator,
    Unknown(u32),
    RawKey(RawKey),
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
    KeyDown { key: Key },
    /// Keyboard key release.
    KeyUp { key: Key },
}
