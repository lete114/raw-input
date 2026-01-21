use std::mem::size_of;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MOUSEEVENTF_ABSOLUTE,
    MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT,
    SendInput, VIRTUAL_KEY,
};

use crate::{
    Display, Event, Key, MouseButton, Simulate, platform::windows::keycode::get_win_codes,
};

impl Simulate {
    pub fn simulate(event: Event) {
        InputBuilder::new().add_event(event).send();
    }

    pub fn mouse_move(dx: f64, dy: f64) {
        InputBuilder::new().add_mouse_move(dx, dy).send();
    }

    pub fn mouse_move_to(x: f64, y: f64) {
        InputBuilder::new().add_mouse_move_to(x, y).send();
    }

    pub fn mouse_wheel(dx: f64, dy: f64) {
        InputBuilder::new().add_mouse_wheel(dx, dy).send();
    }

    pub fn mouse_button(button: MouseButton, down: bool) {
        InputBuilder::new().add_mouse_button(button, down).send();
    }

    pub fn keyboard(key: Key, down: bool) {
        InputBuilder::new().add_keyboard(key, down).send();
    }
}
struct InputBuilder {
    inputs: Vec<INPUT>,
}
impl InputBuilder {
    fn new() -> Self {
        Self { inputs: Vec::new() }
    }

    fn add_event(self, event: Event) -> Self {
        match event {
            Event::MouseMove { delta, .. } => self.add_mouse_move(delta.x, delta.y),
            Event::MouseWheel { delta, .. } => self.add_mouse_wheel(delta.x, delta.y),
            Event::MouseDown { button, .. } => self.add_mouse_button(button, true),
            Event::MouseUp { button, .. } => self.add_mouse_button(button, false),
            Event::KeyDown { key, .. } => self.add_keyboard(key, true),
            Event::KeyUp { key, .. } => self.add_keyboard(key, false),
        }
    }

    fn add_mouse_move(mut self, dx: f64, dy: f64) -> Self {
        self.push_mouse(MOUSEINPUT {
            dx: dx as i32,
            dy: dy as i32,
            dwFlags: MOUSEEVENTF_MOVE,
            ..Default::default()
        });
        self
    }

    /// Adds absolute mouse movement.
    fn add_mouse_move_to(mut self, x: f64, y: f64) -> Self {
        // Get the boundary of the entire virtual desktop (multi-monitor support).
        let (vx, vy, vw, vh) = Display::get_virtual_screen_boundary();

        if vw <= 1 || vh <= 1 {
            return self;
        }

        let scale_factor = Display::get_scale_factor();

        let phys_x = x * scale_factor;
        let phys_y = y * scale_factor;

        // Normalized mapping logic:
        // Coordinate mapping formula for SendInput: (physical coordinates - start offset) * 65535 / (total size - 1)
        // Use f64 calculations to prevent overflow or loss of precision in intermediate steps.
        let dx = ((phys_x - vx as f64) * 65535.0 / (vw - 1) as f64) as i32;
        let dy = ((phys_y - vy as f64) * 65535.0 / (vh - 1) as f64) as i32;

        self.push_mouse(MOUSEINPUT {
            dx,
            dy,
            // MOUSEEVENTF_VIRTUALDESK ensures correct mapping across all monitors.
            dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
            ..Default::default()
        });

        self
    }

    fn add_mouse_button(mut self, button: MouseButton, down: bool) -> Self {
        let (flags, data) = match (button, down) {
            (MouseButton::Left, true) => (MOUSEEVENTF_LEFTDOWN, 0),
            (MouseButton::Left, false) => (MOUSEEVENTF_LEFTUP, 0),
            (MouseButton::Right, true) => (MOUSEEVENTF_RIGHTDOWN, 0),
            (MouseButton::Right, false) => (MOUSEEVENTF_RIGHTUP, 0),
            (MouseButton::Middle, true) => (MOUSEEVENTF_MIDDLEDOWN, 0),
            (MouseButton::Middle, false) => (MOUSEEVENTF_MIDDLEUP, 0),
            (MouseButton::Back, true) => (MOUSEEVENTF_XDOWN, 1),
            (MouseButton::Back, false) => (MOUSEEVENTF_XUP, 1),
            (MouseButton::Forward, true) => (MOUSEEVENTF_XDOWN, 2),
            (MouseButton::Forward, false) => (MOUSEEVENTF_XUP, 2),
        };
        self.push_mouse(MOUSEINPUT {
            mouseData: data,
            dwFlags: flags,
            ..Default::default()
        });
        self
    }

    fn add_mouse_wheel(mut self, dx: f64, dy: f64) -> Self {
        if dy.abs() > f64::EPSILON {
            self.push_mouse(MOUSEINPUT {
                mouseData: (dy * 120.0) as i32 as u32,
                dwFlags: MOUSEEVENTF_WHEEL,
                ..Default::default()
            });
        }
        if dx.abs() > f64::EPSILON {
            self.push_mouse(MOUSEINPUT {
                mouseData: (dx * 120.0) as i32 as u32,
                dwFlags: MOUSEEVENTF_HWHEEL,
                ..Default::default()
            });
        }
        self
    }

    /// Adds a keyboard event to the input queue.
    fn add_keyboard(mut self, key: Key, down: bool) -> Self {
        // 1. Get Windows-specific Virtual Key and Scan Code from cross-platform Key enum.
        let (vk, scancode) = match get_win_codes(key) {
            Some(codes) => codes,
            None => return self,
        };

        // 2. Determine whether to use Scan Code or Virtual Key mode.
        // Referencing logic: scancode mode is preferred for better compatibility with physical layouts.
        let (w_vk, w_scan, mut flags) = if scancode != 0 {
            // Scan code mode: VK is set to 0.
            (0u16, scancode as u16, KEYEVENTF_SCANCODE.0)
        } else {
            // Virtual key mode: Scancode is set to 0.
            (vk as u16, 0u16, 0u32)
        };

        // 3. Handle Extended Key flags.
        // If the scan code has an 0xE0 or 0xE1 prefix, the KEYEVENTF_EXTENDEDKEY flag must be set.
        if (w_scan >> 8) == 0xE0 || (w_scan >> 8) == 0xE1 {
            flags |= KEYEVENTF_EXTENDEDKEY.0;
        }

        // 4. Handle Key Up flag.
        if !down {
            flags |= KEYEVENTF_KEYUP.0;
        }

        self.push_keyboard(KEYBDINPUT {
            wVk: VIRTUAL_KEY(w_vk),
            wScan: w_scan as u16,
            dwFlags: KEYBD_EVENT_FLAGS(flags),
            ..Default::default()
        });
        self
    }

    fn push_mouse(&mut self, mi: MOUSEINPUT) {
        self.inputs.push(INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 { mi },
        });
    }

    fn push_keyboard(&mut self, ki: KEYBDINPUT) {
        self.inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 { ki },
        });
    }

    fn send(self) {
        if self.inputs.is_empty() {
            return;
        }
        unsafe {
            SendInput(&self.inputs, size_of::<INPUT>() as i32);
        }
    }
}
