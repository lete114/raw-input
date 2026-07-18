use std::sync::atomic::Ordering;

use core_graphics::event::CGEventType;

use crate::platform::{
    PlatformGrab, GrabImpl,
    macos::common::{
        GRAB_ALL, GRAB_FLAG, GRAB_KEYBOARD, GRAB_MOUSE_BUTTON, GRAB_MOUSE_MOVE, GRAB_MOUSE_WHEEL,
        IS_GRAB_RUNNING, update_state,
    },
};

impl GrabImpl for PlatformGrab {
    fn start() {
        if Self::is_run() {
            return;
        }
        GRAB_FLAG.fetch_or(GRAB_ALL, Ordering::SeqCst);
    }

    fn is_runing() -> bool {
        IS_GRAB_RUNNING.load(Ordering::SeqCst)
    }

    fn pause() {
        IS_GRAB_RUNNING.store(false, Ordering::SeqCst);
    }

    fn resume() {
        IS_GRAB_RUNNING.store(true, Ordering::SeqCst);
    }

    fn stop() {
        Self::pause();
        GRAB_FLAG.store(0, Ordering::SeqCst);
    }

    fn mouse_move(enable: bool) {
        update_state(&GRAB_FLAG, GRAB_MOUSE_MOVE, enable);
    }

    fn mouse_wheel(enable: bool) {
        update_state(&GRAB_FLAG, GRAB_MOUSE_WHEEL, enable);
    }

    fn mouse_button(enable: bool) {
        update_state(&GRAB_FLAG, GRAB_MOUSE_BUTTON, enable);
    }

    fn keyboard(enable: bool) {
        update_state(&GRAB_FLAG, GRAB_KEYBOARD, enable);
    }
}

impl PlatformGrab {
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

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    fn set_state(running: bool, flags: u32) {
        IS_GRAB_RUNNING.store(running, Ordering::SeqCst);
        GRAB_FLAG.store(flags, Ordering::SeqCst);
    }

    #[serial]
    #[test]
    fn test_should_block_returns_false_when_not_running() {
        set_state(false, GRAB_ALL);
        assert!(!PlatformGrab::should_block(CGEventType::KeyDown));
        assert!(!PlatformGrab::should_block(CGEventType::MouseMoved));
        assert!(!PlatformGrab::should_block(CGEventType::LeftMouseDown));
    }

    #[serial]
    #[test]
    fn test_should_block_returns_false_when_no_flags() {
        set_state(true, 0);
        assert!(!PlatformGrab::should_block(CGEventType::KeyDown));
        assert!(!PlatformGrab::should_block(CGEventType::MouseMoved));
    }

    #[serial]
    #[test]
    fn test_should_block_keyboard_events() {
        set_state(true, GRAB_KEYBOARD);
        assert!(PlatformGrab::should_block(CGEventType::KeyDown));
        assert!(PlatformGrab::should_block(CGEventType::KeyUp));
        assert!(PlatformGrab::should_block(CGEventType::FlagsChanged));
        assert!(!PlatformGrab::should_block(CGEventType::MouseMoved));
    }

    #[serial]
    #[test]
    fn test_should_block_mouse_move_events() {
        set_state(true, GRAB_MOUSE_MOVE);
        assert!(PlatformGrab::should_block(CGEventType::MouseMoved));
        assert!(PlatformGrab::should_block(CGEventType::LeftMouseDragged));
        assert!(PlatformGrab::should_block(CGEventType::RightMouseDragged));
        assert!(PlatformGrab::should_block(CGEventType::OtherMouseDragged));
        assert!(!PlatformGrab::should_block(CGEventType::KeyDown));
    }

    #[serial]
    #[test]
    fn test_should_block_mouse_button_events() {
        set_state(true, GRAB_MOUSE_BUTTON);
        assert!(PlatformGrab::should_block(CGEventType::LeftMouseDown));
        assert!(PlatformGrab::should_block(CGEventType::LeftMouseUp));
        assert!(PlatformGrab::should_block(CGEventType::RightMouseDown));
        assert!(PlatformGrab::should_block(CGEventType::RightMouseUp));
        assert!(PlatformGrab::should_block(CGEventType::OtherMouseDown));
        assert!(PlatformGrab::should_block(CGEventType::OtherMouseUp));
        assert!(!PlatformGrab::should_block(CGEventType::MouseMoved));
    }

    #[serial]
    #[test]
    fn test_should_block_scroll_wheel() {
        set_state(true, GRAB_MOUSE_WHEEL);
        assert!(PlatformGrab::should_block(CGEventType::ScrollWheel));
        assert!(!PlatformGrab::should_block(CGEventType::KeyDown));
    }

    #[serial]
    #[test]
    fn test_should_block_all_when_all_flags_set() {
        set_state(true, GRAB_ALL);
        assert!(PlatformGrab::should_block(CGEventType::KeyDown));
        assert!(PlatformGrab::should_block(CGEventType::MouseMoved));
        assert!(PlatformGrab::should_block(CGEventType::LeftMouseDown));
        assert!(PlatformGrab::should_block(CGEventType::ScrollWheel));
    }

    #[serial]
    #[test]
    fn test_should_block_unknown_event_type() {
        set_state(true, GRAB_ALL);
        let unknown = CGEventType::Null;
        assert!(!PlatformGrab::should_block(unknown));
    }
}
