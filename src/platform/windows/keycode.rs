use crate::key::{Key, KeyCode};

// from: https://github.com/rustdesk-org/rdev/blob/a90dbe1172f8832f54c97c62e823c5a34af5fdfe/src/keycodes/windows.rs
macro_rules! keymap {
     ($($key:ident, $code:literal, $scancode:literal),*) => {
        pub fn key_to_code(key: Key) -> Option<KeyCode> {
            match key {
                $(Key::$key => Some($code),)+
                _=>None,
            }
        }

        pub fn code_to_key(code: KeyCode) -> Key {
            #[allow(unreachable_patterns)]
            match code {
                $($code => Key::$key,)+
                _ => Key::default()
            }
        }

        pub fn key_to_scancode(key: Key) -> Option<KeyCode> {
            match key {
                $(Key::$key => Some($scancode),)+
                _=>None,
            }
        }
        #[allow(dead_code)]
        pub fn scancode_to_key(scancode: KeyCode) -> Key{
            match scancode {
                0 => Key::default(),
                $($scancode => Key::$key,)+
                _ => Key::default(),
            }
        }
        #[allow(dead_code)]
        pub fn get_win_key(keycode: KeyCode, scancode: KeyCode) -> Key{
            let key = code_to_key(keycode);
            let scancode_key = scancode_to_key(scancode);

            if key == Key::AltRight || key == Key::NumpadDivide || key == Key::ControlRight {
                // note: Alt and AltRight have same scancode.
                // slash and divide.
                // left control and right control .
                key
            } else if scancode_key != Key::default() {
                // note: numpad should use scancode directly,
                scancode_key
            } else {
                key
            }
        }

        pub fn get_win_codes(key: Key) -> Option<(KeyCode, KeyCode)>{
            let keycode = key_to_code(key)?;
            let key = if key == Key::default() {
                code_to_key(keycode)
            }else{
                key
            };
            let scancode = key_to_scancode(key)?;
            Some((keycode, scancode))
        }
    };
}

keymap! {
    // --- Writing System Keys ---
    KeyA, 65, 0x1E,
    KeyB, 66, 0x30,
    KeyC, 67, 0x2E,
    KeyD, 68, 0x20,
    KeyE, 69, 0x12,
    KeyF, 70, 0x21,
    KeyG, 71, 0x22,
    KeyH, 72, 0x23,
    KeyI, 73, 0x17,
    KeyJ, 74, 0x24,
    KeyK, 75, 0x25,
    KeyL, 76, 0x26,
    KeyM, 77, 0x32,
    KeyN, 78, 0x31,
    KeyO, 79, 0x18,
    KeyP, 80, 0x19,
    KeyQ, 81, 0x10,
    KeyR, 82, 0x13,
    KeyS, 83, 0x1F,
    KeyT, 84, 0x14,
    KeyU, 85, 0x16,
    KeyV, 86, 0x2F,
    KeyW, 87, 0x11,
    KeyX, 88, 0x2D,
    KeyY, 89, 0x15,
    KeyZ, 90, 0x2C,

    Digit1, 49, 0x02,
    Digit2, 50, 0x03,
    Digit3, 51, 0x04,
    Digit4, 52, 0x05,
    Digit5, 53, 0x06,
    Digit6, 54, 0x07,
    Digit7, 55, 0x08,
    Digit8, 56, 0x09,
    Digit9, 57, 0x0A,
    Digit0, 48, 0x0B,

    Backquote, 192, 0x29,
    Backslash, 220, 0x2B,
    BracketLeft, 219, 0x1A,
    BracketRight, 221, 0x1B,
    Comma, 188, 0x33,
    Equal, 187, 0x0D,
    Minus, 189, 0x0C,
    Period, 190, 0x34,
    Quote, 222, 0x28,
    Semicolon, 186, 0x27,
    Slash, 191, 0x35,
    IntlBackslash, 226, 0x56,

    // --- Functional Keys ---
    AltLeft, 164, 0x38,
    AltRight, 165, 0xE038,
    Backspace, 0x08, 0x0E,
    CapsLock, 20, 0x3A,
    ContextMenu, 93, 0xE05D,
    ControlLeft, 162, 0x1D,
    ControlRight, 163, 0xE01D,
    Enter, 13, 0x1C,
    Escape, 27, 0x01,
    MetaLeft, 91, 0xE05B,
    MetaRight, 92, 0xE05C,
    ShiftLeft, 160, 0x2A,
    ShiftRight, 161, 0x36,
    Space, 32, 0x39,
    Tab, 0x09, 0x0F,
    Convert, 0x1C, 0x79,
    NonConvert, 0x1D, 0x7B,

    // --- Control Pad Section ---
    Delete, 46, 0xE053,
    End, 35, 0xE04F,
    Home, 36, 0xE047,
    Insert, 45, 0xE052,
    PageDown, 34, 0xE051,
    PageUp, 33, 0xE049,

    // --- Arrow Pad Section ---
    ArrowDown, 40, 0xE050,
    ArrowLeft, 37, 0xE04B,
    ArrowRight, 39, 0xE04D,
    ArrowUp, 38, 0xE048,

    // --- Numpad Section ---
    NumLock, 144, 0x45,
    Numpad0, 96, 0x52,
    Numpad1, 97, 0x4F,
    Numpad2, 98, 0x50,
    Numpad3, 99, 0x51,
    Numpad4, 100, 0x4B,
    Numpad5, 101, 0x4C,
    Numpad6, 102, 0x4D,
    Numpad7, 103, 0x47,
    Numpad8, 104, 0x48,
    Numpad9, 105, 0x49,
    NumpadAdd, 107, 0x4E,
    NumpadDecimal, 110, 0x53,
    NumpadDivide, 111, 0xE035,
    NumpadEnter, 13, 0xE01C,
    NumpadMultiply, 106, 0x37,
    NumpadSubtract, 109, 0x4A,

    // --- Function Section ---
    F1, 112, 0x3B,
    F2, 113, 0x3C,
    F3, 114, 0x3D,
    F4, 115, 0x3E,
    F5, 116, 0x3F,
    F6, 117, 0x40,
    F7, 118, 0x41,
    F8, 119, 0x42,
    F9, 120, 0x43,
    F10, 121, 0x44,
    F11, 122, 0x57,
    F12, 123, 0x58,
    F13, 124, 0x64,
    F14, 125, 0x65,
    F15, 126, 0x66,
    F16, 127, 0x67,
    F17, 128, 0x68,
    F18, 129, 0x69,
    F19, 130, 0x6A,
    F20, 131, 0x6B,
    F21, 132, 0x6C,
    F22, 133, 0x6D,
    F23, 134, 0x6E,
    F24, 135, 0x76,

    PrintScreen, 44, 0xE037,
    ScrollLock, 145, 0x46,
    Pause, 19, 0xE145,

    // --- International ---
    IntlRo, 0x00E2, 0x73,
    IntlYen, 0x00DC, 0x7D
}
