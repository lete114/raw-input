#[cfg(test)]
mod grab_tests {
    use std::{thread, time::Duration};

    use raw_input::{Core, Display, Event, Grab,  Point, Simulate};

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
        Grab::stop();

        // Record initial position
        let initial_pos = Display::get_cursor_pos_physical();

        // 1. Action: Start full grab
        Grab::start();

        // 2. Simulate move via Event
        Simulate::simulate(Event::MouseMove {
            delta: Point { x: 100, y: 100 },
        });
        thread::sleep(Duration::from_millis(200));

        // 3. Assert: Position should NOT change
        let current_pos = Display::get_cursor_pos_physical();
        assert_eq!(
            initial_pos, current_pos,
            "Simulate::simulate move should be blocked by Grab"
        );

        stop_core();
    }

    #[test]
    fn test_grab_mouse_move_interception() {
        start_core();
        Grab::stop();

        let initial_pos = Display::get_cursor_pos_physical();

        // 1. Action: Specifically block mouse move
        Grab::mouse_move(true);
        Grab::resume();

        // 2. Simulate relative move
        Simulate::mouse_move(50, 50);
        thread::sleep(Duration::from_millis(200));

        // 3. Assert: Position should NOT change
        let current_pos = Display::get_cursor_pos_physical();
        assert_eq!(
            initial_pos, current_pos,
            "Simulate::mouse_move should be blocked by Grab"
        );

        stop_core();
    }

    #[test]
    fn test_grab_mouse_move_to_interception() {
        start_core();
        Grab::stop();

        let initial_pos = Display::get_cursor_pos_physical();

        // 1. Action: Start grab
        Grab::start();

        // 2. Simulate absolute move to a different coordinate
        let target_x = initial_pos.0 + 100;
        let target_y = initial_pos.1 + 100;
        Simulate::mouse_move_to(target_x, target_y);
        thread::sleep(Duration::from_millis(200));

        // 3. Assert: Position should NOT change to target
        let current_pos = Display::get_cursor_pos_physical();
        assert_eq!(
            initial_pos, current_pos,
            "Simulate::mouse_move_to should be blocked by Grab"
        );

        stop_core();
    }
}
