use crate::key::{Key, KeyCode};

macro_rules! keymap {
    ($($key:ident => $code:expr),+ $(,)?) => {
        pub fn key_to_code(key: Key) -> Option<KeyCode> {
            match key {
                $(Key::$key => Some($code),)+
                _ => None,
            }
        }

        pub fn code_to_key(scancode: KeyCode) -> Key {
            match scancode {
                $($code => Key::$key,)+
                _ => Key::Unidentified,
            }
        }
    };
}

keymap! {
    // --- Writing System Keys ---
    KeyA => 0x00,
    KeyS => 0x01,
    KeyD => 0x02,
    KeyF => 0x03,
    KeyH => 0x04,
    KeyG => 0x05,
    KeyZ => 0x06,
    KeyX => 0x07,
    KeyC => 0x08,
    KeyV => 0x09,
    KeyB => 0x0B,
    KeyQ => 0x0C,
    KeyW => 0x0D,
    KeyE => 0x0E,
    KeyR => 0x0F,
    KeyY => 0x10,
    KeyT => 0x11,
    Digit1 => 0x12,
    Digit2 => 0x13,
    Digit3 => 0x14,
    Digit4 => 0x15,
    Digit5 => 0x17,
    Digit6 => 0x16,
    Digit7 => 0x1A,
    Digit8 => 0x1C,
    Digit9 => 0x19,
    Digit0 => 0x1D,
    Equal => 0x18,
    Minus => 0x1B,
    BracketRight => 0x1E,
    BracketLeft => 0x21,
    KeyO => 0x1F,
    KeyU => 0x20,
    KeyI => 0x22,
    KeyP => 0x23,
    KeyL => 0x25,
    KeyJ => 0x26,
    KeyK => 0x28,
    KeyN => 0x2D,
    KeyM => 0x2E,
    Quote => 0x27,
    Semicolon => 0x29,
    Backslash => 0x2A,
    Comma => 0x2B,
    Slash => 0x2C,
    Period => 0x2F,
    Backquote => 0x32,
    IntlBackslash => 0x0A,

    // --- Functional Keys ---
    Enter => 0x24,
    Tab => 0x30,
    Space => 0x31,
    Backspace => 0x33,
    Escape => 0x35,
    MetaRight => 0x36,
    MetaLeft => 0x37,
    ShiftLeft => 0x38,
    CapsLock => 0x39,
    AltLeft => 0x3A,
    ControlLeft => 0x3B,
    ShiftRight => 0x3C,
    AltRight => 0x3D,
    ControlRight => 0x3E,

    // --- Control Pad Section ---
    // Help => 0x72, // Insert and Help often share keycodes on the Apple keyboard.
    Home => 0x73,
    PageUp => 0x74,
    Delete => 0x75,
    End => 0x77,
    PageDown => 0x79,
    Insert => 0x72,

    // --- Arrow Pad Section ---
    ArrowLeft => 0x7B,
    ArrowRight => 0x7C,
    ArrowDown => 0x7D,
    ArrowUp => 0x7E,

    // --- Numpad Section ---
    NumpadDecimal => 0x41,
    NumpadMultiply => 0x43,
    NumpadAdd => 0x45,
    NumLock => 0x47,
    NumpadDivide => 0x4B,
    NumpadEnter => 0x4C,
    NumpadSubtract => 0x4E,
    Numpad0 => 0x52,
    Numpad1 => 0x53,
    Numpad2 => 0x54,
    Numpad3 => 0x55,
    Numpad4 => 0x56,
    Numpad5 => 0x57,
    Numpad6 => 0x58,
    Numpad7 => 0x59,
    Numpad8 => 0x5B,
    Numpad9 => 0x5C,

    // --- Function Section ---
    F1 => 0x7A,
    F2 => 0x78,
    F3 => 0x63,
    F4 => 0x76,
    F5 => 0x60,
    F6 => 0x61,
    F7 => 0x62,
    F8 => 0x64,
    F9 => 0x65,
    F10 => 0x6D,
    F11 => 0x67,
    F12 => 0x6F,
    F13 => 0x69,
    F14 => 0x6B,
    F15 => 0x71,
    F16 => 0x6A,
    F17 => 0x40,
    F18 => 0x4F,
    F19 => 0x50,
    F20 => 0x5A,

    // --- International ---
    IntlYen => 0x5D,
    IntlRo => 0x5E,
}
