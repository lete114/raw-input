use std::mem::size_of;

use windows::{
    Win32::{
        Foundation::{LPARAM, POINT, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW},
        UI::{
            HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
            WindowsAndMessaging::{
                GetCursorPos, GetMessagePos, GetSystemMetrics, SM_CXSCREEN, SM_CXVIRTUALSCREEN,
                SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
            },
        },
    },
    core::BOOL,
};

use crate::{
    Display,
    platform::{MonitorInfo, windows::common::initialize_dpi_awareness},
};

impl Display {
    /// Returns the UI scale factor of the primary monitor.
    /// This is a convenience method that references the primary monitor's DPI settings.
    pub fn get_scale_factor() -> f64 {
        initialize_dpi_awareness();
        Self::get_primary_monitor()
            .map(|m| m.scale_factor)
            .unwrap_or(1.0)
    }

    /// Retrieves the current cursor position in global physical coordinates.
    /// It attempts to use `GetCursorPos` for high precision, falling back to `GetMessagePos`
    /// if the direct call fails. Coordinates are handled as `i16` to correctly
    /// interpret negative values in multi-monitor setups.
    pub fn get_cursor_position() -> (i32, i32) {
        initialize_dpi_awareness();
        let mut pt = POINT::default();
        unsafe {
            if GetCursorPos(&mut pt).is_ok() {
                (pt.x, pt.y)
            } else {
                let pos = GetMessagePos();
                // Extract coordinates using bitwise operations and cast to i16 to handle negative offsets.
                (
                    (pos & 0xFFFF) as i16 as i32,
                    ((pos >> 16) & 0xFFFF) as i16 as i32,
                )
            }
        }
    }

    /// Gets the physical resolution (width, height) of the primary screen.
    pub fn get_primary_screen_size() -> (i32, i32) {
        initialize_dpi_awareness();
        unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) }
    }

    /// Returns the virtual screen boundary across all monitors.
    /// Results are returned as a tuple of (left, top, width, height).
    pub fn get_virtual_screen_boundary() -> (i32, i32, i32, i32) {
        initialize_dpi_awareness();
        unsafe {
            let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);
            (vx, vy, vw, vh)
        }
    }

    /// Enumerates all connected monitors and retrieves their physical properties.
    pub fn get_available_monitors() -> Vec<MonitorInfo> {
        initialize_dpi_awareness();
        let mut monitors = Vec::new();
        unsafe {
            let _ = EnumDisplayMonitors(
                Some(HDC::default()),
                None,
                Some(monitor_enum_proc),
                LPARAM(&mut monitors as *mut _ as isize),
            );
        }
        monitors
    }

    /// Identifies and returns the primary monitor info if available.
    pub fn get_primary_monitor() -> Option<MonitorInfo> {
        Self::get_available_monitors()
            .into_iter()
            .find(|m| m.is_primary)
    }

    /// Finds the monitor that currently contains the mouse cursor.
    pub fn get_current_monitor() -> Option<MonitorInfo> {
        let (x, y) = Self::get_cursor_position();
        Self::get_monitor_from_point(x, y)
    }

    /// Determines which monitor contains the specified global physical point.
    pub fn get_monitor_from_point(x: i32, y: i32) -> Option<MonitorInfo> {
        Self::get_available_monitors().into_iter().find(|m| {
            x >= m.offset.0
                && x < m.offset.0 + m.size.0
                && y >= m.offset.1
                && y < m.offset.1 + m.size.1
        })
    }
}

/// Windows GDI callback function used to process each monitor during enumeration.
extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _: HDC,
    rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    unsafe {
        let monitors = &mut *(data.0 as *mut Vec<MonitorInfo>);
        let r = *rect;

        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;

        if GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut _).as_bool() {
            // Convert device name from UTF-16 and trim null terminators.
            let name = String::from_utf16_lossy(&info.szDevice)
                .trim_matches(char::from(0))
                .to_string();

            let mut dpi_x: u32 = 0;
            let mut dpi_y: u32 = 0;
            // Fetch effective DPI for the specific monitor to calculate scale factor.
            let _ = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);

            monitors.push(MonitorInfo {
                name,
                is_primary: (info.monitorInfo.dwFlags & 1) != 0,
                offset: (r.left, r.top),
                size: (r.right - r.left, r.bottom - r.top),
                // Windows standard DPI is 96.
                scale_factor: dpi_x as f64 / 96.0,
            });
        }
    };
    true.into()
}
