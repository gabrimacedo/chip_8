pub mod chip8;

use rand::{Rng, rngs::ThreadRng};
use std::{
    env,
    fs::{self},
    io::{self},
    process,
    thread::sleep,
    time::{Duration, Instant},
};

use crate::chip8::{Display, Input};

fn main() -> io::Result<()> {
    // setup Beep
    let params = tinyaudio::OutputDeviceParameters {
        channels_count: 1,
        sample_rate: 44100,
        channel_sample_count: 735, // 44100 / 60 = one frame's worth
    };

    let mut phase = 0.0f32;
    let beeping = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let beeping_clone = beeping.clone();

    let _device = tinyaudio::run_output_device(params, move |buf| {
        for sample in buf.iter_mut() {
            if beeping_clone.load(std::sync::atomic::Ordering::Relaxed) {
                *sample = (phase * 2.0 * std::f32::consts::PI).sin() * 0.25;
                phase += 440.0 / 44100.0;
                if phase >= 1.0 {
                    phase -= 1.0;
                }
            } else {
                *sample = 0.0;
            }
        }
    })
    .unwrap();

    let mut chip = Chip8::new(
        Display::build(640, 320).expect("could not create window"),
        Input::default(),
    );

    // load program
    let program_path = env::args().nth(1).expect("no argument provided");
    match fs::read(&program_path) {
        Ok(instructions) => chip.load_instructions(&instructions),
        Err(err) => {
            eprintln!("error loading program: {err}");
            process::exit(1);
        }
    }
    // config
    let clock_speed = 700.0;
    let frame_rate = 60.0;

    let target_frame_duration = Duration::from_millis((1_000.0 / frame_rate) as u64);
    let cycles_per_frame = (clock_speed / frame_rate) as u64;

    // Main loop
    loop {
        let frame_start = Instant::now();

        // run cycles for this frame
        for _ in 0..cycles_per_frame {
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

        beeping.store(chip.st > 0, std::sync::atomic::Ordering::Relaxed);
        // update 60Hz timers
        if chip.dt > 0 {
            chip.dt -= 1;
        }
        if chip.st > 0 {
            chip.st -= 1;
        }

        // render display if needed
        chip.display.render();

        // poll input
        chip.input.poll_input(chip.display.window());

        // sleep until full end of frame
        if frame_start.elapsed() < target_frame_duration {
            sleep(target_frame_duration - frame_start.elapsed());
        }
    }
}

enum CpuState {
    Running,
    WaitingForKey { register: u16 },
    WaitingForRelease { register: u16, key: u8 },
}

struct Chip8 {
    state: CpuState,
    memory: [u8; 4096],
    display: Display,
    input: Input,
    registers: [u8; 16],
    stack: [u16; 16],
    sp: u8,
    pc: u16,
    i: u16,
    dt: u8,
    st: u8,
    rng: ThreadRng,
}

impl Chip8 {
    fn new(display: Display, input: Input) -> Self {
        let digit_sprites: [[u8; 5]; 16] = [
            [0xf0, 0x90, 0x90, 0x90, 0xf0], // 0
            [0x20, 0x60, 0x20, 0x20, 0x70], // 1
            [0xf0, 0x10, 0xf0, 0x80, 0xf0], // 2
            [0xf0, 0x10, 0xf0, 0x10, 0xf0], // 3
            [0x90, 0x90, 0xf0, 0x10, 0x10], // 4
            [0xf0, 0x80, 0xf0, 0x10, 0xf0], // 5
            [0xf0, 0x80, 0xf0, 0x90, 0xf0], // 6
            [0xf0, 0x10, 0x20, 0x40, 0x40], // 7
            [0xf0, 0x90, 0xf0, 0x90, 0xf0], // 8
            [0xf0, 0x90, 0xf0, 0x10, 0xf0], // 9
            [0xf0, 0x90, 0xf0, 0x90, 0x90], // A
            [0xe0, 0x90, 0xe0, 0x90, 0xe0], // B
            [0xf0, 0x80, 0x80, 0x80, 0xf0], // C
            [0xe0, 0x90, 0x90, 0x90, 0xe0], // D
            [0xf0, 0x80, 0xf0, 0x80, 0xf0], // E
            [0xf0, 0x80, 0xf0, 0x80, 0x80], // F
        ];

        let mut memory: [u8; 4096] = [0; 4096];

        let mut i = 0;
        for digit in digit_sprites {
            for byte in digit {
                memory[i] = byte;
                i += 1;
            }
        }

        Chip8 {
            state: CpuState::Running,
            memory,
            display,
            input,
            registers: [0; 16],
            i: 0,
            pc: 0x200,
            dt: 0,
            st: 9,
            stack: [0; 16],
            sp: 0,
            rng: rand::thread_rng(),
        }
    }

    fn load_instructions(&mut self, instructions: &[u8]) {
        let mut i = 0x200;

        for intruc in instructions {
            self.memory[i] = *intruc;
            i += 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        let high = self.memory[self.pc as usize] as u16;
        let low = self.memory[(self.pc + 1) as usize] as u16;

        (high << 8) + low
    }

    fn v(&self, reg: u16) -> u8 {
        self.registers[reg as usize]
    }

    fn set_v(&mut self, reg: u16, value: u8) {
        self.registers[reg as usize] = value;
    }

    fn decode(&mut self, code: u16) {
        let op_code = code >> 12;
        let nnn = code & 0x0fff;
        let kk = (code & 0xff) as u8;
        let nibble = code & 0xf;
        let x = (code >> 8) & 0xf;
        let y = (code >> 4) & 0xf;
        let vx = self.v(x);
        let vy = self.v(y);

        self.pc += 2;

        match op_code {
            0x0 => match kk {
                0xE0 => {
                    self.display.clear_vram();
                }
                0xEE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
                _ => println!("Invalid instruction code: 0x{:x}", code),
            },
            0x1 => self.pc = nnn,
            0x2 => {
                self.stack[self.sp as usize] = self.pc;
                self.pc = nnn;
                self.sp += 1;
            }
            0x3 => {
                if vx == kk {
                    self.pc += 2
                }
            }
            0x4 => {
                if vx != kk {
                    self.pc += 2
                }
            }
            0x5 => {
                if vx == vy {
                    self.pc += 2
                }
            }
            0x6 => self.set_v(x, kk),
            0x7 => self.set_v(x, vx.wrapping_add(kk)),
            0x8 => match nibble {
                0x0 => self.set_v(x, vy),
                0x1 => {
                    self.set_v(x, vx | vy);
                    self.set_v(0xf, 0);
                }
                0x2 => {
                    self.set_v(x, vx & vy);
                    self.set_v(0xf, 0);
                }
                0x3 => {
                    self.set_v(x, vx ^ vy);
                    self.set_v(0xf, 0);
                }
                0x4 => {
                    let sum = vx as u16 + vy as u16;
                    self.set_v(x, sum as u8);
                    self.set_v(0xf, (sum > 255) as u8);
                }
                0x5 => {
                    self.set_v(x, vx.wrapping_sub(vy));
                    self.set_v(0xf, (vx >= vy) as u8);
                }
                0x6 => {
                    self.set_v(x, vy.wrapping_shr(1));
                    self.set_v(0xf, (vy & 0x1 == 1) as u8);
                }
                0x7 => {
                    self.set_v(x, vy.wrapping_sub(vx));
                    self.set_v(0xf, (vy >= vx) as u8);
                }
                0xE => {
                    self.set_v(x, vy.wrapping_shl(1));
                    self.set_v(0xf, ((vy >> 7) & 0x1 == 1) as u8);
                }
                _ => println!("Invalid instruction code: 0x{:x}", code),
            },
            0x9 => {
                if vx != vy {
                    self.pc += 2
                }
            }
            0xA => self.i = nnn,
            0xB => self.pc = nnn + (self.v(0)) as u16,
            0xC => {
                let rand = self.rng.r#gen::<u8>();
                self.set_v(x, rand & kk);
            }
            0xD => {
                let sprite_data = &self.memory[self.i as usize..(self.i + nibble) as usize];
                self.display
                    .draw_to_vram((vx, vy), &mut self.registers[0xf], sprite_data);
            }
            0xE => match kk {
                0x9E => {
                    if self.input.is_key_pressed(vx) {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    if !self.input.is_key_pressed(vx) {
                        self.pc += 2;
                    }
                }
                _ => println!("Invalid instruction code: 0x{:x}", code),
            },
            0xF => match kk {
                0x07 => self.set_v(x, self.dt),
                0x0A => self.state = CpuState::WaitingForKey { register: x },
                0x15 => self.dt = vx,
                0x18 => self.st = vx,
                0x1E => self.i += vx as u16,
                0x29 => {
                    self.i = (vx * 5) as u16;
                }
                0x33 => {
                    let hundreds = vx / 100;
                    let decimals = (vx % 100) / 10;
                    let ones = vx % 10;

                    self.memory[self.i as usize] = hundreds;
                    self.memory[(self.i as usize) + 1] = decimals;
                    self.memory[(self.i as usize) + 2] = ones;
                }
                0x55 => {
                    let i = self.i as usize;
                    self.memory[i..=i + (x as usize)]
                        .copy_from_slice(&self.registers[0..=x as usize]);
                    self.i += x + 1;
                }
                0x65 => {
                    let i = self.i as usize;
                    self.registers[0..=x as usize]
                        .copy_from_slice(&self.memory[i..=i + (x as usize)]);
                    self.i += x + 1;
                }
                _ => println!("Invalid instruction code: 0x{:x}", code),
            },
            _ => println!("Invalid instruction code: 0x{:x}", code),
        }
    }
}
