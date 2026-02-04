use std::fs;
use std::os::fd::{AsFd, AsRawFd};
use std::sync::atomic::Ordering;
use std::sync::Mutex;

use evdev::{Device, InputEventKind, Key};

use crate::platform::{
    CoreError, CoreImpl, GrabImpl, ListenImpl, PlatformCore, PlatformGrab, PlatformListen,
    linux::common::IS_CORE_RUNNING,
};

static DEVICES: Mutex<Vec<Device>> = Mutex::new(Vec::new());

impl CoreImpl for PlatformCore {
    fn start() -> Result<(), CoreError> {
        if Self::is_run() {
            return Ok(());
        }

        let devices = Self::enumerate_input_devices()?;
        if devices.is_empty() {
            return Err(CoreError::LinuxKeyboardError);
        }

        {
            let mut guard = DEVICES.lock().unwrap();
            *guard = devices;
        }

        Self::run_event_loop();

        Self::stop();
        Ok(())
    }

    fn is_runing() -> bool {
        IS_CORE_RUNNING.load(Ordering::SeqCst)
    }

    fn pause() {
        IS_CORE_RUNNING.store(false, Ordering::SeqCst);
    }

    fn resume() {
        IS_CORE_RUNNING.store(true, Ordering::SeqCst);
    }

    fn stop() {
        Self::pause();
        PlatformListen::stop();
        PlatformGrab::stop();
        DEVICES.lock().unwrap().clear();
    }
}

impl PlatformCore {
    #[inline]
    fn is_run() -> bool {
        IS_CORE_RUNNING
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
    }

    fn enumerate_input_devices() -> Result<Vec<Device>, CoreError> {
        let mut devices = Vec::new();

        let entries = fs::read_dir("/dev/input")
            .map_err(|_| CoreError::LinuxMissingDisplayError)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.to_string_lossy().contains("event") {
                continue;
            }

            let device = match Device::open(&path) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let has_keys = device
                .supported_keys()
                .map_or(false, |keys| keys.contains(Key::KEY_A) || keys.contains(Key::BTN_LEFT));

            let has_rel = device.supported_relative_axes().is_some();

            if has_keys || has_rel {
                devices.push(device);
            }
        }

        Ok(devices)
    }

    fn run_event_loop() {
        let mut poll_fds: Vec<libc::pollfd> = Vec::new();

        {
            let guard = DEVICES.lock().unwrap();
            for device in guard.iter() {
                poll_fds.push(libc::pollfd {
                    fd: device.as_fd().as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                });
            }
        }

        while IS_CORE_RUNNING.load(Ordering::Relaxed) {
            let ret = unsafe {
                libc::poll(poll_fds.as_mut_ptr(), poll_fds.len() as libc::nfds_t, 100)
            };

            if ret <= 0 {
                continue;
            }

            let mut guard = DEVICES.lock().unwrap();
            for (i, pfd) in poll_fds.iter().enumerate() {
                if pfd.revents & libc::POLLIN == 0 {
                    continue;
                }

                if let Some(device) = guard.get_mut(i) {
                    if let Ok(events) = device.fetch_events() {
                        for event in events {
                            Self::process_event(&event);
                        }
                    }
                }
            }
        }
    }

    fn process_event(event: &evdev::InputEvent) {
        if !IS_CORE_RUNNING.load(Ordering::Relaxed) {
            return;
        }

        match event.kind() {
            InputEventKind::Key(_) => {
                PlatformListen::handle_key(event);
                PlatformGrab::handle_key(event);
            }
            InputEventKind::RelAxis(_) => {
                PlatformListen::handle_rel(event);
                PlatformGrab::handle_rel(event);
            }
            _ => {}
        }
    }
}

