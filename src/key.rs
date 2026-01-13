/// Platform raw key code (OS-dependent numeric value)
pub type KeyCode = u32;

#[rustfmt::skip]
/// A high-level representation of keyboard keys based on W3C "code" values.
/// 
/// Reference: https://www.w3.org/TR/uievents-code/
/// 
/// Physical key codes aligned with W3C KeyboardEvent.code.
/// 
/// These represent key positions, not produced characters.
/// 
/// Layout- and locale-independent.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Key {
    // --- Writing System Keys ---
    Backquote,    // ` ~
    Backslash,    // \ |
    BracketLeft,  // [ {
    BracketRight, // ] }
    Comma,        // , <
    Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
    Equal,        // = +
    IntlBackslash,
    IntlRo,
    IntlYen,
    KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM,
    KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
    Minus,        // - _
    Period,       // . >
    Quote,        // ' "
    Semicolon,    // ; :
    Slash,        // / ?

    // --- Functional Keys ---
    AltLeft,      // Alt or Option (macOS)
    AltRight,
    Backspace,
    CapsLock,
    ContextMenu,  // Apps key
    ControlLeft,
    ControlRight,
    Enter,        // Return
    MetaLeft,     // Command (macOS), Windows, or Super
    MetaRight,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    Convert,     // IME convert key
    // Lang1,
    // Lang2,
    // Lang3,
    // Lang4,
    // Lang5,
    NonConvert,  // IME non-convert key

    // --- Control Pad Section ---
    Delete,
    End,
    Help,       // macOS-only
    Home,
    Insert,
    PageDown,
    PageUp,

    // --- Arrow Pad Section ---
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,

    // --- Numpad Section ---
    NumLock,
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd,  
    // NumpadBackspace,
    // NumpadClear,
    // NumpadClearEntry,
    // NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    // NumpadEqual,
    // NumpadHash,
    // NumpadMemoryAdd,
    // NumpadMemoryClear,
    // NumpadMemoryRecall,
    // NumpadMemoryStore,
    // NumpadMemorySubtract,
    NumpadMultiply,
    // NumpadParenLeft,
    // NumpadParenRight,
    // NumpadStar,
    NumpadSubtract,

    // --- Function Section ---
    Escape,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
    PrintScreen,
    ScrollLock,
    Pause,
    // Fn,           // Function key (usually hardware level)
    // FnLock,

    // --- Media Keys ---
    // BrowserBack,
    // BrowserFavorites,
    // BrowserForward,
    // BrowserHome,
    // BrowserRefresh,
    // BrowserSearch,
    // BrowserStop,
    // Eject,
    // LaunchApp1,
    // LaunchApp2,
    // LaunchMail,
    // MediaPlayPause,
    // MediaSelect,
    // MediaStop,
    // MediaTrackNext,
    // MediaTrackPrevious,
    // Power,
    // Sleep,
    // AudioVolumeDown,
    // AudioVolumeMute,
    // AudioVolumeUp,
    // WakeUp,

    // --- Legacy/Special ---
    // Hyper,
    // Super,
    // Turbo,
    // Abort,
    // Resume,
    // Suspend,
    // Again,
    // Copy,
    // Cut,
    // Find,
    // Open,
    // Paste,
    // Props,
    // Select,
    // Undo,
    // Hiragana,
    // Katakana,
    Unidentified,
}

impl Default for Key {
    fn default() -> Self {
        Key::Unidentified
    }
}
