use std::env;

use cpu::CPU;
use sdl2::event::{self, Event, EventType};

mod display;
mod fonts;
mod cpu;

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;
const SCALE: u16 = 35;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    let path = &args[1];
    
    let mut display = display::Display::create(SCREEN_WIDTH, SCREEN_HEIGHT, SCALE)?;
    let mut processor = cpu::CPU::init(&path, display.get_window_size())?;
    let keypad = [0 as u8; 17];

    loop {
        let state: &mut CPU = processor.tick(&keypad)?;

        if state.vram_updated {
            display.draw(&state.vram)?;
        }

        let event = match display.poll() {
            Some(event) => event,
            None => Event::Unknown { timestamp: 0, type_: 0 },
        };

        match event {
            Event::Quit { .. } => break,
            _ => ()
        }
    }

    Ok(())
}