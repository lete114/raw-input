use std::sync::atomic::Ordering;

use evdev::{InputEventKind, Key, RelativeAxisType};

use crate::{
    dispatcher::{CALLBACKS, NEXT_ID, Status, Subscriber, dispatch, remove_all},
    event::{Event, MouseButton, Point},
    key::KeyCode,
    platform::{
        ListenImpl, PlatformListen,
        linux::{
            common::{
                IS_LISTEN_RUNNING, LISTEN_FLAG, LISTEN_KEYBOARD, LISTEN_MOUSE_BUTTON,
                LISTEN_MOUSE_MOVE, LISTEN_MOUSE_WHEEL, LISTENS_ALL, update_state,
            },
            keycode::code_to_key,
        },
    },
    subscription::SubscriptionHandle,
};

impl ListenImpl for PlatformListen {
    fn start() {
        if Self::is_run() {
            return;
        }
        LISTEN_FLAG.store(LISTENS_ALL, Ordering::SeqCst);
    }

    fn is_runing() -> bool {
        IS_LISTEN_RUNNING.load(Ordering::SeqCst)
    }

    fn pause() {
        IS_LISTEN_RUNNING.store(false, Ordering::SeqCst);
    }

    fn resume() {
        IS_LISTEN_RUNNING.store(true, Ordering::SeqCst);
    }

    fn stop() {
        LISTEN_FLAG.store(0, Ordering::SeqCst);
        Self::pause();
        Self::unsubscribe_all();
    }

    fn mouse_move(enable: bool) {
        update_state(&LISTEN_FLAG, LISTEN_MOUSE_MOVE, enable);
    }

    fn mouse_wheel(enable: bool) {
        update_state(&LISTEN_FLAG, LISTEN_MOUSE_WHEEL, enable);
    }

    fn mouse_button(enable: bool) {
        update_state(&LISTEN_FLAG, LISTEN_MOUSE_BUTTON, enable);
    }

    fn keyboard(enable: bool) {
        update_state(&LISTEN_FLAG, LISTEN_KEYBOARD, enable);
    }

    fn subscribe<F>(callback: F) -> SubscriptionHandle
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        CALLBACKS.insert(
            id,
            Subscriber {
                status: Status::Active,
                callback: Box::new(callback),
            },
        );
        SubscriptionHandle { id }
    }

    fn unsubscribe_all() {
        remove_all();
    }
}

impl PlatformListen {
    fn is_run() -> bool {
        IS_LISTEN_RUNNING
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
    }

    pub(crate) fn handle_key(event: &evdev::InputEvent) {
        if !IS_LISTEN_RUNNING.load(Ordering::Relaxed) {
            return;
        }

        let state = LISTEN_FLAG.load(Ordering::Relaxed);
        if state == 0 {
            return;
        }

        let InputEventKind::Key(key) = event.kind() else {
            return;
        };

        let value = event.value();
        if value == 2 {
            return;
        }

        let btn = match key {
            Key::BTN_LEFT => Some(MouseButton::Left),
            Key::BTN_RIGHT => Some(MouseButton::Right),
            Key::BTN_MIDDLE => Some(MouseButton::Middle),
            Key::BTN_SIDE => Some(MouseButton::Back),
            Key::BTN_EXTRA => Some(MouseButton::Forward),
            _ => None,
        };

        if let Some(button) = btn {
            if state & LISTEN_MOUSE_BUTTON == 0 {
                return;
            }
            let ev = if value == 1 {
                Event::MouseDown { button }
            } else {
                Event::MouseUp { button }
            };
            dispatch(ev);
        } else {
            if state & LISTEN_KEYBOARD == 0 {
                return;
            }
            let code = key.code() as KeyCode;
            let key = code_to_key(code);
            let ev = if value == 1 {
                Event::KeyDown { key, code: Some(code) }
            } else {
                Event::KeyUp { key, code: Some(code) }
            };
            dispatch(ev);
        }
    }

    pub(crate) fn handle_rel(event: &evdev::InputEvent) {
        if !IS_LISTEN_RUNNING.load(Ordering::Relaxed) {
            return;
        }

        let state = LISTEN_FLAG.load(Ordering::Relaxed);
        if state == 0 {
            return;
        }

        let InputEventKind::RelAxis(axis) = event.kind() else {
            return;
        };

        let value = event.value() as f64;

        match axis {
            RelativeAxisType::REL_X | RelativeAxisType::REL_Y => {
                if state & LISTEN_MOUSE_MOVE == 0 {
                    return;
                }
                let (dx, dy) = match axis {
                    RelativeAxisType::REL_X => (value, 0.0),
                    RelativeAxisType::REL_Y => (0.0, value),
                    _ => return,
                };
                dispatch(Event::MouseMove {
                    delta: Point { x: dx, y: dy },
                });
            }
            RelativeAxisType::REL_WHEEL | RelativeAxisType::REL_HWHEEL => {
                if state & LISTEN_MOUSE_WHEEL == 0 {
                    return;
                }
                let (dx, dy) = match axis {
                    RelativeAxisType::REL_WHEEL => (0.0, value),
                    RelativeAxisType::REL_HWHEEL => (value, 0.0),
                    _ => return,
                };
                dispatch(Event::MouseWheel {
                    delta: Point { x: dx, y: dy },
                });
            }
            _ => {}
        }
    }
}
