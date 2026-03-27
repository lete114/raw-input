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
    // --- Writing System Keys (evdev KEY_* codes) ---
    KeyA => 30,
    KeyB => 48,
    KeyC => 46,
    KeyD => 32,
    KeyE => 18,
    KeyF => 33,
    KeyG => 34,
    KeyH => 35,
    KeyI => 23,
    KeyJ => 36,
    KeyK => 37,
    KeyL => 38,
    KeyM => 50,
    KeyN => 49,
    KeyO => 24,
    KeyP => 25,
    KeyQ => 16,
    KeyR => 19,
    KeyS => 31,
    KeyT => 20,
    KeyU => 22,
    KeyV => 47,
    KeyW => 17,
    KeyX => 45,
    KeyY => 21,
    KeyZ => 44,
    Digit1 => 2,
    Digit2 => 3,
    Digit3 => 4,
    Digit4 => 5,
    Digit5 => 6,
    Digit6 => 7,
    Digit7 => 8,
    Digit8 => 9,
    Digit9 => 10,
    Digit0 => 11,
    Minus => 12,
    Equal => 13,
    BracketLeft => 26,
    BracketRight => 27,
    Backslash => 43,
    Semicolon => 39,
    Quote => 40,
    Backquote => 41,
    Comma => 51,
    Period => 52,
    Slash => 53,
    IntlBackslash => 86,

    // --- Functional Keys ---
    Enter => 28,
    Tab => 15,
    Space => 57,
    Backspace => 14,
    Escape => 1,
    ShiftLeft => 42,
    ShiftRight => 54,
    ControlLeft => 29,
    ControlRight => 97,
    AltLeft => 56,
    AltRight => 100,
    MetaLeft => 125,
    MetaRight => 126,
    CapsLock => 58,

    // --- Control Pad Section ---
    Insert => 110,
    Delete => 111,
    Home => 102,
    End => 107,
    PageUp => 104,
    PageDown => 109,

    // --- Arrow Pad Section ---
    ArrowUp => 103,
    ArrowDown => 108,
    ArrowLeft => 105,
    ArrowRight => 106,

    // --- Numpad Section ---
    NumLock => 69,
    NumpadDivide => 98,
    NumpadMultiply => 55,
    NumpadSubtract => 74,
    NumpadAdd => 78,
    NumpadEnter => 96,
    NumpadDecimal => 83,
    Numpad0 => 82,
    Numpad1 => 79,
    Numpad2 => 80,
    Numpad3 => 81,
    Numpad4 => 75,
    Numpad5 => 76,
    Numpad6 => 77,
    Numpad7 => 71,
    Numpad8 => 72,
    Numpad9 => 73,

    // --- Function Section ---
    F1 => 59,
    F2 => 60,
    F3 => 61,
    F4 => 62,
    F5 => 63,
    F6 => 64,
    F7 => 65,
    F8 => 66,
    F9 => 67,
    F10 => 68,
    F11 => 87,
    F12 => 88,
    F13 => 183,
    F14 => 184,
    F15 => 185,
    F16 => 186,
    F17 => 187,
    F18 => 188,
    F19 => 189,
    F20 => 190,
    F21 => 191,
    F22 => 192,
    F23 => 193,
    F24 => 194,
    PrintScreen => 99,
    ScrollLock => 70,
    Pause => 119,

    // --- International ---
    IntlYen => 124,
    IntlRo => 89,
}
