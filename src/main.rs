use std::{env, thread, time::Duration};

use cpu::CPU;
use audio::Audio;
use keypad::map_scancode;
use sdl2::event::Event;

mod display;
mod fonts;
mod cpu;
mod keypad;
mod audio;

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;
const SCALE: u16 = 35;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    let path = &args[1];

    let sdl = sdl2::init()?;

    let mut display = display::Display::create(&sdl, (SCREEN_WIDTH, SCREEN_HEIGHT, SCALE))?;
    let audio = Audio::create(&sdl)?;
    let mut processor = cpu::CPU::init(&path, display.get_window_size())?;
    let mut keypad: [u8; 17] = [0; 17];
    let mut key: Option<u8> = None;

    'main: loop {
        let cpu: &mut CPU = processor.tick(&keypad, key)?;
        
        cpu.delay();

        if cpu.sound_timer > 0 {
            audio.play_audio(cpu.sound_timer);
            cpu.reset_sound_timer();
        }

        if cpu.vram_updated {
            display.draw(&cpu.vram)?;
        }

        for event in display.poll() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { scancode, .. } => {
                    if let Some(scancode) = scancode {
                        let code = map_scancode(scancode);
                        keypad[code as usize] = 1;
                        key = Some(code);
                    }
                },
                Event::KeyUp { scancode , .. } => {
                    if let Some(scancode) = scancode {
                        let code = map_scancode(scancode);
                        keypad[code as usize] = 0;
                        key.is_some_and(|x| x == code).then(|| key = None);
                    }
                },
                _ => ()
            }
        }
    }

    Ok(())
}