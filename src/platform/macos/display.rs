use crate::{Display, platform::MonitorInfo};

use core_foundation::{base::CFRelease, uuid::CFUUIDRef};
use core_graphics::{
    display::{CGDirectDisplayID, CGDisplay},
    event::CGEvent,
    event_source::{CGEventSource, CGEventSourceStateID},
};
use objc2::{msg_send, runtime::AnyObject};
use objc2_app_kit::NSScreen;
use objc2_foundation::{MainThreadMarker, NSArray, NSString, NSUInteger};

// private functions
impl Display {
    fn match_scale_factor(id: CGDirectDisplayID, screens: &NSArray<NSScreen>) -> f64 {
        let key = NSString::from_str("NSScreenNumber");
        unsafe {
            let target_uuid = CGDisplayCreateUUIDFromDisplayID(id);
            let mut scale = 1.0;

            for i in 0..screens.count() {
                let screen = screens.objectAtIndex(i);
                let device_description = screen.deviceDescription();
                let value: *mut AnyObject = msg_send![&device_description, objectForKey: &*key];

                if !value.is_null() {
                    let other_native_id: NSUInteger = msg_send![value, unsignedIntegerValue];
                    let other_uuid =
                        CGDisplayCreateUUIDFromDisplayID(other_native_id as CGDirectDisplayID);

                    if target_uuid == other_uuid {
                        scale = screen.backingScaleFactor() as f64;
                        if !other_uuid.is_null() {
                            CFRelease(other_uuid as _);
                        }
                        break;
                    }
                    if !other_uuid.is_null() {
                        CFRelease(other_uuid as _);
                    }
                }
            }
            if !target_uuid.is_null() {
                CFRelease(target_uuid as _);
            }
            scale
        }
    }
}

// public functions
impl Display {
    pub fn get_scale_factor() -> f64 {
        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        let screens = NSScreen::screens(mtm);
        Self::match_scale_factor(CGDisplay::main().id, &screens)
    }

    pub fn get_cursor_position() -> Option<(f64, f64)> {
        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
        let event = CGEvent::new(source).ok()?;
        let point = event.location();
        Some((point.x, point.y))
    }

    pub fn get_primary_screen_size() -> (f64, f64) {
        let display = CGDisplay::main();
        let bounds = display.bounds();
        (bounds.size.width, bounds.size.height)
    }

    pub fn get_virtual_screen_size() -> (f64, f64) {
        let (_, _, w, h) = Self::get_virtual_screen_bounds();
        (w, h)
    }

    pub fn get_virtual_screen_bounds() -> (f64, f64, f64, f64) {
        let Ok(active_displays) = CGDisplay::active_displays() else {
            return (0.0, 0.0, 0.0, 0.0);
        };

        if active_displays.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let first_bounds = CGDisplay::new(active_displays[0]).bounds();
        let mut min_x = first_bounds.origin.x;
        let mut min_y = first_bounds.origin.y;
        let mut max_x = first_bounds.origin.x + first_bounds.size.width;
        let mut max_y = first_bounds.origin.y + first_bounds.size.height;

        for &id in &active_displays[1..] {
            let bounds = CGDisplay::new(id).bounds();
            min_x = min_x.min(bounds.origin.x);
            min_y = min_y.min(bounds.origin.y);
            max_x = max_x.max(bounds.origin.x + bounds.size.width);
            max_y = max_y.max(bounds.origin.y + bounds.size.height);
        }

        (min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn get_available_monitors() -> Vec<MonitorInfo> {
        let mut monitors = Vec::new();
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        let all_screens = NSScreen::screens(mtm);
        let main_id = CGDisplay::main().id;

        if let Ok(active_displays) = CGDisplay::active_displays() {
            for display_id in active_displays {
                let display = CGDisplay::new(display_id);
                let bounds = display.bounds();

                let scale_factor = Self::match_scale_factor(display_id, &all_screens);

                monitors.push(MonitorInfo {
                    name: format!("Monitor #{}", display.model_number()),
                    is_primary: display_id == main_id,
                    offset: (bounds.origin.x, bounds.origin.y),
                    size: (bounds.size.width, bounds.size.height),
                    scale_factor,
                });
            }
        }
        monitors
    }

    pub fn get_primary_monitor() -> Option<MonitorInfo> {
        Self::get_available_monitors()
            .into_iter()
            .find(|m| m.is_primary)
    }

    pub fn get_current_monitor() -> Option<MonitorInfo> {
        Self::get_cursor_position()
            .map(|(x, y)| Self::get_monitor_from_point(x, y))
            .unwrap_or(None)
    }

    pub fn get_monitor_from_point(x: f64, y: f64) -> Option<MonitorInfo> {
        Self::get_available_monitors().into_iter().find(|m| {
            x >= m.offset.0 as f64
                && x < m.offset.0 as f64 + m.size.0 as f64
                && y >= m.offset.1 as f64
                && y < m.offset.1 as f64 + m.size.1 as f64
        })
    }
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    pub unsafe fn CGDisplayCreateUUIDFromDisplayID(display: CGDirectDisplayID) -> CFUUIDRef;
}
