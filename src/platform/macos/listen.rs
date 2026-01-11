use std::sync::atomic::{AtomicU64, Ordering};

use core_graphics::event::{CGEvent, CGEventField, CGEventType, EventField};

use crate::{
    Listen,
    dispatcher::{CALLBACKS, NEXT_ID, Status, Subscriber, dispatch, remove_all},
    event::{Event, FloatPoint, KeyCode, MouseButton, Point},
    keycodes::macos::key_from_code,
    platform::macos::common::{
        IS_LISTEN_RUNNING, LISTEN_FLAG, LISTEN_KEYBOARD, LISTEN_MOUSE_BUTTON, LISTEN_MOUSE_MOVE,
        LISTEN_MOUSE_WHEEL, LISTENS_ALL, update_state,
    },
    subscription::SubscriptionHandle,
};

static LAST_FLAGS: AtomicU64 = AtomicU64::new(0);

impl Listen {
    pub fn start() {
        if Self::is_run() {
            return;
        }

        LISTEN_FLAG.store(LISTENS_ALL, Ordering::SeqCst);
    }

    pub fn is_runing() -> bool {
        IS_LISTEN_RUNNING.load(Ordering::SeqCst)
    }

    pub fn pause() {
        IS_LISTEN_RUNNING.store(false, Ordering::SeqCst);
    }

    pub fn resume() {
        IS_LISTEN_RUNNING.store(true, Ordering::SeqCst);
    }

    pub fn stop() {
        LISTEN_FLAG.store(0, Ordering::SeqCst);
        Self::pause();
        Self::unsubscribe_all();
    }

    pub fn mouse_move(enable: bool) {
        update_state(&LISTEN_FLAG, LISTEN_MOUSE_MOVE, enable);
    }

    pub fn mouse_wheel(enable: bool) {
        update_state(&LISTEN_FLAG, LISTEN_MOUSE_WHEEL, enable);
    }

    pub fn mouse_button(enable: bool) {
        update_state(&LISTEN_FLAG, LISTEN_MOUSE_BUTTON, enable);
    }

    pub fn keyboard(enable: bool) {
        update_state(&LISTEN_FLAG, LISTEN_KEYBOARD, enable);
    }

    pub fn subscribe<F>(callback: F) -> SubscriptionHandle
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

    pub fn unsubscribe_all() {
        remove_all();
    }
}

impl Listen {
    fn is_run() -> bool {
        IS_LISTEN_RUNNING
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
    }

    fn get_code(event: &CGEvent, event_field: CGEventField) -> i64 {
        event.get_integer_value_field(event_field)
    }

    pub(crate) fn handle(event_type: CGEventType, event: &CGEvent) {
        if !IS_LISTEN_RUNNING.load(Ordering::Relaxed) {
            return;
        }

        let state = LISTEN_FLAG.load(Ordering::Relaxed);
        if state == 0 {
            return;
        }

        let event = match event_type {
            CGEventType::MouseMoved
            | CGEventType::LeftMouseDragged
            | CGEventType::RightMouseDragged
            | CGEventType::OtherMouseDragged => {
                if state & LISTEN_MOUSE_MOVE == 0 {
                    return;
                }
                let dx = Self::get_code(event, EventField::MOUSE_EVENT_DELTA_X);
                let dy = Self::get_code(event, EventField::MOUSE_EVENT_DELTA_Y);

                if dx != 0 || dy != 0 {
                    Event::MouseMove {
                        delta: Point {
                            x: dx as i32,
                            y: dy as i32,
                        },
                    }
                } else {
                    return;
                }
            }
            CGEventType::LeftMouseDown
            | CGEventType::LeftMouseUp
            | CGEventType::RightMouseDown
            | CGEventType::RightMouseUp => {
                if state & LISTEN_MOUSE_BUTTON == 0 {
                    return;
                }
                let match_type = matches!(
                    event_type,
                    CGEventType::LeftMouseDown | CGEventType::LeftMouseUp
                );
                let button = if match_type {
                    MouseButton::Left
                } else {
                    MouseButton::Right
                };

                let match_type = matches!(
                    event_type,
                    CGEventType::LeftMouseDown | CGEventType::RightMouseDown
                );
                if match_type {
                    Event::MouseDown { button }
                } else {
                    Event::MouseUp { button }
                }
            }
            CGEventType::OtherMouseDown | CGEventType::OtherMouseUp => {
                if state & LISTEN_MOUSE_BUTTON == 0 {
                    return;
                }
                let num = Self::get_code(event, EventField::MOUSE_EVENT_BUTTON_NUMBER);
                let button = match num {
                    2 => MouseButton::Middle,
                    3 => MouseButton::Back,
                    4 => MouseButton::Forward,
                    _ => return,
                };

                match event_type {
                    CGEventType::OtherMouseDown => Event::MouseDown { button },
                    _ => Event::MouseUp { button },
                }
            }
            CGEventType::ScrollWheel => {
                if state & LISTEN_MOUSE_WHEEL == 0 {
                    return;
                }
                let dy = Self::get_code(event, EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_1);
                let dx = Self::get_code(event, EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_2);

                Event::MouseWheel {
                    delta: FloatPoint {
                        x: dx as f64,
                        y: dy as f64,
                    },
                }
            }
            CGEventType::KeyDown | CGEventType::KeyUp => {
                if state & LISTEN_KEYBOARD == 0 {
                    return;
                }
                let code = Self::get_code(event, EventField::KEYBOARD_EVENT_KEYCODE);
                let key = key_from_code(code as KeyCode);

                match event_type {
                    CGEventType::KeyDown => Event::KeyDown { key },
                    _ => Event::KeyUp { key },
                }
            }
            CGEventType::FlagsChanged => {
                if state & LISTEN_KEYBOARD == 0 {
                    return;
                }

                let new_flags = event.get_flags().bits();
                let old_flags = LAST_FLAGS.swap(new_flags, Ordering::SeqCst);
                let changed_bit = new_flags ^ old_flags;
                if changed_bit == 0 {
                    return;
                }

                let code = Self::get_code(event, EventField::KEYBOARD_EVENT_KEYCODE);
                let key = key_from_code(code as KeyCode);

                if new_flags & changed_bit != 0 {
                    Event::KeyDown { key }
                } else {
                    Event::KeyUp { key }
                }
            }
            _ => return,
        };

        dispatch(event);
    }
}
