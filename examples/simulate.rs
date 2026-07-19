use std::{thread, time::Duration};

use raw_input::{Event, Key, MouseButton, Simulate};

fn wait() {
    thread::sleep(Duration::from_millis(500));
}

fn main() {
    println!("Test will start in 3 seconds. Please focus on a text editor.");
    thread::sleep(Duration::from_secs(3));

    println!("Scrolling wheel down...");
    Simulate::mouse_wheel(0.0, -1.0);
    wait();

    println!("Typing 'Hi'...");
    Simulate::keyboard(Key::ShiftLeft, true);
    Simulate::keyboard(Key::KeyH, true);
    Simulate::keyboard(Key::KeyH, false);
    Simulate::keyboard(Key::ShiftLeft, false);
    wait();
    Simulate::keyboard(Key::KeyI, true);
    Simulate::keyboard(Key::KeyI, false);
    wait();

    println!("Moving to (200, 200) and right-clicking...");
    Simulate::mouse_move_to(200.0, 200.0);
    wait();
    Simulate::mouse_button(MouseButton::Right, true);
    Simulate::mouse_button(MouseButton::Right, false);
    wait();

    println!("Jiggling mouse relative...");
    Simulate::mouse_move(50.0, 0.0);
    thread::sleep(Duration::from_millis(100));
    Simulate::mouse_move(0.0, 50.0);
    thread::sleep(Duration::from_millis(100));
    Simulate::mouse_move(-50.0, 0.0);
    thread::sleep(Duration::from_millis(100));
    Simulate::mouse_move(0.0, -50.0);
    wait();

    println!("Testing Event-based simulation (Enter key)...");
    let enter_down = Event::KeyDown {
        key: Key::Enter,
        code: None,
    };
    let enter_up = Event::KeyUp {
        key: Key::Enter,
        code: None,
    };
    Simulate::simulate(enter_down);
    Simulate::simulate(enter_up);

    println!("Simulation tests finished.");
}
