use std::sync::atomic::Ordering;

use evdev::{InputEventKind, Key, RelativeAxisType};

use crate::platform::{
    GrabImpl, PlatformGrab,
    linux::common::{
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

    pub(crate) fn handle_key(event: &evdev::InputEvent) {
        if !IS_GRAB_RUNNING.load(Ordering::Relaxed) {
            return;
        }

        let state = GRAB_FLAG.load(Ordering::Relaxed);
        if state == 0 {
            return;
        }

        let InputEventKind::Key(key) = event.kind() else {
            return;
        };

        let is_mouse_button = matches!(
            key,
            Key::BTN_LEFT | Key::BTN_RIGHT | Key::BTN_MIDDLE | Key::BTN_SIDE | Key::BTN_EXTRA
        );

        if is_mouse_button {
            if state & GRAB_MOUSE_BUTTON != 0 {
                // On Linux, true grab requires EVIOCGRAB ioctl
                // which is handled at device level, not per-event
            }
        } else if state & GRAB_KEYBOARD != 0 {
            // Same note about device-level grab
        }
    }

    pub(crate) fn handle_rel(event: &evdev::InputEvent) {
        if !IS_GRAB_RUNNING.load(Ordering::Relaxed) {
            return;
        }

        let state = GRAB_FLAG.load(Ordering::Relaxed);
        if state == 0 {
            return;
        }

        let InputEventKind::RelAxis(axis) = event.kind() else {
            return;
        };

        match axis {
            RelativeAxisType::REL_X | RelativeAxisType::REL_Y => {
                if state & GRAB_MOUSE_MOVE != 0 {
                    // Device-level grab required
                }
            }
            RelativeAxisType::REL_WHEEL | RelativeAxisType::REL_HWHEEL => {
                if state & GRAB_MOUSE_WHEEL != 0 {
                    // Device-level grab required
                }
            }
            _ => {}
        }
    }
}
