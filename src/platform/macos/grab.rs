use std::sync::atomic::Ordering;

use core_graphics::event::CGEventType;

use crate::{
    Grab,
    platform::macos::common::{
        GRAB_ALL, GRAB_FLAG, GRAB_KEYBOARD, GRAB_MOUSE_BUTTON, GRAB_MOUSE_MOVE, GRAB_MOUSE_WHEEL,
        IS_GRAB_RUNNING, update_state,
    },
};

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
    pub(crate) fn should_block(event_type: CGEventType) -> bool {
        if !IS_GRAB_RUNNING.load(Ordering::Relaxed) {
            return false;
        }

        let state = GRAB_FLAG.load(Ordering::Relaxed);
        if state == 0 {
            return false;
        }

        match event_type {
            // Mouse move & Dragging
            CGEventType::MouseMoved
            | CGEventType::LeftMouseDragged
            | CGEventType::RightMouseDragged
            | CGEventType::OtherMouseDragged => (state & GRAB_MOUSE_MOVE) != 0,

            // Mouse buttons (Left, Right, Middle/Other)
            CGEventType::LeftMouseDown
            | CGEventType::LeftMouseUp
            | CGEventType::RightMouseDown
            | CGEventType::RightMouseUp
            | CGEventType::OtherMouseDown
            | CGEventType::OtherMouseUp => (state & GRAB_MOUSE_BUTTON) != 0,

            // Mouse wheel
            CGEventType::ScrollWheel => (state & GRAB_MOUSE_WHEEL) != 0,

            // Keyboard (Normal keys & Modifier keys)
            CGEventType::KeyDown | CGEventType::KeyUp | CGEventType::FlagsChanged => {
                (state & GRAB_KEYBOARD) != 0
            }
            _ => false,
        }
    }
}
