use core::time;
use std::env;
use std::io::Read;
use std::thread;
use std::fs;

use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::{event::Event};

const SCREEN_WIDTH: u16 = 64;
const SCREEN_HEIGHT: u16 = 32;
const REFRESH_RATE: f32 = 1.0 / 60.0;
fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    let path = &args[1];

    let mut ram: Vec<u8> = Vec::from([
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ]);
    
    let mut pc: usize = ram.len();
    let mut i: u16 = 0;
    let mut delay_timer: u8 = 255;
    let mut sound_timer: u8 = 255;
    let mut v_registers: Vec<u8> = vec![0; 16];
    let stack: Vec<u16> = Vec::new();

    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let mut event_pump = sdl_context.event_pump()?;

    let window = video_subsystem
        .window("CHIP-8", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .read_to_end(&mut ram)
        .map_err(|e| e.to_string())?;

    ram.resize(4096, 0);
    
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    
    'render: while delay_timer > 0 && sound_timer > 0 {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'render,
                _ => (),
            }
        }

        let instruction = u16::from((u16::from(ram[pc]) << 8) | u16::from(ram[pc + 1]));
        pc += 2;

        match get_n_bits(instruction, 12, 4) {
            0x0 => {
                match get_n_bits(instruction, 0, 12) {
                    0xE0 => canvas.clear(),
                    _ => ()
                }
            },
            0x1 => {
                pc = get_n_bits(instruction, 0, 12) as usize;
            },
            0x6 => {
                let register = get_n_bits(instruction, 8, 4);
                let value = get_n_bits(instruction, 0, 8);
                v_registers[register as usize] = value as u8; 
            }
            0x7 => {
                let register = get_n_bits(instruction, 8, 4);
                let value = get_n_bits(instruction, 0, 8);
                v_registers[register as usize] = value as u8;            
            },
            0xA => {
                let value = get_n_bits(instruction, 0, 12);
                println!("{}", value);
                i = value % 80;
            },
            0xD => {
                let mut x = get_n_bits(instruction, 8, 4) % SCREEN_WIDTH;
                let mut y = get_n_bits(instruction, 4, 4) % SCREEN_HEIGHT;

                v_registers[0xF] = 0;

                canvas.set_draw_color(Color::RGB(255, 255, 255));

                let n = get_n_bits(instruction, 0, 4);

                for row in 0..n {
                    if y > SCREEN_HEIGHT {
                        break;
                    }
                    let sprite = ram[usize::from(i + row)];
                    for pos in 0..7 {
                        if x > SCREEN_WIDTH {
                            break;
                        }
                        let point = Point::new(x as i32, y as i32);
                        canvas.draw_point(point)?;
                        x += 1;
                    }
                    y += 1; 
                }
            }
            _ => (),
        }

        canvas.present();
        delay_timer -= 1;
        sound_timer -= 1;
        thread::sleep(time::Duration::from_secs_f32(REFRESH_RATE));
    }


    Ok(())
}

fn get_n_bits(n: u16, from: u16, length: u16) -> u16 {
    (n & ((1 << length) - 1) << from) >> from
}
