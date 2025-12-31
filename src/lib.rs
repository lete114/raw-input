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
//! use raw_input::{Core, Listen, Event};
//! use std::thread;
//!
//! fn main() {
//!     // 1. Start the core engine in a background thread
//!     thread::spawn(|| {
//!         Core::start().expect("Failed to start core");
//!     });
//!
//!     // 2. Listen to global events
//!     let _handle = Listen::listen(|event| {
//!         if let Event::KeyDown { key } = event {
//!             println!("Key pressed: {:?}", key);
//!         }
//!     });
//!
//!     // Keep the main thread alive
//!     loop { thread::sleep(std::time::Duration::from_secs(1)); }
//! }
//! ```

mod dispatcher;
mod event;
mod keycodes;
mod platform;
mod subscription;

pub use crate::event::{Event, FloatPoint, Key, MouseButton, Point};
pub use crate::platform::MonitorInfo;
pub use crate::subscription::SubscriptionHandle;

pub use crate::platform::{Core, Display, Grab, Listen, Simulate};

pub use crate::platform::CoreError;
