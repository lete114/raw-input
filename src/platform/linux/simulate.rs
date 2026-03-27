use std::fs::{File, OpenOptions};
use std::io::Write;
use std::mem;
use std::os::fd::AsRawFd;
use std::sync::Mutex;

use crate::platform::linux::keycode::key_to_code;
use crate::platform::{PlatformSimulate, SimulateImpl};
use crate::{Event, Key, MouseButton};

const UINPUT_PATH: &str = "/dev/uinput";
const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_REL: u16 = 0x02;
const SYN_REPORT: u16 = 0;
const REL_X: u16 = 0x00;
const REL_Y: u16 = 0x01;
const REL_WHEEL: u16 = 0x08;
const REL_HWHEEL: u16 = 0x06;
const BTN_LEFT: u16 = 0x110;
const BTN_RIGHT: u16 = 0x111;
const BTN_MIDDLE: u16 = 0x112;
const BTN_SIDE: u16 = 0x113;
const BTN_EXTRA: u16 = 0x114;

#[repr(C)]
struct InputEvent {
    time: libc::timeval,
    type_: u16,
    code: u16,
    value: i32,
}

#[repr(C)]
struct UinputSetup {
    id: InputId,
    name: [u8; 80],
    ff_effects_max: u32,
}

#[repr(C)]
struct InputId {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

const UI_DEV_SETUP: libc::c_ulong = 0x405c5503;
const UI_DEV_CREATE: libc::c_ulong = 0x5501;
const UI_SET_EVBIT: libc::c_ulong = 0x40045564;
const UI_SET_KEYBIT: libc::c_ulong = 0x40045565;
const UI_SET_RELBIT: libc::c_ulong = 0x40045566;

static UINPUT_FD: Mutex<Option<File>> = Mutex::new(None);

fn ensure_device() -> bool {
    let mut guard = UINPUT_FD.lock().unwrap();
    if guard.is_some() {
        return true;
    }

    let file = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(UINPUT_PATH)
    {
        Ok(f) => f,
        Err(_) => return false,
    };

    let fd = file.as_raw_fd();

    unsafe {
        if libc::ioctl(fd, UI_SET_EVBIT, EV_KEY as libc::c_int) < 0
            || libc::ioctl(fd, UI_SET_EVBIT, EV_REL as libc::c_int) < 0
            || libc::ioctl(fd, UI_SET_EVBIT, EV_SYN as libc::c_int) < 0
        {
            return false;
        }

        for key in 1..256 {
            if libc::ioctl(fd, UI_SET_KEYBIT, key as libc::c_int) < 0 {
                return false;
            }
        }
        if libc::ioctl(fd, UI_SET_KEYBIT, BTN_LEFT as libc::c_int) < 0
            || libc::ioctl(fd, UI_SET_KEYBIT, BTN_RIGHT as libc::c_int) < 0
            || libc::ioctl(fd, UI_SET_KEYBIT, BTN_MIDDLE as libc::c_int) < 0
            || libc::ioctl(fd, UI_SET_KEYBIT, BTN_SIDE as libc::c_int) < 0
            || libc::ioctl(fd, UI_SET_KEYBIT, BTN_EXTRA as libc::c_int) < 0
        {
            return false;
        }

        if libc::ioctl(fd, UI_SET_RELBIT, REL_X as libc::c_int) < 0
            || libc::ioctl(fd, UI_SET_RELBIT, REL_Y as libc::c_int) < 0
            || libc::ioctl(fd, UI_SET_RELBIT, REL_WHEEL as libc::c_int) < 0
            || libc::ioctl(fd, UI_SET_RELBIT, REL_HWHEEL as libc::c_int) < 0
        {
            return false;
        }

        let mut setup: UinputSetup = mem::zeroed();
        setup.id.bustype = 0x03;
        setup.id.vendor = 0x1234;
        setup.id.product = 0x5678;
        setup.id.version = 1;
        let name = b"raw-input-virtual";
        setup.name[..name.len()].copy_from_slice(name);

        if libc::ioctl(fd, UI_DEV_SETUP, &setup) < 0 {
            return false;
        }
        if libc::ioctl(fd, UI_DEV_CREATE) < 0 {
            return false;
        }
    }

    *guard = Some(file);
    true
}

fn write_event(type_: u16, code: u16, value: i32) {
    let mut guard = UINPUT_FD.lock().unwrap();
    let Some(file) = guard.as_mut() else {
        return;
    };

    let event = InputEvent {
        time: libc::timeval { tv_sec: 0, tv_usec: 0 },
        type_,
        code,
        value,
    };

    let bytes = unsafe {
        std::slice::from_raw_parts(
            &event as *const InputEvent as *const u8,
            mem::size_of::<InputEvent>(),
        )
    };

    let _ = file.write_all(bytes);
}

fn syn() {
    write_event(EV_SYN, SYN_REPORT, 0);
}

impl SimulateImpl for PlatformSimulate {
    fn simulate(event: Event) {
        match event {
            Event::MouseMove { delta, .. } => Self::mouse_move(delta.x, delta.y),
            Event::MouseWheel { delta, .. } => Self::mouse_wheel(delta.x, delta.y),
            Event::MouseDown { button, .. } => Self::mouse_button(button, true),
            Event::MouseUp { button, .. } => Self::mouse_button(button, false),
            Event::KeyDown { key, .. } => Self::keyboard(key, true),
            Event::KeyUp { key, .. } => Self::keyboard(key, false),
        }
    }

    fn mouse_move(dx: f64, dy: f64) {
        if !ensure_device() {
            return;
        }
        if dx != 0.0 {
            write_event(EV_REL, REL_X, dx as i32);
        }
        if dy != 0.0 {
            write_event(EV_REL, REL_Y, dy as i32);
        }
        syn();
    }

    fn mouse_move_to(_x: f64, _y: f64) {
        // Absolute positioning requires uinput ABS events with proper setup
        // For simplicity, relative movement is preferred
    }

    fn mouse_wheel(dx: f64, dy: f64) {
        if !ensure_device() {
            return;
        }
        if dy != 0.0 {
            write_event(EV_REL, REL_WHEEL, dy as i32);
        }
        if dx != 0.0 {
            write_event(EV_REL, REL_HWHEEL, dx as i32);
        }
        syn();
    }

    fn mouse_button(button: MouseButton, down: bool) {
        if !ensure_device() {
            return;
        }
        let code = match button {
            MouseButton::Left => BTN_LEFT,
            MouseButton::Right => BTN_RIGHT,
            MouseButton::Middle => BTN_MIDDLE,
            MouseButton::Back => BTN_SIDE,
            MouseButton::Forward => BTN_EXTRA,
        };
        write_event(EV_KEY, code, if down { 1 } else { 0 });
        syn();
    }

    fn keyboard(key: Key, down: bool) {
        if !ensure_device() {
            return;
        }
        let Some(code) = key_to_code(key) else {
            return;
        };
        write_event(EV_KEY, code as u16, if down { 1 } else { 0 });
        syn();
    }
}
