fn main() {
    let mut processor = Chip8::new();

    // fetch instruction from memory 0 and forth
    let code = processor.fetch();

    // decode instruction
    processor.decode(code);

    // execute instruction
}

struct Chip8 {
    memory: [u8; 4096],
    registers: [u8; 16],
    pc: u16,
    i: u16,
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            memory: [0; 4096],
            registers: [0; 16],
            i: 0,
            pc: 0,
        }
    }

    fn fetch(&self) -> u16 {
        let high = self.memory[self.pc as usize] as u16;
        let low = self.memory[(self.pc + 1) as usize];

        (high << 2) + (low as u16)
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
            1 => todo!(),
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
            8 => {
                match nibble {
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
                    // Set Vx = Vx SHR 1
                    6 => {}
                    // Set Vx = Vy - Vx, set VF = NOT borrow
                    7 => {}
                    // Set Vx = Vx SHL 1
                    0xE => {}
                    _ => todo!(),
                }
            }
            9 => todo!(),
            10 => todo!(),
            11 => todo!(),
            12 => todo!(),
            13 => todo!(),
            14 => todo!(),
            15 => todo!(),
            _ => todo!(),
        }
        println!("Counter is {}", self.pc)
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn fetch_correct_code() {}
}
