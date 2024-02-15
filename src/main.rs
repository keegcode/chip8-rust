use std::env;

mod display;
mod fonts;
mod keyboard;
mod audio;
mod cpu;

const SCREEN_WIDTH: u16 = 64;
const SCREEN_HEIGHT: u16 = 32;
const SCALE: u16 = 40;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    let path = &args[1];

    let audio = Audio::init();
    let display = Display::init();
    let processor = CPU::init();
    let keyboard = Keyboard::init();

    while {
        let cpu = processor.tick(keyboard)?;


        cpu.start_delay();
        audio.play_sound(state.sound_timer);
        display.redraw_screen(&mut cpu.vram);
    }   

    Ok(())
}