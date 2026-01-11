use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use core_graphics::event::CGEventType;

// --- Global Runtime States ---

/// Indicates if the core engine is currently active.
pub static IS_CORE_RUNNING: AtomicBool = AtomicBool::new(false);
/// Indicates if the input listener thread is active.
pub static IS_LISTEN_RUNNING: AtomicBool = AtomicBool::new(false);
/// Indicates if the input grabber (interceptor) is active.
pub static IS_GRAB_RUNNING: AtomicBool = AtomicBool::new(false);

pub const INTERESTED_EVENTS: &[CGEventType] = &[
    CGEventType::MouseMoved,
    CGEventType::LeftMouseDown,
    CGEventType::LeftMouseUp,
    CGEventType::RightMouseDown,
    CGEventType::RightMouseUp,
    CGEventType::ScrollWheel,
    CGEventType::OtherMouseDown,
    CGEventType::OtherMouseUp,
    CGEventType::LeftMouseDragged,
    CGEventType::RightMouseDragged,
    CGEventType::OtherMouseDragged,
    CGEventType::KeyDown,
    CGEventType::KeyUp,
    CGEventType::FlagsChanged,
];

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
pub const GRAB_MOUSE_MOVE: u32 = 1 << 0; // 0x01
pub const GRAB_MOUSE_BUTTON: u32 = 1 << 1; // 0x02
pub const GRAB_MOUSE_WHEEL: u32 = 1 << 2; // 0x04
pub const GRAB_KEYBOARD: u32 = 1 << 3; // 0x08
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
