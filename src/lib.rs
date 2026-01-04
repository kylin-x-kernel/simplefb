#![no_std]
#![allow(dead_code)]

extern crate alloc;

mod buffer;
mod color;
mod config;
mod console;
pub mod picture; // Make picture public so caller can use it if needed

pub use config::FramebufferConfig;
pub use console::SimpleFbConsole;
pub use buffer::LogBuffer;

// No global instance here. The caller (platform code) must manage the instance.
