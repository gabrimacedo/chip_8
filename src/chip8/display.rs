use minifb::{Error, Window, WindowOptions};

pub struct Display {
    pub vram: [u64; 32],
    draw_flag: bool,
    framebuffer: Vec<u32>,
    buf_width: usize,
    buf_height: usize,
    window: Window,
    on_color: u32,
    off_color: u32,
}

impl Display {
    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }

    /// TODO: add wrapping configuration option
    pub fn draw_to_vram(&mut self, (x, y): (u8, u8), vf: &mut u8, sprite: &[u8]) {
        *vf = 0; // reset vf flag
        let x = x % 64;
        let y = y % 32;

        for (row, sprite_byte) in sprite.iter().enumerate() {
            let data = ((*sprite_byte as u64) << 56) >> x;

            let row_y = (y as usize) + row;
            if row_y > 31 {
                break;
            }
            let line = &mut self.vram[row_y];

            // check collision
            if data & *line != 0 {
                *vf = 1
            }

            *line ^= data;
        }
        self.draw_flag = true;
    }

    pub fn clear_vram(&mut self) {
        self.vram.iter_mut().for_each(|line| *line = 0);
        self.draw_flag = true;
    }

    /// TODO: pass scale as configuration
    pub fn render(&mut self) {
        let scale = 10;

        if self.draw_flag {
            for (row, num) in self.vram.iter().enumerate() {
                for shift in (0..64).rev() {
                    let bit = num >> shift & 0x1;

                    for i in 0..scale {
                        let x = ((63 - shift) * scale) + i;

                        for j in 0..scale {
                            let y = (row * scale + j) * self.buf_width;

                            self.framebuffer[x + y] = if bit != 0 {
                                self.on_color
                            } else {
                                self.off_color
                            }
                        }
                    }
                }
            }

            self.draw_flag = false;
        }

        if let Err(e) =
            self.window
                .update_with_buffer(&self.framebuffer, self.buf_width, self.buf_height)
        {
            eprintln!("Display update failed: {}", e);
        }
    }
}

impl Display {
    pub fn build(buf_width: usize, buf_height: usize) -> Result<Self, Error> {
        let window = Window::new("Main", buf_width, buf_height, WindowOptions::default())?;

        Ok(Display {
            vram: [0; 32],
            framebuffer: vec![0; buf_width * buf_height],
            buf_width,
            buf_height,
            draw_flag: false,
            window,
            on_color: Display::u8_to_rgb(255, 255, 255),
            off_color: Display::u8_to_rgb(0, 0, 0),
        })
    }

    fn u8_to_rgb(r: u8, g: u8, b: u8) -> u32 {
        let mut rgb: u32 = 0;
        rgb |= (r as u32) << 16;
        rgb |= (g as u32) << 8;
        rgb |= b as u32;

        rgb
    }
}
