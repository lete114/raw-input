use crate::{Display, platform::MonitorInfo};

use core_foundation::uuid::CFUUIDRef;
use core_graphics::{
    display::{CGDirectDisplayID, CGDisplay},
    event::CGEvent,
    event_source::{CGEventSource, CGEventSourceStateID},
};
use objc2::{msg_send, runtime::AnyObject};
use objc2_app_kit::NSScreen;
use objc2_foundation::{MainThreadMarker, NSString, NSUInteger};

impl Display {
    pub fn get_scale_factor() -> f64 {
        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        let screens = NSScreen::screens(mtm);
        Self::match_scale_factor(CGDisplay::main().id, &screens)
    }

    pub fn get_cursor_position() -> (i32, i32) {
        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).unwrap();
        if let Ok(event) = CGEvent::new(source) {
            let point = event.location();
            (point.x as i32, point.y as i32)
        } else {
            (0, 0)
        }
    }

    pub fn get_primary_screen_size() -> (i32, i32) {
        let display = CGDisplay::main();
        let bounds = display.bounds();
        (bounds.size.width as i32, bounds.size.height as i32)
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
                    id: display_id,
                    name: format!("Monitor #{}", display.model_number()),
                    is_primary: display_id == main_id,
                    offset: (bounds.origin.x as i32, bounds.origin.y as i32),
                    size: (bounds.size.width as i32, bounds.size.height as i32),
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
        let (x, y) = Self::get_cursor_position();
        Self::get_monitor_from_point(x, y)
    }

    pub fn get_monitor_from_point(x: i32, y: i32) -> Option<MonitorInfo> {
        Self::get_available_monitors().into_iter().find(|m| {
            x >= m.offset.0
                && x < m.offset.0 + m.size.0
                && y >= m.offset.1
                && y < m.offset.1 + m.size.1
        })
    }

    fn match_scale_factor(
        id: CGDirectDisplayID,
        screens: &objc2_foundation::NSArray<NSScreen>,
    ) -> f64 {
        let key = NSString::from_str("NSScreenNumber");
        unsafe {
            let target_uuid = CGDisplayCreateUUIDFromDisplayID(id);

            for i in 0..screens.count() {
                let screen = screens.objectAtIndex(i);
                let device_description = screen.deviceDescription();
                let value: *mut AnyObject = msg_send![&device_description, objectForKey: &*key];

                if !value.is_null() {
                    let other_native_id: NSUInteger = msg_send![value, unsignedIntegerValue];
                    let other_uuid =
                        CGDisplayCreateUUIDFromDisplayID(other_native_id as CGDirectDisplayID);

                    if target_uuid == other_uuid {
                        return screen.backingScaleFactor() as f64;
                    }
                }
            }
        }
        1.0
    }
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    pub unsafe fn CGDisplayCreateUUIDFromDisplayID(display: CGDirectDisplayID) -> CFUUIDRef;
}
