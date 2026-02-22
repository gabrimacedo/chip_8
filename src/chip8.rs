pub mod audio;
pub mod config;
pub mod core;
pub mod display;
pub mod input;

use std::{thread::sleep, time::Instant};

pub use audio::{Audio, BeepingState};
pub use config::ChipConfig;
pub use core::Chip8;
pub use display::Display;
pub use input::Input;

use crate::chip8::core::CpuState;

pub fn run(config: ChipConfig) {
    // init chip
    let mut chip = Chip8::new(
        Display::build(640, 320).expect("could not create window"),
        Input::default(),
        Audio::build().expect("open default sink"),
        &config.instructions,
    );

    loop {
        let frame_start = Instant::now();

        // run cycles for this frame
        for _ in 0..config.cycles_per_frame {
            match chip.state {
                CpuState::Running => {
                    let op = chip.fetch();
                    chip.decode(op);
                }
                CpuState::WaitingForKey { register } => {
                    if let Some(key) = chip.input.get_any_pressed_key() {
                        chip.set_v(register, key);
                        chip.state = CpuState::WaitingForRelease { register, key };
                    }
                }
                CpuState::WaitingForRelease { register, key } => {
                    if !chip.input.is_key_pressed(key) {
                        chip.set_v(register, key);
                        chip.state = CpuState::Running
                    }
                }
            }
        }

        chip.update_timers();

        chip.display.render();

        chip.input.poll(chip.display.window());

        // sleep until full end of frame
        if frame_start.elapsed() < config.target_frame_duration {
            sleep(config.target_frame_duration - frame_start.elapsed());
        }
    }
}
