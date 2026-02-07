use rand::{Rng, rngs::ThreadRng};

fn main() {
    let mut chip = Chip8::new();
    chip.load_instructions(&[0x6042, 0x7005, 0x4047, 0x3047]);

    loop {
        // fetch instruction from memory 0 and forth
        let op = chip.fetch();
        if op == 0 {
            break;
        }
        // decode instruction
        chip.decode(op);
        // execute instruction
        dbg!(&chip.v(0));
    }

    let four: &[u8] = &[0x90, 0x90, 0xf0, 0x10, 0x10];
    let two: &[u8] = &[0xf0, 0x10, 0xf0, 0x80, 0xf0];
    let zero: &[u8] = &[0xf0, 0x90, 0x90, 0x90, 0xf0];
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
        Chip8 {
            memory: [0; 4096],
            registers: [0; 16],
            display: [0; 32],
            i: 0,
            pc: 0x200,
            rng: rand::thread_rng(),
        }
    }

    fn load_sprite_data(display: &mut [u64; 32], (x, y): (u8, u8), sprite: &[u8]) {
        // TODO: make it prettier danmit
        for (i, byte) in sprite.iter().enumerate() {
            let z = (*byte as u64) << (63 - 8 - x);
            display[(y + i as u8) as usize] ^= z;
        }
    }

    fn clear_display(&self) {}

    fn print_display(&self) {
        for line in self.display {
            let mut shift = 63;
            println!();

            while shift > 0 {
                let bit = (line >> shift) & 0x1;

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
        let op_code = code >> 12;
        let adrr = code >> 4;
        let nibble = code & 0xf;
        let kk = (code & 0xff) as u8;
        let x = (code >> 8) & 0xf;
        let y = (code >> 4) & 0xf;
        let vx = self.v(x);
        let vy = self.v(y);

        self.pc += 2;

        match op_code {
            0 => todo!(),
            1 => self.pc = adrr,
            2 => todo!(),
            3 => {
                if vx == kk {
                    self.pc += 2
                }
            }
            4 => {
                if vx != kk {
                    self.pc += 2
                }
            }
            5 => {
                if vx == vy {
                    self.pc += 2
                }
            }
            6 => self.set_v(x, kk),
            7 => self.set_v(x, vx + kk),
            8 => match nibble {
                0 => self.set_v(x, vy),
                1 => self.set_v(x, vx | vy),
                2 => self.set_v(x, vx & vy),
                3 => self.set_v(x, vx ^ vy),
                4 => {
                    let sum = vx as u16 + vy as u16;
                    self.set_v(x, sum as u8);
                    self.set_v(0xf, (sum > 255) as u8);
                }
                5 => {
                    self.set_v(0xf, (vx >= vy) as u8);
                    self.set_v(x, vx.wrapping_sub(vy));
                }
                6 => {
                    self.set_v(0xf, (vx & 0x1 == 1) as u8);
                    self.set_v(x, vx >> 1);
                }
                7 => {
                    self.set_v(0xf, (vy >= vx) as u8);
                    self.set_v(x, vy.wrapping_sub(vx));
                }
                0xE => {
                    self.set_v(0xf, ((vx >> 7) & 0x1 == 1) as u8);
                    self.set_v(x, vx << 1);
                }
                _ => println!("Invalid instruction code: {op_code}"),
            },
            9 => {
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
                Chip8::load_sprite_data(&mut self.display, (vx, vy), sprite_data); // todo pass vf
                self.print_display();
            }
            0xE => todo!(),
            0xF => todo!(),
            _ => println!("Invalid instruction code: {op_code}"),
        }
        println!("Counter is {}", self.pc)
    }
}

#[cfg(test)]
mod test {
    use crate::Chip8;

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
