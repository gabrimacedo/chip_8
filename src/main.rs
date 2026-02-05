fn main() {
    // reserve the 4kb of ram
    let mut processor = Chip8::new();

    // let counter = &memory[512];

    // fetch instruction from memory 0 and forth
    // read 2 adressses from counter pointer
    // let a = *counter;
    // counter = &memory[513];
    // let b = *counter;
    // let ab: u16 = a << 8;

    // decode instruction
    processor.decode(0x8123);

    // execute instruction
}

struct Chip8 {
    memory: Vec<u8>,
    registers: Vec<u8>,
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            memory: vec![0; 4096],
            registers: vec![0; 15],
        }
    }

    fn v(&self, reg: u16) -> u8 {
        self.registers[reg as usize]
    }
    fn set_v(&mut self, reg: u16, value: u8) {
        self.registers[reg as usize] = value;
    }

    fn decode(&mut self, code: u16) {
        let op_code = code >> 12;
        let x = (code >> 8) & 0xf;
        let y = (code >> 4) & 0xf;
        let adrr = code >> 4;
        let nibble = code & 0xf;
        let kk = (code & 0xff) as u8;

        let mut counter = 2;

        match op_code {
            0 => todo!(),
            1 => todo!(),
            2 => todo!(),
            3 => {
                if self.v(x) == kk {
                    counter += 2
                }
            }
            4 => {
                if self.v(x) != kk {
                    counter += 2
                }
            }
            5 => {
                if self.v(x) == self.v(y) {
                    counter += 2
                }
            }
            6 => todo!(),
            7 => todo!(),
            8 => {
                match nibble {
                    0 => self.set_v(x, self.v(y)),

                    1 => {
                        // set Vx = Vx OR Vy
                    }
                    2 => {
                        // set Vx = Vx AND Vy
                    }
                    3 => {
                        // set Vx = Vx XOR Vy
                    }
                    4 => {
                        // set Vx = Vx + Vy, set VF = carry
                    }
                    5 => {
                        // Set Vx = Vx - Vy, set VF = NOT borrow
                    }
                    6 => {
                        // Set Vx = Vx SHR 1
                    }
                    7 => {
                        // Set Vx = Vy - Vx, set VF = NOT borrow
                    }
                    0xE => {
                        // Set Vx = Vx SHL 1
                    }
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
        println!("Counter is {counter}")
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn get_op_code() {
        // let code = 0x8123;
        // let codeb = 0xe123;

        // assert_eq!(res, "");
        // assert_eq!(resb, "SIr, its an E code!");
    }
}
