use std::sync::Mutex;
use std::sync::atomic::Ordering;

use core_foundation::runloop::kCFRunLoopCommonModes;
use core_foundation::runloop::{CFRunLoop, CFRunLoopRun};
use core_graphics::display::CGWarpMouseCursorPosition;
use core_graphics::event::{
    CGEvent, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CallbackResult,
};
use core_graphics::event::{CGEventTap, CGEventTapProxy};

use super::common::{GRAB_FLAG, GRAB_MOUSE_MOVE, INTERESTED_EVENTS, IS_CORE_RUNNING};
use crate::{
    Grab, Listen,
    platform::{Core, CoreError},
};

static CORE_RUN_LOOP: Mutex<Option<CFRunLoop>> = Mutex::new(None);

impl Core {
    pub fn start() -> Result<(), CoreError> {
        // Ensure only one instance is running
        if Self::is_run() {
            return Ok(());
        }

        Self::set_hook()?;

        // Perform cleanup after the message loop exits
        Self::stop();
        Ok(())
    }

    pub fn is_runing() -> bool {
        IS_CORE_RUNNING.load(Ordering::SeqCst)
    }

    pub fn pause() {
        IS_CORE_RUNNING.store(false, Ordering::SeqCst);
    }

    pub fn resume() {
        IS_CORE_RUNNING.store(true, Ordering::SeqCst);
    }

    /// Stops the core engine, unhooks all listeners, and terminates the message loop.
    pub fn stop() {
        Self::pause();
        Listen::stop();
        Grab::stop();
        Self::unhook();
    }
}

impl Core {
    /// Atomic check-and-set to ensure the core starts only once.
    #[inline]
    fn is_run() -> bool {
        IS_CORE_RUNNING
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
    }

    /// Wraps the Win32 SetWindowsHookExW API.
    fn set_hook() -> Result<(), CoreError> {
        let tap = match CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            INTERESTED_EVENTS.to_vec(),
            hook_event_callback,
        ) {
            Ok(tap) => tap,
            Err(_) => return Err(CoreError::MacEventTapError),
        };

        let run_loop_source = match tap.mach_port().create_runloop_source(0) {
            Ok(run_loop_source) => run_loop_source,
            Err(_) => return Err(CoreError::MacLoopSourceError),
        };

        let run_loop = CFRunLoop::get_current();
        run_loop.add_source(&run_loop_source, unsafe { kCFRunLoopCommonModes });
        {
            let mut guard = CORE_RUN_LOOP.lock().unwrap();
            *guard = Some(run_loop.clone());
        }

        tap.enable();

        unsafe { CFRunLoopRun() };

        {
            let mut guard = CORE_RUN_LOOP.lock().unwrap();
            *guard = None;
        }

        Ok(())
    }

    /// Safely removes a hook and resets the atomic pointer.
    fn unhook() {
        if let Some(rl) = CORE_RUN_LOOP.lock().unwrap().as_ref() {
            rl.stop();
        }
    }
}

fn hook_event_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: &CGEvent,
) -> CallbackResult {
    if !IS_CORE_RUNNING.load(Ordering::Relaxed) {
        return CallbackResult::Keep;
    }

    Listen::handle(event_type, event);

    if Grab::should_block(event_type) {
        if (GRAB_FLAG.load(Ordering::Relaxed) & GRAB_MOUSE_MOVE) != 0 {
            unsafe {
                CGWarpMouseCursorPosition(event.location());
            }
        }

        return CallbackResult::Drop;
    }

    CallbackResult::Keep
}
