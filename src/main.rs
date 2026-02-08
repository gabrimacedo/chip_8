use rand::{Rng, rngs::ThreadRng};

fn main() {
    let mut chip = Chip8::new();

    let instructions = [
        0x611E, // load coord (x) 30 into v1
        0x620B, // load coord (y) 11 into v2
        0xA032, // Set I to adrress of sprite 'A'
        0xD125, // display 5 bytes wide letter stored in I
        0x620D, // load coord (y) 13 into v2
        0xA041, // Set I to adrress of sprite 'D'
        0xD125, // display 5 bytes wide letter stored in I
    ];

    chip.load_instructions(&instructions);

    for _ in instructions {
        let op = chip.fetch();
        chip.decode(op);
    }

    dbg!(chip.v(0xf));
}

struct Chip8 {
    memory: [u8; 4096],
    registers: [u8; 16],
    display: [u64; 32],
    pc: u16,
    i: u16,
    rng: ThreadRng,
}

impl Chip8 {
    fn new() -> Self {
        let digit_sprites: [[u8; 5]; 15] = [
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
            memory,
            registers: [0; 16],
            display: [0; 32],
            i: 0,
            pc: 0x200,
            rng: rand::thread_rng(),
        }
    }

    fn draw(display: &mut [u64; 32], (x, y): (u8, u8), vf: &mut u8, sprite: &[u8]) {
        // load to buffer
        for (i, byte) in sprite.iter().enumerate() {
            let sprite = (*byte as u64) << (63 - 8 - x);
            display[(y + i as u8) as usize] ^= sprite;

            //check for erased pixels
            let current = display[(y + i as u8) as usize];
            let erased = (current >> (63 - (y + 5))) as u8 == *byte;
            if erased {
                *vf = 1;
            } else {
                *vf = 0;
            }
        }

        // print
        for line in display {
            let mut shift = 63;
            println!();

            while shift > 0 {
                let bit = (*line >> shift) & 0x1;

                if bit == 1 {
                    print!("x");
                } else {
                    print!(" ");
                }

                shift -= 1;
            }
        }
    }

    fn load_instructions(&mut self, instructions: &[u16]) {
        let mut i = 0x200;

        for opcode in instructions {
            let high = (opcode >> 8) as u8;
            let low = (opcode & 0xff) as u8;

            self.memory[i] = high;
            self.memory[i + 1] = low;
            i += 2;
        }
    }

    fn fetch(&self) -> u16 {
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
        let op_code = (code >> 12) as u8;
        let adrr = code & 0x0fff;
        let kk = (code & 0xff) as u8;
        let nibble = code & 0xf;
        let x = (code >> 8) & 0xf;
        let y = (code >> 4) & 0xf;
        let vx = self.v(x);
        let vy = self.v(y);

        self.pc += 2;

        match op_code {
            0x0 => todo!(),
            0x1 => self.pc = adrr,
            0x2 => todo!(),
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
            0x7 => self.set_v(x, vx + kk),
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
                    self.set_v(0xf, (vx >= vy) as u8);
                    self.set_v(x, vx.wrapping_sub(vy));
                }
                0x6 => {
                    self.set_v(0xf, (vx & 0x1 == 1) as u8);
                    self.set_v(x, vx >> 1);
                }
                0x7 => {
                    self.set_v(0xf, (vy >= vx) as u8);
                    self.set_v(x, vy.wrapping_sub(vx));
                }
                0xE => {
                    self.set_v(0xf, ((vx >> 7) & 0x1 == 1) as u8);
                    self.set_v(x, vx << 1);
                }
                _ => println!("Invalid instruction code: {op_code}"),
            },
            0x9 => {
                if vx != vy {
                    self.pc += 2
                }
            }
            0xA => self.i = adrr,
            0xB => self.pc = adrr + (self.v(0)) as u16,
            0xC => {
                let r = self.rng.r#gen::<u8>();
                self.set_v(x, r & kk);
            }
            0xD => {
                let sprite_data = &self.memory[self.i as usize..(self.i + nibble) as usize];
                Self::draw(
                    &mut self.display,
                    (vx, vy),
                    &mut self.registers[0xf],
                    sprite_data,
                );
            }
            0xE => todo!(),
            0xF => match kk {
                0x07 => todo!(),
                0x0A => todo!(),
                0x15 => todo!(),
                0x18 => todo!(),
                0x1E => self.i += vx as u16,
                // sprite data (0 - F) starts at adrr 0, and are 5 bytes long
                0x29 => self.i = self.memory[(vx * 5) as usize] as u16,
                0x33 => todo!(),
                0x55 => todo!(),
                0x65 => todo!(),
                _ => println!("Invalid instruction code: {op_code}"),
            },
            _ => println!("Invalid instruction code: {op_code}"),
        }
        println!("Counter is {}", self.pc)
    }
}

#[cfg(test)]
mod test {
    use crate::Chip8;

    #[test]
    fn display_letters() {
        let mut chip = Chip8::new();
        let instructions = [
            0x611E, // load coord (x) 30 into v1
            0x620B, // load coord (y) 11 into v2
            0xA032, // Set I to adrress of sprite 'A'
            0xD125, // display 5 bytes wide letter stored in I
            0x620D, // load coord (y) 13 into v2
            0xA041, // Set I to adrress of sprite 'D'
            0xD125, // display 5 bytes wide letter stored in I
        ];

        chip.load_instructions(&instructions);

        for _ in instructions {
            let op = chip.fetch();
            chip.decode(op);
        }
    }

    #[test]
    fn fetch_correct_instructions() {
        let mut chip = Chip8::new();
        let instructions = [0x6042, 0x7005, 0x4047, 0x3047];
        chip.load_instructions(&instructions);

        for intruct in instructions {
            let op = chip.fetch();
            chip.decode(op);

            assert_eq!(op, intruct);
        }
    }

    #[test]
    // add 200 to 10, then add 110 to 210 to test overflow
    fn add_numbers() {
        let mut chip = Chip8::new();
        let instructions = [0x600A, 0x61C8, 0x8014, 0x616E, 0x8014];
        chip.load_instructions(&instructions);

        for _ in instructions {
            let op = chip.fetch();
            chip.decode(op);
        }

        let result = (chip.v(0xf) as u16) << 8 | (chip.v(0) as u16);
        assert_eq!(result, 320);
    }
}
