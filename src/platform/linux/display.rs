use crate::platform::{DisplayImpl, MonitorInfo, PlatformDisplay};

use x11rb::connection::Connection;
use x11rb::protocol::randr::{self, ConnectionExt as RandrExt};
use x11rb::protocol::xproto::ConnectionExt;

impl DisplayImpl for PlatformDisplay {
    fn get_scale_factor() -> f64 {
        get_x11_scale_factor().unwrap_or(1.0)
    }

    fn get_cursor_position() -> Option<(f64, f64)> {
        get_x11_cursor_position()
    }

    fn get_primary_screen_size() -> (f64, f64) {
        Self::get_primary_monitor()
            .map(|m| m.size)
            .unwrap_or((0.0, 0.0))
    }

    fn get_virtual_screen_size() -> (f64, f64) {
        let (_, _, w, h) = Self::get_virtual_screen_bounds();
        (w, h)
    }

    fn get_virtual_screen_bounds() -> (f64, f64, f64, f64) {
        let monitors = Self::get_available_monitors();
        if monitors.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let first = &monitors[0];
        let mut min_x = first.offset.0;
        let mut min_y = first.offset.1;
        let mut max_x = first.offset.0 + first.size.0;
        let mut max_y = first.offset.1 + first.size.1;

        for m in &monitors[1..] {
            min_x = min_x.min(m.offset.0);
            min_y = min_y.min(m.offset.1);
            max_x = max_x.max(m.offset.0 + m.size.0);
            max_y = max_y.max(m.offset.1 + m.size.1);
        }

        (min_x, min_y, max_x - min_x, max_y - min_y)
    }

    fn get_available_monitors() -> Vec<MonitorInfo> {
        get_x11_monitors().unwrap_or_default()
    }

    fn get_primary_monitor() -> Option<MonitorInfo> {
        Self::get_available_monitors()
            .into_iter()
            .find(|m| m.is_primary)
    }

    fn get_current_monitor() -> Option<MonitorInfo> {
        Self::get_cursor_position()
            .and_then(|(x, y)| Self::get_monitor_from_point(x, y))
    }

    fn get_monitor_from_point(x: f64, y: f64) -> Option<MonitorInfo> {
        Self::get_available_monitors().into_iter().find(|m| {
            x >= m.offset.0
                && x < m.offset.0 + m.size.0
                && y >= m.offset.1
                && y < m.offset.1 + m.size.1
        })
    }
}

fn get_x11_cursor_position() -> Option<(f64, f64)> {
    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let screen = &conn.setup().roots[screen_num];
    let reply = conn.query_pointer(screen.root).ok()?.reply().ok()?;
    Some((reply.root_x as f64, reply.root_y as f64))
}

fn get_x11_monitors() -> Option<Vec<MonitorInfo>> {
    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let screen = &conn.setup().roots[screen_num];

    let resources = conn
        .randr_get_screen_resources(screen.root)
        .ok()?
        .reply()
        .ok()?;

    let primary = conn
        .randr_get_output_primary(screen.root)
        .ok()?
        .reply()
        .ok()?
        .output;

    let mut monitors = Vec::new();

    for output in &resources.outputs {
        let output_info = conn
            .randr_get_output_info(*output, resources.config_timestamp)
            .ok()?
            .reply()
            .ok()?;

        if output_info.connection != randr::Connection::CONNECTED {
            continue;
        }

        let crtc = output_info.crtc;
        if crtc == 0 {
            continue;
        }

        let crtc_info = conn
            .randr_get_crtc_info(crtc, resources.config_timestamp)
            .ok()?
            .reply()
            .ok()?;

        let name = String::from_utf8_lossy(&output_info.name).to_string();
        let is_primary = *output == primary;

        monitors.push(MonitorInfo {
            name,
            is_primary,
            offset: (crtc_info.x as f64, crtc_info.y as f64),
            size: (crtc_info.width as f64, crtc_info.height as f64),
            scale_factor: get_x11_scale_factor().unwrap_or(1.0),
        });
    }

    if monitors.is_empty() {
        monitors.push(MonitorInfo {
            name: "Default".to_string(),
            is_primary: true,
            offset: (0.0, 0.0),
            size: (screen.width_in_pixels as f64, screen.height_in_pixels as f64),
            scale_factor: 1.0,
        });
    }

    Some(monitors)
}

fn get_x11_scale_factor() -> Option<f64> {
    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let screen = &conn.setup().roots[screen_num];

    // Get RESOURCE_MANAGER atom which contains Xft.dpi setting
    let rm_atom = conn
        .intern_atom(false, b"RESOURCE_MANAGER")
        .ok()?
        .reply()
        .ok()?
        .atom;

    // Get STRING atom for property type
    let string_atom = conn
        .intern_atom(false, b"STRING")
        .ok()?
        .reply()
        .ok()?
        .atom;

    let xrdb = conn
        .get_property(false, screen.root, rm_atom, string_atom, 0, 4096)
        .ok()?
        .reply()
        .ok()?;

    if !xrdb.value.is_empty() {
        let resources = String::from_utf8_lossy(&xrdb.value);
        // Parse xrdb format: "Xft.dpi:\t192\n" or "Xft.dpi: 192\n"
        for line in resources.lines() {
            if let Some(rest) = line.strip_prefix("Xft.dpi:") {
                if let Ok(dpi) = rest.trim().parse::<f64>() {
                    return Some(dpi / 96.0);
                }
            }
        }
    }

    Some(1.0)
}
