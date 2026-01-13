#[cfg(test)]
mod core_tests {
    use serial_test::serial;
    use std::{thread, time::Duration};

    use raw_input::{Core, Event, Grab, Key, Listen};

    #[test]
    fn test_input() {
        thread::spawn(|| {
            let _ = Core::start();
        });

        Grab::start();
        Listen::start();
        Listen::subscribe(|event| {
            println!("event: {:?}", event);
            match event {
                Event::KeyUp { key, .. } => {
                    if key == Key::Escape {
                        println!("stop ...");
                        std::process::exit(0);
                    }
                }
                _ => {}
            }
        });

        // thread::sleep(Duration::from_millis(5000));
        // std::process::exit(0);

        loop {}
    }

    /// Test the full start-to-stop lifecycle of the Core.
    #[serial]
    #[test]
    fn test_core_lifecycle_management() {
        // 1. Pre-check: Core should not be running initially
        assert!(
            !Core::is_runing(),
            "Core should not be running before start"
        );

        // 2. Start Core in a background thread because it's a blocking call
        let core_handle = thread::spawn(|| {
            println!("Core thread: Calling Core::start()...");
            let result = Core::start();
            println!("Core thread: Core::start() has exited.");
            result
        });

        // 3. Wait for the Core to initialize (Window creation and Hook setup)
        thread::sleep(Duration::from_millis(800));

        // 4. Assert: Core should be running now
        assert!(
            Core::is_runing(),
            "Core::is_runing() should return true after start"
        );

        // 5. Action: Stop the Core
        // This triggers unhooking and posts WM_QUIT to the message loop
        println!("Main thread: Calling Core::stop()...");
        Core::stop();

        // 6. Join the thread to ensure the message loop actually terminated
        let join_result = core_handle.join().expect("Core thread panicked");

        // 7. Final Assertions
        assert!(
            join_result.is_ok(),
            "Core::start() should return Ok(()) after stopping"
        );
        assert!(
            !Core::is_runing(),
            "Core::is_runing() should return false after stop"
        );
    }

    /// Test the pause and resume functionality via Core's state flags.
    #[serial]
    #[test]
    fn test_core_pause_resume_logic() {
        // Start core
        thread::spawn(|| {
            let _ = Core::start();
        });
        thread::sleep(Duration::from_millis(500));

        // Test Pause
        Core::pause();
        // Here we assume IS_CORE_RUNNING is the backing store for is_runing()
        assert!(!Core::is_runing(), "is_runing should be false after pause");

        // Test Resume
        Core::resume();
        assert!(Core::is_runing(), "is_runing should be true after resume");

        // Cleanup
        Core::stop();
    }

    /// Verify that multiple calls to start() are handled gracefully.
    #[serial]
    #[test]
    fn test_core_reentrancy_protection() {
        // First start
        thread::spawn(|| {
            let _ = Core::start();
        });
        thread::sleep(Duration::from_millis(500));
        assert!(Core::is_runing());

        // Second start should return Ok(()) immediately due to is_run() check
        let second_start = Core::start();
        assert!(
            second_start.is_ok(),
            "Subsequent Core::start() should not fail or re-register"
        );

        Core::stop();
    }
}
