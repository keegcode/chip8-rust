use sdl2::keyboard::Scancode;

pub fn map_scancode(code: Scancode) -> u8 {
    match code {
        Scancode::Num1 => 1,
        Scancode::Num2 => 2,
        Scancode::Num3 => 3,
        Scancode::Num4 => 12,
        Scancode::Q => 4,
        Scancode::W => 5,
        Scancode::E => 6,
        Scancode::R => 13,
        Scancode::A => 7,
        Scancode::S => 8,
        Scancode::D => 9,
        Scancode::F => 14,
        Scancode::Z => 10,
        Scancode::X => 0,
        Scancode::C => 11,
        Scancode::V => 15,
        _ => 0
    }
}