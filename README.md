
# raw-input

**raw-input** is a lightweight, high-performance cross-platform Rust library designed to capture and simulate global input events (keyboard and mouse). It is ideal for building global hotkey managers, input monitors, automation scripts, and accessibility tools.

## ✨ Features

* **Global Event Listening**: Capture system-wide keyboard, mouse movement, clicks, and scroll events without requiring window focus.
* **Subscription Management**: Each listener returns a `SubscriptionHandle` that allows you to **Pause**, **Resume**, or **Unsubscribe** at runtime.
* **Input Interception (Grab)**: Intercept and optionally block specific input events from reaching other applications.
* **Input Simulation**: Inject physical-level keyboard and mouse events, supporting both relative movement and absolute screen coordinates.
* **Display Utilities**: Query monitor information, physical resolutions, and DPI scale factors.
* **Thread-Safe**: Designed with `DashMap` and atomic operations for safe multi-threaded usage.

## 🚀 Quick Start

Install this to your `Cargo.toml`:

```bash
cargo add raw-input
```

### Basic Example: Monitoring Global Input

```rust
use std::thread;
use std::time::Duration;

use raw_input::{Core, Listen, Event};

fn main() {
    // 1. Start the core engine in a background thread 
    // (Crucial for processing Windows message loops)
    thread::spawn(|| {
        Core::start().expect("Failed to start raw-input core");
    });

    // 2. Subscribe to global events
    Listen::start();
    let handle = Listen::subscribe(|event| {
        match event {
            Event::KeyDown { key } => println!("Key pressed: {:?}", key),
            Event::MouseMove { delta } => println!("Mouse moved by: {}, {}", delta.x, delta.y),
            _ => {},
        }
    });

    // 3. Manage the subscription lifecycle
    thread::sleep(Duration::from_secs(5));
    handle.pause();    // Stop receiving events temporarily

    thread::sleep(Duration::from_secs(2));
    handle.resume();   // Start receiving events again

    handle.unsubscribe(); // Permanently remove the listener
    Listen::stop(); // Stop Listen
    Core::stop()
}
```

## 🛠 Core Modules

| Module | Description |
| --- | --- |
| **`Core`** | The underlying driver. Manages low-level hooks and the OS event loop. |
| **`Listen`** | The primary interface for subscribing to global input events. |
| **`Simulate`** | Provides tools to programmatically synthesize keyboard and mouse input. |
| **`Grab`** | Allows exclusive access to inputs by blocking them for other applications. |
| **`Display`** | Utilities for monitor enumeration and coordinate mapping. |

## 📦 Optional Features

* `serialize`: Enables `serde` support (Serialize/Deserialize) for event structures like `Event`, `Key`, and `Point`.

## 🖥 Platform Support

| OS | Status | Notes |
| --- | --- | --- |
| **Windows** | ✅ Supported | Implemented via `SetWindowsHookEx` and `Raw Input` API. |
| **macOS** | 🚧 Planned | Will be based on `CGEventTap`. |
| **Linux** | 🚧 Planned | Will be based on `XRecord` or `evdev`. |
