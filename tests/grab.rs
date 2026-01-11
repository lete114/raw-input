#[cfg(test)]
mod grab_tests {
    use std::{thread, time::Duration};

    use raw_input::{Core, Grab, Listen};

    /// Helper to start the core in a background thread
    fn start_core() {
        thread::spawn(|| {
            let _ = Core::start();
        });
        // Wait for hooks to be registered
        thread::sleep(Duration::from_millis(500));
    }

    /// Helper to stop the core and cleanup grab state
    fn stop_core() {
        Grab::stop();
        Core::stop();
        thread::sleep(Duration::from_millis(300));
    }

    #[test]
    fn test_grab_simulate_move_interception() {
        start_core();

        Listen::start();
        Listen::subscribe(|event| {
            println!("event: {:?}", event);
        });
        
        Grab::start();
        thread::sleep(Duration::from_millis(5000));

        stop_core();
    }
}
