use minifb::{Key, Window, WindowOptions};
use rand::{Rng, rngs::ThreadRng};
use std::{
    env,
    fs::{self},
    io::{self},
    process,
    thread::sleep,
    time::Duration,
};

fn main() -> io::Result<()> {
    let mut chip = Chip8::new();

    let program_path = env::args().nth(1).expect("no argument provided");
    match fs::read(&program_path) {
        Ok(instructions) => chip.load_instructions(&instructions),
        Err(err) => {
            eprintln!("error loading program: {err}");
            process::exit(1);
        }
    }

    // 700 mhz = 700 instructions per second
    // or 1 instruction per 1_000_000 / 700 microseconds
    let clock_speed = 700.0;
    let intruction_interval = 1_000_000.0 / clock_speed;

    let cycles_per_tick = ((1_000_000.0 / 60.0) / intruction_interval) as u64;
    let mut cycle_counter = 0;

    let mut window = match Window::new("Test", 640, 320, WindowOptions::default()) {
        Ok(win) => win,
        Err(err) => {
            println!("Unable to crate window {}", err);
            process::exit(1);
        }
    };
    window.set_target_fps(0);

    loop {
        window
            .update_with_buffer(&chip.display_buffer, 640, 320)
            .expect("Could not update bufffer");

        chip.update_keys(&window);

        // 60Hz timers
        if cycle_counter % cycles_per_tick == 0 {
            // decrease timers
            if chip.dt > 0 {
                chip.dt -= 1;
            }
            if chip.st > 0 {
                chip.st -= 1;
            }

            // render display
            if chip.draw_flag {
                chip.render();
                chip.draw_flag = false;
            }

            // when it reaches cycles_per_tick, reset to 0
            cycle_counter = 0;
        }
        cycle_counter += 1;

        match chip.state {
            CpuState::Running => {
                let op = chip.fetch();
                chip.decode(op);
            }
            CpuState::WaitingForKey { register } => {
                if let Some(key) = chip.get_any_pressed_key() {
                    chip.set_v(register, key);
                    chip.state = CpuState::WaitingForRelease { register, key };
                }
            }
            CpuState::WaitingForRelease { register, key } => {
                if !chip.is_key_pressed(key) {
                    chip.set_v(register, key);
                    chip.state = CpuState::Running
                }
            }
        }

        sleep(Duration::from_micros(intruction_interval as u64));
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
    registers: [u8; 16],
    display: [u64; 32],
    display_buffer: Vec<u32>,
    draw_flag: bool,
    stack: [u16; 16],
    sp: u8,
    pc: u16,
    i: u16,
    dt: u8,
    st: u8,
    rng: ThreadRng,
    keys: [bool; 16],
}

impl Chip8 {
    fn new() -> Self {
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
            registers: [0; 16],
            display: [0; 32],
            display_buffer: vec![0; 640 * 320],
            draw_flag: false,
            i: 0,
            pc: 0x200,
            dt: 0,
            st: 9,
            stack: [0; 16],
            sp: 0,
            rng: rand::thread_rng(),
            keys: [false; 16],
        }
    }

    fn draw(display: &mut [u64; 32], (x, y): (u8, u8), vf: &mut u8, sprite: &[u8]) {
        *vf = 0; // reset vf flag

        for (row, sprite_byte) in sprite.iter().enumerate() {
            let overflow = (x % 64) as u32;
            let data = ((*sprite_byte as u64) << 56).rotate_right(overflow);

            let line = &mut display[((y as usize) + row) % 32];

            // check collision
            if data & *line != 0 {
                *vf = 1
            }

            *line ^= data;
        }
    }

    fn render(&mut self) {
        let window_width = 640;

        fn u8_to_rgb(r: u8, g: u8, b: u8) -> u32 {
            let mut rgb: u32 = 0;
            rgb |= (r as u32) << 16;
            rgb |= (g as u32) << 8;
            rgb |= b as u32;

            rgb
        }

        for (row, num) in self.display.iter().enumerate() {
            for shift in (0..64).rev() {
                let bit = num >> shift & 0x1;

                for i in 0..10 {
                    let x = ((63 - shift) * 10) + i;
                    for j in 0..10 {
                        let y = row * 10 * window_width + (j * window_width);
                        self.display_buffer[x + y] = if bit != 0 {
                            u8_to_rgb(255, 255, 255)
                        } else {
                            u8_to_rgb(0, 0, 0)
                        }
                    }
                }
            }
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

    fn get_any_pressed_key(&self) -> Option<u8> {
        for (i, &pressed) in self.keys.iter().enumerate() {
            if pressed {
                return Some(i as u8);
            }
        }
        None
    }

    fn update_keys(&mut self, window: &Window) {
        self.keys[0] = window.is_key_down(Key::Key0);
        self.keys[1] = window.is_key_down(Key::Key1);
        self.keys[2] = window.is_key_down(Key::Key2);
        self.keys[3] = window.is_key_down(Key::Key3);
        self.keys[4] = window.is_key_down(Key::Key4);
        self.keys[5] = window.is_key_down(Key::Key5);
        self.keys[6] = window.is_key_down(Key::Key6);
        self.keys[7] = window.is_key_down(Key::Key7);
        self.keys[8] = window.is_key_down(Key::Key8);
        self.keys[9] = window.is_key_down(Key::Key9);
        self.keys[0xA] = window.is_key_down(Key::A);
        self.keys[0xB] = window.is_key_down(Key::B);
        self.keys[0xC] = window.is_key_down(Key::C);
        self.keys[0xD] = window.is_key_down(Key::D);
        self.keys[0xE] = window.is_key_down(Key::E);
        self.keys[0xF] = window.is_key_down(Key::F);
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
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
                    self.display.iter_mut().for_each(|line| *line = 0);
                    self.draw_flag = true;
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
                0x1 => self.set_v(x, vx | vy),
                0x2 => self.set_v(x, vx & vy),
                0x3 => self.set_v(x, vx ^ vy),
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
                    self.set_v(x, vx.wrapping_shr(1));
                    self.set_v(0xf, (vx & 0x1 == 1) as u8);
                }
                0x7 => {
                    self.set_v(x, vy.wrapping_sub(vx));
                    self.set_v(0xf, (vy >= vx) as u8);
                }
                0xE => {
                    self.set_v(x, vx.wrapping_shl(1));
                    self.set_v(0xf, ((vx >> 7) & 0x1 == 1) as u8);
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
                Self::draw(
                    &mut self.display,
                    (vx, vy),
                    &mut self.registers[0xf],
                    sprite_data,
                );
                self.draw_flag = true;
            }
            0xE => match kk {
                0x9E => {
                    if self.is_key_pressed(vx) {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    if !self.is_key_pressed(vx) {
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
                }
                0x65 => {
                    let i = self.i as usize;
                    self.registers[0..=x as usize]
                        .copy_from_slice(&self.memory[i..=i + (x as usize)]);
                }
                _ => println!("Invalid instruction code: 0x{:x}", code),
            },
            _ => println!("Invalid instruction code: 0x{:x}", code),
        }
    }
}
