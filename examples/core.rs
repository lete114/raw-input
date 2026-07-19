use std::{thread, time::Duration};

use raw_input::Core;

fn main() {
    println!("=== Core Lifecycle Management ===");
    test_core_lifecycle_management();

    println!("\n=== Core Pause/Resume Logic ===");
    test_core_pause_resume_logic();

    println!("\n=== Core Reentrancy Protection ===");
    test_core_reentrancy_protection();

    println!("\nAll core tests passed.");
}

fn test_core_lifecycle_management() {
    assert!(
        !Core::is_runing(),
        "Core should not be running before start"
    );

    let core_handle = thread::spawn(|| {
        println!("  Core thread: Calling Core::start()...");
        let result = Core::start();
        println!("  Core thread: Core::start() has exited.");
        result
    });

    thread::sleep(Duration::from_millis(800));

    assert!(
        Core::is_runing(),
        "Core::is_runing() should return true after start"
    );

    println!("  Main thread: Calling Core::stop()...");
    Core::stop();

    let join_result = core_handle.join().expect("Core thread panicked");

    assert!(
        join_result.is_ok(),
        "Core::start() should return Ok(()) after stopping"
    );
    assert!(
        !Core::is_runing(),
        "Core::is_runing() should return false after stop"
    );

    println!("  PASS");
}

fn test_core_pause_resume_logic() {
    thread::spawn(|| {
        let _ = Core::start();
    });
    thread::sleep(Duration::from_millis(500));

    Core::pause();
    assert!(!Core::is_runing(), "is_runing should be false after pause");

    Core::resume();
    assert!(Core::is_runing(), "is_runing should be true after resume");

    Core::stop();
    println!("  PASS");
}

fn test_core_reentrancy_protection() {
    thread::spawn(|| {
        let _ = Core::start();
    });
    thread::sleep(Duration::from_millis(500));
    assert!(Core::is_runing());

    let second_start = Core::start();
    assert!(
        second_start.is_ok(),
        "Subsequent Core::start() should not fail or re-register"
    );

    Core::stop();
    println!("  PASS");
}
