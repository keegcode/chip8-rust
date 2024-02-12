use std::env;

use emulator::Emulator;

mod display;
mod emulator;
mod chip8;
mod fonts;

const SCREEN_WIDTH: u16 = 64;
const SCREEN_HEIGHT: u16 = 32;
const SCALE: u16 = 40;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    let path = &args[1];
     
    let display = display::create(SCREEN_WIDTH, SCREEN_HEIGHT, SCALE)?;
    let mut chip8_emulator = chip8::init(display)?;

    chip8_emulator
        .load_fonts(fonts::FONTS)?
        .load_rom(path)?
        .run()?;

    Ok(())
}