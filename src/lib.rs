//! # raw-input
//!
//! A cross-platform library for capturing and simulating global input events (keyboard and mouse).
//!
//! ## Core Components
//!
//! - **[`Core`]**: Manages the platform-specific event loop. Must be started to enable other features.
//! - **[`Listen`]**: Provides a way to subscribe to global input events without blocking them.
//! - **[`Simulate`]**: Allows programmatic injection of keyboard and mouse events.
//! - **[`Grab`]**: Enables intercepting and optionally blocking input events from reaching other applications.
//! - **[`Display`]**: Utilities for querying monitor information and cursor positions.
//!
//! ## Example
//!
//! ```no_run
//! use std::thread;
//! use std::time::Duration;
//!
//! use raw_input::{Core, Listen, Event};
//!
//! fn main() {
//!     // 1. Start the core engine in a background thread
//!     // (Crucial for processing Windows message loops)
//!     thread::spawn(|| {
//!         Core::start().expect("Failed to start raw-input core");
//!     });
//!
//!     // 2. Subscribe to global events
//!     Listen::start();
//!     let handle = Listen::subscribe(|event| {
//!         match event {
//!             Event::KeyDown { key } => println!("Key pressed: {:?}", key),
//!             Event::MouseMove { delta } => println!("Mouse moved by: {}, {}", delta.x, delta.y),
//!             _ => {},
//!         }
//!     });
//!
//!     // 3. Manage the subscription lifecycle
//!     thread::sleep(Duration::from_secs(5));
//!     handle.pause();    // Stop receiving events temporarily
//!
//!     thread::sleep(Duration::from_secs(2));
//!     handle.resume();   // Start receiving events again
//!
//!     thread::sleep(Duration::from_secs(2));
//!     handle.unsubscribe(); // Permanently remove the listener
//!     Listen::stop(); // Stop Listen
//!     Core::stop()
//! }
//! ```

mod dispatcher;
mod event;
mod key;
mod platform;
mod subscription;

pub use crate::event::{Event, FloatPoint, MouseButton, Point};
pub use crate::key::{Key, KeyCode};
pub use crate::platform::MonitorInfo;
pub use crate::subscription::SubscriptionHandle;

pub use crate::platform::{Core, Display, Grab, Listen, Simulate};

pub use crate::platform::CoreError;
