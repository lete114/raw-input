#[cfg(test)]
mod listen_tests {
    use serial_test::serial;
    use std::{thread, time::Duration};

    use raw_input::{Core, Listen};

    /// Starts the Core message loop in a background thread and waits for initialization.
    fn start_core_env() {
        thread::spawn(|| {
            let _ = Core::start();
        });
        thread::sleep(Duration::from_millis(500));
    }

    /// Stops all modules and cleans up the Windows environment to prevent side effects.
    fn stop_core_env() {
        Core::stop();
        thread::sleep(Duration::from_millis(300));
    }

    #[serial]
    #[test]
    fn test_listen() {
        start_core_env();

        Listen::start();
        Listen::subscribe(|event| {
            println!("event: {:?}", event);
        });

        thread::sleep(Duration::from_millis(5000));

        stop_core_env();
    }
}
