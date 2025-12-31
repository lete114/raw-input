use std::{
    ffi::c_void,
    mem::size_of,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicU32, Ordering},
};

use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::{
            Input::{RAWINPUTDEVICE, RIDEV_INPUTSINK, RegisterRawInputDevices},
            WindowsAndMessaging::{
                CallNextHookEx, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
                HC_ACTION, HHOOK, HWND_MESSAGE, MSG, PostMessageW, PostThreadMessageW,
                RegisterClassW, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL,
                WH_MOUSE_LL, WINDOWS_HOOK_ID, WM_INPUT, WM_QUIT, WNDCLASSW,
            },
        },
    },
    core::w,
};

use crate::{
    Grab, Listen,
    platform::{
        Core, CoreError,
        windows::{
            common::{GLOBAL_HWND, IS_CORE_RUNNING, IS_GRAB_RUNNING},
            grab::{KEYBOARD_HOOK, MOUSE_HOOK},
        },
    },
};

/// Stores the ID of the thread running the message loop to allow remote shutdown.
static CORE_THREAD_ID: AtomicU32 = AtomicU32::new(0);

impl Core {
    /// Starts the core engine and blocks the current thread with a Windows message loop.
    /// 
    /// Because this function is blocking, you should typically call it in a dedicated thread.
    /// 
    /// # Example
    /// 
    /// ```
    /// use std::thread;
    /// use raw_input::Core;
    /// 
    /// thread::spawn(|| {
    ///     if let Err(e) = Core::start() {
    ///         eprintln!("Core error: {:?}", e);
    ///     }
    /// });
    /// ```
    pub fn start() -> Result<(), CoreError> {
        // Ensure only one instance is running
        if Self::is_run() {
            return Ok(());
        }

        // Initialize a hidden window to receive Raw Input messages (WM_INPUT)
        if GLOBAL_HWND.load(Ordering::SeqCst).is_null() {
            let hwnd: HWND = Self::setup_raw_input_window()?;
            GLOBAL_HWND.store(hwnd.0, Ordering::SeqCst);
        }

        // Set up low-level system hooks for mouse and keyboard
        Self::handle_hook(WH_MOUSE_LL)?;
        Self::handle_hook(WH_KEYBOARD_LL)?;

        unsafe {
            // Save current thread ID so stop() can send WM_QUIT to this thread
            CORE_THREAD_ID.store(GetCurrentThreadId(), Ordering::SeqCst);
        }

        // Standard Win32 Message Loop: Required for hooks and Raw Input to function
        let mut msg = MSG::default();
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                DispatchMessageW(&msg);
            }
        }

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

        // Remove Windows Hooks
        Self::unhook(&MOUSE_HOOK);
        Self::unhook(&KEYBOARD_HOOK);

        // Notify the core thread to exit the GetMessage loop
        let thread_id = CORE_THREAD_ID.swap(0, Ordering::SeqCst);
        if thread_id != 0 {
            unsafe {
                let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }

        // Destroy the message-only window
        let hwnd = GLOBAL_HWND.load(Ordering::SeqCst);
        if !hwnd.is_null() {
            unsafe {
                let _ = PostMessageW(Some(HWND(hwnd)), WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }
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

    /// Internal helper to manage hook registration and error mapping.
    fn handle_hook(hook_id: WINDOWS_HOOK_ID) -> Result<(), CoreError> {
        let target_static = match hook_id {
            WH_MOUSE_LL => &MOUSE_HOOK,
            WH_KEYBOARD_LL => &KEYBOARD_HOOK,
            _ => return Ok(()),
        };

        // Skip if hook is already active
        if !target_static.load(Ordering::SeqCst).is_null() {
            return Ok(());
        }

        let hook = Self::set_hook(hook_id, hook_event_callback);
        match hook {
            Ok(hook) => {
                target_static.store(hook.0, Ordering::SeqCst);
                Ok(())
            }
            Err(err) => match hook_id {
                WH_MOUSE_LL => Err(CoreError::WindowsMouseHookError(err)),
                WH_KEYBOARD_LL => Err(CoreError::WindowsKeyHookError(err)),
                _ => Ok(()),
            },
        }
    }

    /// Wraps the Win32 SetWindowsHookExW API.
    fn set_hook(
        hook_id: WINDOWS_HOOK_ID,
        callback: extern "system" fn(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT,
    ) -> Result<HHOOK, String> {
        unsafe {
            let instance = GetModuleHandleW(None);
            let instance = match instance {
                Ok(i) => i,
                Err(err) => return Err(format!("Failed to get module handle: {:?}", err)),
            };
            let h_instance = HINSTANCE(instance.0);
            
            // Register a low-level hook. 0 as the thread ID means global hook.
            let hook = SetWindowsHookExW(hook_id, Some(callback), Some(h_instance), 0);
            match hook {
                Ok(h) => Ok(h),
                Err(err) => Err(format!("Failed to set hook: {:?}", err)),
            }
        }
    }

    /// Safely removes a hook and resets the atomic pointer.
    fn unhook(hook: &AtomicPtr<c_void>) {
        let ptr = hook.swap(null_mut(), Ordering::SeqCst);
        if !ptr.is_null() {
            unsafe {
                let _ = UnhookWindowsHookEx(HHOOK(ptr));
            }
        }
    }

    /// Creates a hidden "message-only" window to subscribe to Raw Input events.
    /// This allows receiving high-definition mouse delta data without a visible UI.
    fn setup_raw_input_window() -> Result<HWND, CoreError> {
        unsafe {
            let instance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = w!("RawInputMouseDeltaClass");

            let wc = WNDCLASSW {
                lpfnWndProc: Some(listen_mouse_move_event_callback),
                hInstance: instance.into(),
                lpszClassName: class_name,
                ..Default::default()
            };
            RegisterClassW(&wc);

            // HWND_MESSAGE makes this a "Message-Only" window, invisible to the user
            let hwnd = CreateWindowExW(
                Default::default(),
                class_name,
                w!("RawInputMouseDeltaWindow"),
                Default::default(),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE), 
                None,
                Some(instance.into()),
                None,
            )
            .map_err(|e| {
                CoreError::WindowsRegisterRawInputError(format!("CreateWindowExW failed: {:?}", e))
            })?;

            // Register Mouse (Usage: 0x02) for Raw Input.
            // RIDEV_INPUTSINK allows receiving input even when the window is not focused.
            let devices = [RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x02,
                dwFlags: RIDEV_INPUTSINK,
                hwndTarget: hwnd,
            }];

            RegisterRawInputDevices(&devices, size_of::<RAWINPUTDEVICE>() as u32).map_err(|e| {
                CoreError::WindowsRegisterRawInputError(format!("Registration failed: {:?}", e))
            })?;

            Ok(hwnd)
        }
    }
}

/// The callback function invoked by Windows for every low-level keyboard/mouse event.
extern "system" fn hook_event_callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // HC_ACTION means the hook is processing an actual input event
    if code == HC_ACTION as i32 {
        // Dispatch the event to the Listen module for monitoring
        Listen::handle(wparam, lparam);

        // If the 'Grab' (interception) feature is active, check if we should block this event
        if !IS_GRAB_RUNNING.load(Ordering::Relaxed) {
            return unsafe { CallNextHookEx(None, code, wparam, lparam) };
        }
        
        let msg = wparam.0 as u32;
        if Grab::should_block(msg) {
            // Returning LRESULT(1) consumes the event and prevents it from reaching other apps
            return LRESULT(1);
        }
    }

    // Always call the next hook in the chain if we don't block the event
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Window Procedure for the hidden window to handle WM_INPUT messages.
extern "system" fn listen_mouse_move_event_callback(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_INPUT {
        // Raw Input provides relative mouse movement (deltas)
        let is_handle = Listen::handle_mouse_move(lparam);
        if is_handle {
            return LRESULT(0);
        }
    }

    // Pass unhandled messages to the default window procedure
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
