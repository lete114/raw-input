use std::{ffi::c_void, mem::size_of, sync::atomic::Ordering};

use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    UI::{
        Input::{
            GetRawInputData, HRAWINPUT, MOUSE_MOVE_ABSOLUTE, RAWINPUT, RAWINPUTHEADER, RID_INPUT,
            RIM_TYPEMOUSE,
        },
        WindowsAndMessaging::{
            KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT, WHEEL_DELTA, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN,
            WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEWHEEL,
            WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
            XBUTTON1, XBUTTON2,
        },
    },
};

use crate::{
    Listen,
    dispatcher::{CALLBACKS, NEXT_ID, Status, Subscriber, dispatch, remove_all},
    event::{Event, MouseButton, Point},
    key::KeyCode,
    platform::windows::{
        common::{
            IS_LISTEN_RUNNING, LISTEN_FLAG, LISTEN_KEYBOARD, LISTEN_MOUSE_BUTTON,
            LISTEN_MOUSE_MOVE, LISTEN_MOUSE_WHEEL, LISTENS_ALL, update_state, utils,
        },
        keycode::code_to_key,
    },
    subscription::SubscriptionHandle,
};

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

    pub(crate) fn handle(wparam: WPARAM, lparam: LPARAM) {
        if !IS_LISTEN_RUNNING.load(Ordering::Relaxed) {
            return;
        }

        let state = LISTEN_FLAG.load(Ordering::Relaxed);
        if state == 0 {
            return;
        }

        let msg = wparam.0 as u32;

        let event = match msg {
            // ================= Mouse Buttons & Wheel =================
            WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN
            | WM_MBUTTONUP | WM_XBUTTONDOWN | WM_XBUTTONUP | WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                let is_wheel = msg == WM_MOUSEWHEEL || msg == WM_MOUSEHWHEEL;
                if is_wheel {
                    if (state & LISTEN_MOUSE_WHEEL) == 0 {
                        return;
                    }
                } else {
                    if (state & LISTEN_MOUSE_BUTTON) == 0 {
                        return;
                    }
                }

                // Cast LPARAM to Low-Level Mouse Hook structure
                let mouse = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
                // Extract high-order word for wheel delta or X-button index
                let delta = utils::hiword(mouse.mouseData);

                match msg {
                    WM_LBUTTONDOWN => Event::MouseDown {
                        button: MouseButton::Left,
                    },
                    WM_LBUTTONUP => Event::MouseUp {
                        button: MouseButton::Left,
                    },
                    WM_RBUTTONDOWN => Event::MouseDown {
                        button: MouseButton::Right,
                    },
                    WM_RBUTTONUP => Event::MouseUp {
                        button: MouseButton::Right,
                    },
                    WM_MBUTTONDOWN => Event::MouseDown {
                        button: MouseButton::Middle,
                    },
                    WM_MBUTTONUP => Event::MouseUp {
                        button: MouseButton::Middle,
                    },

                    WM_MOUSEWHEEL => {
                        // Normalize vertical wheel delta
                        let y = delta as i16 as f64 / WHEEL_DELTA as f64;
                        Event::MouseWheel {
                            delta: Point { x: 0.0, y },
                        }
                    }
                    WM_MOUSEHWHEEL => {
                        // Normalize horizontal wheel delta
                        let x = delta as i16 as f64 / WHEEL_DELTA as f64;
                        Event::MouseWheel {
                            delta: Point { x, y: 0.0 },
                        }
                    }

                    WM_XBUTTONDOWN | WM_XBUTTONUP => {
                        // Distinguish between XBUTTON1 (Back) and XBUTTON2 (Forward)
                        let button = match delta {
                            XBUTTON1 => MouseButton::Back,
                            XBUTTON2 => MouseButton::Forward,
                            _ => return,
                        };
                        if msg == WM_XBUTTONDOWN {
                            Event::MouseDown { button }
                        } else {
                            Event::MouseUp { button }
                        }
                    }
                    _ => return,
                }
            }

            // ================= Keyboard =================
            WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => {
                if (state & LISTEN_KEYBOARD) == 0 {
                    return;
                }

                // Cast LPARAM to Low-Level Keyboard Hook structure
                let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
                let code: KeyCode = utils::get_code(kb);
                let key = code_to_key(code.into());
                let code = Some(code);

                if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                    Event::KeyDown { key, code }
                } else {
                    Event::KeyUp { key, code }
                }
            }
            _ => return,
        };

        dispatch(event);
    }

    pub(crate) fn handle_mouse_move(lparam: LPARAM) -> bool {
        if !IS_LISTEN_RUNNING.load(Ordering::Relaxed) {
            return false;
        }

        let state = LISTEN_FLAG.load(Ordering::Relaxed);
        if state & LISTEN_MOUSE_MOVE == 0 {
            return false;
        }

        let h_raw_input = HRAWINPUT(lparam.0 as *mut c_void);
        let mut raw = RAWINPUT::default();
        let mut size = std::mem::size_of::<RAWINPUT>() as u32;

        // Retrieve Raw Input data from the message LPARAM
        let raw_size = unsafe {
            GetRawInputData(
                h_raw_input,
                RID_INPUT,
                Some(&mut raw as *mut _ as *mut _),
                &mut size,
                size_of::<RAWINPUTHEADER>() as u32,
            )
        };

        if raw_size == u32::MAX {
            return false;
        }

        // Ensure the input type is mouse
        if raw.header.dwType != RIM_TYPEMOUSE.0 {
            return false;
        }

        let mouse = unsafe { &raw.data.mouse };

        // Filter out absolute movement events to keep only relative deltas
        if mouse.usFlags.0 & MOUSE_MOVE_ABSOLUTE.0 != 0 {
            return true;
        }

        let dx = mouse.lLastX as f64;
        let dy = mouse.lLastY as f64;

        if dx != 0.0 || dy != 0.0 {
            dispatch(Event::MouseMove {
                delta: Point { x: dx, y: dy },
            });
        }

        true
    }
}
