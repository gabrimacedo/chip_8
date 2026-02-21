pub mod chip8;

use std::{
    env,
    fs::{self},
};

use crate::chip8::ChipConfig;

fn main() {
    let program_path = env::args().nth(1).expect("no argument provided");
    let instructions = fs::read(&program_path).expect("Could not load program");

    let clock_speed = 700.0;
    let target_frame_rate = 60.0;

    chip8::run(ChipConfig::new(
        clock_speed,
        target_frame_rate,
        instructions,
    ));
}
