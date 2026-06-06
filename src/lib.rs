#![no_std]

extern crate alloc;

pub use macropad_model::*;

pub mod containers;
pub mod dummy_time_source;
pub mod encoder;
pub mod example_config;
pub mod read_config;
pub mod update_display;
pub const NUM_LEDS: usize = 8;
