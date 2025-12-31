#[cfg(test)]
mod simulate_tests {
    use raw_input::{Display, Event, Key, MouseButton, Simulate};
    use serial_test::serial;
    use std::thread;
    use std::time::Duration;

    // Helper function to provide a delay between simulated actions
    // to allow Windows to process the message queue and for the user to observe.
    fn wait() {
        thread::sleep(Duration::from_millis(500));
    }

    #[serial]
    #[test]
    fn test_simulation_workflow() {
        // Step 0: Warm up. Give the user time to switch to a target window (e.g., Notepad).
        println!("Test will start in 3 seconds. Please focus on a text editor.");
        thread::sleep(Duration::from_secs(3));

        // 1. Test Mouse Wheel
        // Scroll down a bit
        println!("Scrolling wheel down...");
        Simulate::mouse_wheel(0.0, -1.0);
        wait();

        // 2. Test Keyboard Input
        // Type "Hi"
        println!("Typing 'Hi'...");
        Simulate::keyboard(Key::ShiftLeft, true);
        Simulate::keyboard(Key::KeyH, true);
        Simulate::keyboard(Key::KeyH, false);
        Simulate::keyboard(Key::ShiftLeft, false);
        wait();
        Simulate::keyboard(Key::KeyI, true);
        Simulate::keyboard(Key::KeyI, false);
        wait();

        // 3. Test Mouse Button & Movement
        // Move to a specific location and perform a right-click
        println!("Moving to (200, 200) and right-clicking...");
        Simulate::mouse_move_to(200, 200);
        wait();
        Simulate::mouse_button(MouseButton::Right, true);
        Simulate::mouse_button(MouseButton::Right, false);
        wait();

        // 4. Test Mouse Relative Movement
        // Jiggle the mouse cursor
        println!("Jiggling mouse relative...");
        Simulate::mouse_move(50, 0);
        thread::sleep(Duration::from_millis(100));
        Simulate::mouse_move(0, 50);
        thread::sleep(Duration::from_millis(100));
        Simulate::mouse_move(-50, 0);
        thread::sleep(Duration::from_millis(100));
        Simulate::mouse_move(0, -50);
        wait();

        // 5. Test Event Wrapper
        // This tests the `add_event` match logic
        println!("Testing Event-based simulation (Enter key)...");
        let enter_down = Event::KeyDown { key: Key::Return };
        let enter_up = Event::KeyUp { key: Key::Return };
        Simulate::simulate(enter_down);
        Simulate::simulate(enter_up);

        println!("Simulation tests finished.");
    }

    #[serial]
    #[test]
    fn test_coordinate_conversion() {
        // This test specifically checks the absolute coordinate mapping logic.
        // It moves the cursor to the four corners of the virtual screen.
        let (vx, vy, vw, vh) = Display::get_virtual_screen_boundary();
        println!("Virtual Screen: x={}, y={}, w={}, h={}", vx, vy, vw, vh);

        if vw > 1 && vh > 1 {
            println!("Moving to Top-Left...");
            Simulate::mouse_move_to(vx, vy);
            thread::sleep(Duration::from_millis(800));

            println!("Moving to Bottom-Right...");
            Simulate::mouse_move_to(vx + vw - 1, vy + vh - 1);
            thread::sleep(Duration::from_millis(800));
        }
    }

    #[test]
    #[serial]
    fn test_move_and_scroll() {
        let start = std::time::Instant::now();
        println!("Move to Bottom...");
        while start.elapsed() < Duration::from_secs(1) {
            Simulate::mouse_wheel(0.0, -0.1);
            thread::sleep(Duration::from_millis(20));
        }

        let start = std::time::Instant::now();

        println!("Move to Top...");
        while start.elapsed() < Duration::from_secs(1) {
            Simulate::mouse_wheel(0.0, 0.1);
            thread::sleep(Duration::from_millis(20));
        }

        println!("Move to Right...");
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(1) {
            Simulate::mouse_wheel(0.1, 0.0);
            thread::sleep(Duration::from_millis(20));
        }

        thread::sleep(Duration::from_millis(500));

        println!("Move to Left...");
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(1) {
            Simulate::mouse_wheel(-0.1, 0.0);
            thread::sleep(Duration::from_millis(20));
        }
    }
}
