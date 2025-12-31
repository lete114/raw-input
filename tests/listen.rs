#[cfg(test)]
mod listen_tests {
    use serial_test::serial;
    use std::{sync::{Arc, Mutex}, thread, time::Duration};

    use raw_input::{Core, Display, Event, Grab, Key, Listen, Simulate};

    /// Starts the Core message loop in a background thread and waits for initialization.
    fn start_core_env() {
        thread::spawn(|| {
            let _ = Core::start();
        });
        thread::sleep(Duration::from_millis(500));
    }

    /// Stops all modules and cleans up the Windows environment to prevent side effects.
    fn stop_core_env() {
        Listen::stop();
        Grab::stop();
        Core::stop();
        thread::sleep(Duration::from_millis(300));
    }

    /// Verifies that Listen can capture MouseMove events even when Grab is active.
    /// Grab prevents the system from moving the actual cursor, but Listen should still see the input.
    #[serial]
    #[test]
    fn test_listen_mouse_move_event() {
        start_core_env();
        Listen::start();
        Grab::start(); // Intercept system input to keep environment stable.

        let received_move = Arc::new(Mutex::new(false));
        let cloned_received = Arc::clone(&received_move);

        let _handle = Listen::subscribe(move |event| {
            if let Event::MouseMove { .. } = event {
                let mut received = cloned_received.lock().unwrap();
                *received = true;
            }
        });

        // Simulate relative movement.
        Simulate::mouse_move(10, 10);
        thread::sleep(Duration::from_millis(200));

        let result = *received_move.lock().unwrap();
        assert!(
            result,
            "Listen failed to capture MouseMove event while Grab was active."
        );

        stop_core_env();
    }

    /// Validates that Listen correctly ignores MOUSE_MOVE_ABSOLUTE events according to the internal logic.
    #[serial]
    #[test]
    fn test_listen_mouse_move_to_interception_logic() {
        start_core_env();
        Listen::start();
        Grab::start();

        let received_any = Arc::new(Mutex::new(false));
        let cloned_received = Arc::clone(&received_any);

        let _handle = Listen::subscribe(move |_| {
            let mut received = cloned_received.lock().unwrap();
            *received = true;
        });

        // Get current position and simulate absolute move.
        // Internal logic: handle_mouse_move returns true (handled) but does not dispatch ABSOLUTE events.
        let (x, y) = Display::get_cursor_pos_physical();
        Simulate::mouse_move_to(x + 50, y + 50);
        thread::sleep(Duration::from_millis(200));

        let result = *received_any.lock().unwrap();
        assert!(
            !result,
            "Listen should ignore MOUSE_MOVE_ABSOLUTE events as they are filtered internally."
        );

        stop_core_env();
    }

    /// Confirms that keyboard events are captured by Listen even if Grab intercepts them from the OS.
    #[serial]
    #[test]
    fn test_listen_keyboard_event() {
        start_core_env();
        Listen::start();
        Grab::start();

        let captured_key = Arc::new(Mutex::new(None));
        let cloned_key = Arc::clone(&captured_key);

        let _handle = Listen::subscribe(move |event| {
            if let Event::KeyDown { key } = event {
                let mut k = cloned_key.lock().unwrap();
                *k = Some(key);
            }
        });

        // Simulate key press.
        Simulate::keyboard(Key::KeyA, true);
        thread::sleep(Duration::from_millis(200));
        Simulate::keyboard(Key::KeyA, false);

        let result = *captured_key.lock().unwrap();
        assert_eq!(
            result,
            Some(Key::KeyA),
            "Listen failed to capture the KeyDown event while Grab was active."
        );

        stop_core_env();
    }

    /// Ensures MouseWheel events are correctly reported by Listen with expected delta values.
    #[serial]
    #[test]
    fn test_listen_mouse_wheel_event() {
        start_core_env();
        Listen::start();
        Grab::start();

        let wheel_delta = Arc::new(Mutex::new(0.0));
        let cloned_delta = Arc::clone(&wheel_delta);

        let _handle = Listen::subscribe(move |event| {
            if let Event::MouseWheel { delta, .. } = event {
                let mut d = cloned_delta.lock().unwrap();
                *d = delta.y;
            }
        });

        // Simulate vertical wheel scroll (1.0 unit).
        Simulate::mouse_wheel(0.0, 1.0);
        thread::sleep(Duration::from_millis(200));

        let result = *wheel_delta.lock().unwrap();
        assert!(
            (result - 1.0).abs() < f64::EPSILON,
            "Listen reported an incorrect MouseWheel delta."
        );

        stop_core_env();
    }

    /// Tests the Pause/Resume mechanism of the Listen module.
    /// Events should be dropped when paused and captured only after resuming.
    #[serial]
    #[test]
    fn test_listen_pause_resume_logic() {
        start_core_env();
        Listen::start();
        Grab::start();

        let counter = Arc::new(Mutex::new(0));
        let cloned_counter = Arc::clone(&counter);

        let _handle = Listen::subscribe(move |_| {
            let mut c = cloned_counter.lock().unwrap();
            *c += 1;
        });

        // Phase 1: Pause listening and simulate movement.
        Listen::pause();
        Simulate::mouse_move(5, 5);
        thread::sleep(Duration::from_millis(100));

        // Phase 2: Resume listening and simulate movement.
        Listen::resume();
        Simulate::mouse_move(5, 5);
        thread::sleep(Duration::from_millis(200));

        let final_count = *counter.lock().unwrap();
        assert_eq!(
            final_count, 1,
            "Listen capture logic failed: expected 1 event (after resume), but got {}.",
            final_count
        );

        stop_core_env();
    }
}
