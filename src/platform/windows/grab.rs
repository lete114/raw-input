use std::{
    ffi::c_void,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use windows::Win32::UI::WindowsAndMessaging::{
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
};

use crate::{
    Grab,
    platform::windows::common::{
        GRAB_ALL, GRAB_FLAG, GRAB_KEYBOARD, GRAB_MOUSE_BUTTON, GRAB_MOUSE_MOVE, GRAB_MOUSE_WHEEL,
        IS_GRAB_RUNNING, update_state,
    },
};

pub static MOUSE_HOOK: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
pub static KEYBOARD_HOOK: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

impl Grab {
    pub fn start() {
        if Self::is_run() {
            return;
        }

        GRAB_FLAG.fetch_or(GRAB_ALL, Ordering::SeqCst);
    }

    pub fn is_runing() -> bool {
        IS_GRAB_RUNNING.load(Ordering::SeqCst)
    }

    pub fn pause() {
        IS_GRAB_RUNNING.store(false, Ordering::SeqCst);
    }

    pub fn resume() {
        IS_GRAB_RUNNING.store(true, Ordering::SeqCst);
    }

    pub fn stop() {
        Self::pause();
        GRAB_FLAG.store(0, Ordering::SeqCst);
    }

    pub fn mouse_move(enable: bool) {
        update_state(&GRAB_FLAG, GRAB_MOUSE_MOVE, enable);
    }

    pub fn mouse_wheel(enable: bool) {
        update_state(&GRAB_FLAG, GRAB_MOUSE_WHEEL, enable);
    }

    pub fn mouse_button(enable: bool) {
        update_state(&GRAB_FLAG, GRAB_MOUSE_BUTTON, enable);
    }

    pub fn keyboard(enable: bool) {
        update_state(&GRAB_FLAG, GRAB_KEYBOARD, enable);
    }
}

impl Grab {
    #[inline]
    fn is_run() -> bool {
        IS_GRAB_RUNNING
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
    }

    #[inline]
    pub(crate) fn should_block(msg: u32) -> bool {
        let state = GRAB_FLAG.load(Ordering::Relaxed);
        if state == 0 {
            return false;
        }

        match msg {
            // mouse move
            WM_MOUSEMOVE => (state & GRAB_MOUSE_MOVE) != 0,

            // mouse button
            WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN
            | WM_MBUTTONUP | WM_XBUTTONDOWN | WM_XBUTTONUP => (state & GRAB_MOUSE_BUTTON) != 0,

            // mouse wheel
            WM_MOUSEWHEEL | WM_MOUSEHWHEEL => (state & GRAB_MOUSE_WHEEL) != 0,

            // keyboard
            WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => (state & GRAB_KEYBOARD) != 0,
            _ => false,
        }
    }
}
