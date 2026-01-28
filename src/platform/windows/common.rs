use std::{
    ffi::c_void,
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering},
};

// --- Global Runtime States ---

/// Indicates if the core engine is currently active.
pub static IS_CORE_RUNNING: AtomicBool = AtomicBool::new(false);
/// Indicates if the input listener thread is active.
pub static IS_LISTEN_RUNNING: AtomicBool = AtomicBool::new(false);
/// Indicates if the input grabber (interceptor) is active.
pub static IS_GRAB_RUNNING: AtomicBool = AtomicBool::new(false);

/// Stores the global window handle (HWND) for reference across threads.
pub static GLOBAL_HWND: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

// --- Listen Flags: Define which events to monitor ---

pub static LISTEN_FLAG: AtomicU32 = AtomicU32::new(0);
pub const LISTEN_MOUSE_MOVE: u32 = 1 << 0;
pub const LISTEN_MOUSE_BUTTON: u32 = 1 << 1;
pub const LISTEN_MOUSE_WHEEL: u32 = 1 << 2;
pub const LISTEN_KEYBOARD: u32 = 1 << 3;
#[rustfmt::skip]
pub const LISTENS_ALL: u32 = LISTEN_MOUSE_MOVE | LISTEN_MOUSE_BUTTON | LISTEN_MOUSE_WHEEL | LISTEN_KEYBOARD;

// --- Grab Flags: Define which events to intercept/block ---

pub static GRAB_FLAG: AtomicU32 = AtomicU32::new(0);
pub const GRAB_MOUSE_MOVE: u32 = 1 << 0;    // 0x01
pub const GRAB_MOUSE_BUTTON: u32 = 1 << 1;  // 0x02
pub const GRAB_MOUSE_WHEEL: u32 = 1 << 2;   // 0x04
pub const GRAB_KEYBOARD: u32 = 1 << 3;      // 0x08
pub const GRAB_ALL: u32 = GRAB_MOUSE_MOVE | GRAB_MOUSE_BUTTON | GRAB_MOUSE_WHEEL | GRAB_KEYBOARD;

/// Updates an atomic bitmask in a thread-safe manner using Compare-And-Swap (CAS).
///
/// # Arguments
/// * `atomic` - The atomic U32 bitmask to modify.
/// * `bit` - The specific bit(s) to set or clear.
/// * `enable` - True to set the bit (OR), false to clear the bit (AND NOT).
pub fn update_state(atomic: &AtomicU32, bit: u32, enable: bool) {
    let mut current = atomic.load(Ordering::SeqCst);
    loop {
        let next = if enable {
            current | bit // Set the bit to 1
        } else {
            current & !bit // Set the bit to 0
        };

        // Attempt to swap the value if it hasn't changed since we loaded it
        match atomic.compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => break,
            Err(actual) => current = actual, // Update current value and retry
        }
    }
}

pub mod utils {
    use windows::Win32::UI::{
        Input::KeyboardAndMouse::VK_PACKET,
        WindowsAndMessaging::{KBDLLHOOKSTRUCT, KBDLLHOOKSTRUCT_FLAGS},
    };

    // Low-level macros ported to Rust for extracting bytes/words from Windows messages.

    #[inline]
    pub fn hiword(l: u32) -> u16 {
        ((l >> 16) & 0xffff) as u16
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn loword(l: u32) -> u16 {
        (l & 0xffff) as u16
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn hibyte(w: u16) -> u8 {
        ((w >> 8) & 0xff) as u8
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn lobyte(w: u16) -> u8 {
        (w & 0xff) as u8
    }

    /// Extracts the virtual key code or unicode packet from the keyboard hook structure.
    pub(crate) fn get_code(kb: &KBDLLHOOKSTRUCT) -> u32 {
        // If it's a VK_PACKET (used for injecting unicode characters via SendInput),
        // the scan code contains the unicode character.
        // Reference: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes#:~:text=OEM%20specific-,VK_PACKET,-0xE7
        if kb.vkCode == VK_PACKET.0 as u32 {
            kb.scanCode
        } else {
            kb.vkCode
        }
    }

    /// Reconstructs the full scan code, including extended key prefixes (0xE0).
    #[allow(dead_code)]
    pub(crate) fn get_scan_code(kb: &KBDLLHOOKSTRUCT) -> u32 {
        // The right-hand SHIFT, NumLock, and some other keys are handled specifically.
        // Reference: https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input#:~:text=The%20right%2Dhand%20SHIFT%20key%20is%20not%20considered%20an%20extended%2Dkey%2C%20it%20has%20a%20separate%20scan%20code%20instead.
        match kb.scanCode {
            0x36 | 0x45 => kb.scanCode,
            _ => {
                // Check if the LLKHF_EXTENDED (0x01) flag is set.
                // If it is an extended key, we prefix the scan code with 0xE0.
                if (kb.flags & KBDLLHOOKSTRUCT_FLAGS(0x01)) == KBDLLHOOKSTRUCT_FLAGS(0x01) {
                    0xE0 << 8 | kb.scanCode
                } else {
                    kb.scanCode
                }
            }
        }
    }
}
