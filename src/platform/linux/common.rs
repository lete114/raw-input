use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub static IS_CORE_RUNNING: AtomicBool = AtomicBool::new(false);
pub static IS_LISTEN_RUNNING: AtomicBool = AtomicBool::new(false);
pub static IS_GRAB_RUNNING: AtomicBool = AtomicBool::new(false);

pub static LISTEN_FLAG: AtomicU32 = AtomicU32::new(0);
pub const LISTEN_MOUSE_MOVE: u32 = 1 << 0;
pub const LISTEN_MOUSE_BUTTON: u32 = 1 << 1;
pub const LISTEN_MOUSE_WHEEL: u32 = 1 << 2;
pub const LISTEN_KEYBOARD: u32 = 1 << 3;
#[rustfmt::skip]
pub const LISTENS_ALL: u32 = LISTEN_MOUSE_MOVE | LISTEN_MOUSE_BUTTON | LISTEN_MOUSE_WHEEL | LISTEN_KEYBOARD;

pub static GRAB_FLAG: AtomicU32 = AtomicU32::new(0);
pub const GRAB_MOUSE_MOVE: u32 = 1 << 0;
pub const GRAB_MOUSE_BUTTON: u32 = 1 << 1;
pub const GRAB_MOUSE_WHEEL: u32 = 1 << 2;
pub const GRAB_KEYBOARD: u32 = 1 << 3;
pub const GRAB_ALL: u32 = GRAB_MOUSE_MOVE | GRAB_MOUSE_BUTTON | GRAB_MOUSE_WHEEL | GRAB_KEYBOARD;

pub fn update_state(atomic: &AtomicU32, bit: u32, enable: bool) {
    let mut current = atomic.load(Ordering::SeqCst);
    loop {
        let next = if enable {
            current | bit
        } else {
            current & !bit
        };

        match atomic.compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => break,
            Err(actual) => current = actual,
        }
    }
}
