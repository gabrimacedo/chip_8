use minifb::{Key, Window};

#[derive(Default)]
pub struct Input {
    keys: [bool; 16],
}

impl Input {
    pub fn poll_input(&mut self, window: &mut Window) {
        window.update();
        self.update_keys(window);
    }

    pub fn is_key_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    pub fn get_any_pressed_key(&self) -> Option<u8> {
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
}
