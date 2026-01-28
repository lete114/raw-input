use std::{mem::size_of, sync::Once};

use windows::{
    Win32::{
        Foundation::{LPARAM, POINT, RECT},
        Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTONEAREST,
            MONITORINFOEXW, MonitorFromPoint,
        },
        UI::{
            HiDpi::{
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForMonitor, MDT_EFFECTIVE_DPI,
                SetProcessDpiAwarenessContext,
            },
            WindowsAndMessaging::{
                GetCursorPos, GetSystemMetrics, MONITORINFOF_PRIMARY, SM_CXSCREEN,
                SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
                SM_YVIRTUALSCREEN, USER_DEFAULT_SCREEN_DPI,
            },
        },
    },
    core::BOOL,
};

use crate::{Display, platform::MonitorInfo};

/// Initializes DPI awareness for the process to ensure coordinates are handled correctly
/// on high-resolution displays. This is called only once.
static DPI_INIT: Once = Once::new();

// private functions
impl Display {
    fn ensure_dpi_awareness() {
        DPI_INIT.call_once(|| unsafe {
            // Set awareness to Per-Monitor V2 for modern Windows 10/11 behavior
            let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        });
    }

    fn get_scale_for_hmonitor(h_monitor: HMONITOR) -> f64 {
        let mut dpi_x: u32 = 0;
        let mut dpi_y: u32 = 0;
        unsafe {
            let _ = GetDpiForMonitor(h_monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
        }
        if dpi_x == 0 {
            1.0
        } else {
            dpi_x as f64 / USER_DEFAULT_SCREEN_DPI as f64
        }
    }
}

// public functions
impl Display {
    /// Returns the UI scale factor of the primary monitor.
    /// This is a convenience method that references the primary monitor's DPI settings.
    pub fn get_scale_factor() -> f64 {
        Self::ensure_dpi_awareness();
        unsafe {
            let h_monitor = MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTONEAREST);
            Self::get_scale_for_hmonitor(h_monitor)
        }
    }

    /// Retrieves the current cursor position in global physical coordinates.
    /// It attempts to use `GetCursorPos` for high precision, falling back to `GetMessagePos`
    /// if the direct call fails. Coordinates are handled as `i16` to correctly
    /// interpret negative values in multi-monitor setups.
    pub fn get_cursor_position() -> Option<(f64, f64)> {
        Self::ensure_dpi_awareness();
        let mut pt = POINT::default();
        unsafe {
            if GetCursorPos(&mut pt).is_ok() {
                Some((pt.x as f64, pt.y as f64))
            } else {
                None
            }
        }
    }

    /// Gets the physical resolution (width, height) of the primary screen.
    pub fn get_primary_screen_size() -> (f64, f64) {
        Self::ensure_dpi_awareness();
        unsafe {
            (
                GetSystemMetrics(SM_CXSCREEN) as f64,
                GetSystemMetrics(SM_CYSCREEN) as f64,
            )
        }
    }

    pub fn get_virtual_screen_size() -> (f64, f64) {
        let (_, _, w, h) = Self::get_virtual_screen_bounds();
        (w as f64, h as f64)
    }

    /// Returns the virtual screen boundary across all monitors.
    /// (x, y, width, height) in logical units
    pub fn get_virtual_screen_bounds() -> (f64, f64, f64, f64) {
        Self::ensure_dpi_awareness();
        unsafe {
            let vx = GetSystemMetrics(SM_XVIRTUALSCREEN) as f64;
            let vy = GetSystemMetrics(SM_YVIRTUALSCREEN) as f64;
            let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN) as f64;
            let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN) as f64;
            (vx, vy, vw, vh)
        }
    }

    /// Enumerates all connected monitors and retrieves their physical properties.
    pub fn get_available_monitors() -> Vec<MonitorInfo> {
        Self::ensure_dpi_awareness();
        let mut monitors = Vec::new();
        unsafe {
            let _ = EnumDisplayMonitors(
                None,
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
        Self::get_cursor_position()
            .map(|(x, y)| Self::get_monitor_from_point(x, y))
            .unwrap_or(None)
    }

    /// Determines which monitor contains the specified global physical point.
    pub fn get_monitor_from_point(x: f64, y: f64) -> Option<MonitorInfo> {
        Self::get_available_monitors().into_iter().find(|m| {
            x >= m.offset.0 as f64
                && x < m.offset.0 as f64 + m.size.0 as f64
                && y >= m.offset.1 as f64
                && y < m.offset.1 as f64 + m.size.1 as f64
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

            let offset = (r.left as f64, r.top as f64);
            let size = ((r.right - r.left) as f64, (r.bottom - r.top) as f64);

            let scale_factor = Display::get_scale_for_hmonitor(hmonitor);
            monitors.push(MonitorInfo {
                name,
                is_primary: (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0,
                offset,
                size,
                scale_factor,
            });
        }
    };
    true.into()
}
