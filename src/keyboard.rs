use std::collections::HashSet;

use sdl2::keyboard::Scancode;

pub struct Keyboard {
    wait_for_press: bool, 
    key: Option<Scancode>,
    pressed_keys: HashSet<Scancode>,
}

pub fn create() -> Keyboard {
    Keyboard{pressed_keys: HashSet::new(), key: None, wait_for_press: false}
}

impl Keyboard {
    pub fn key_down(&mut self, scancode: Scancode) -> bool {
        self.wait_for_press = false;
        self.key = Some(map_key(scancode));
        self.pressed_keys.insert(map_key(scancode))
    }
    pub fn wait_for_press(&mut self) -> bool {
        self.key = None;
        self.wait_for_press = true;
        true
    }
    pub fn is_waiting_for_press(&self) -> bool {
        self.wait_for_press
    }
    pub fn key_up(&mut self, scancode: Scancode) -> bool {
        self.key.is_some_and(|x| x.eq(&map_key(scancode))).then(|| self.key = None);
        self.pressed_keys.remove(&map_key(scancode))
    }
    pub fn is_key_down(&mut self, scancode: Scancode) -> bool {
        self.pressed_keys.contains(&map_key(scancode))
    }
    pub fn get_recently_pressed_key(&mut self) -> Option<Scancode> {
        self.key
    }
}

fn map_key(key: Scancode) -> Scancode {
    match key {
        Scancode::Num4 => Scancode::C,
        Scancode::Q => Scancode::Num4,
        Scancode::W => Scancode::Num5,
        Scancode::E => Scancode::Num6,
        Scancode::R => Scancode::D,
        Scancode::A => Scancode::Num7,
        Scancode::S => Scancode::Num8,
        Scancode::D => Scancode::Num9,
        Scancode::F => Scancode::E,
        Scancode::Z => Scancode::A,
        Scancode::X => Scancode::Num0,
        Scancode::C => Scancode::B,
        Scancode::V => Scancode::F,
        _ => Scancode::Escape
    }
}