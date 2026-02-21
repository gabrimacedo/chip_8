use core::f64;
use std::time::Duration;

pub struct ChipConfig {
    pub target_frame_duration: Duration,
    pub cycles_per_frame: u64,
    pub instructions: Vec<u8>,
}

impl ChipConfig {
    pub fn new(clock_speed: f64, target_frame_rate: f64, instructions: Vec<u8>) -> Self {
        ChipConfig {
            target_frame_duration: Duration::from_millis((1_000.0 / target_frame_rate) as u64),
            cycles_per_frame: (clock_speed / target_frame_rate) as u64,
            instructions,
        }
    }
}
