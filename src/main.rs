use core::time;
use std::env;
use std::io::Read;
use std::thread;
use std::fs;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

const SCREEN_WIDTH: u16 = 64;
const SCREEN_HEIGHT: u16 = 32;
const REFRESH_RATE: f32 = 1.0 / 60.0;
const SCALE: u16 = 40;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    let path = &args[1];

    let mut font: Vec<u8> = Vec::from([
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

    let mut pixels: Vec<Vec<u8>> = vec![vec![0; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize];
    let mut ram = vec![0; 4096];

    for (pos, e) in font.iter().enumerate() { 
        ram[pos] = *e;   
    }

    let mut pc: u16 = 0x200; 

    ram.resize(pc as usize, 0);

    let mut i: u16 = 0;
    let mut delay_timer: u8 = 255;
    let mut sound_timer: u8 = 255;
    let mut v_registers: Vec<u8> = vec![0; 16];
    let stack: Vec<u16> = Vec::new();
    
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("CHIP-8", (SCREEN_WIDTH * SCALE) as u32, (SCREEN_HEIGHT * SCALE) as u32)
        .fullscreen()
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut event_pump = sdl
        .event_pump()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .read_to_end(&mut ram)
        .map_err(|e| e.to_string())?;

    'render: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'render,
                _ => (),
            }
        }

        let high = ram[pc as usize];
        let low = ram[(pc + 1) as usize];
        pc += 2;

        match high >> 4 {
            0x0 => {
                match low {
                    0xE0 => canvas.clear(),
                    _ => ()
                }
            },
            0x1 => {
                pc = u16::from(high & 0x0F) << 8 | u16::from(low);
            },
            0x6 => {
                let register = high & 0x0F;
                v_registers[register as usize] = low;
            }
            0x7 => {
                let register = high & 0x0F; 
                v_registers[register as usize] += low;            
            },
            0xA => {
                i = u16::from(high & 0x0F) << 8 | u16::from(low);
            },
            0xD => {
                let mut y = u16::from(v_registers[(low >> 4) as usize]) % SCREEN_HEIGHT;
                
                v_registers[0xF] = 0;
                
                let rows = u16::from(low & 0x0F); 
                
                for pos in i..(i + rows) {
                    let mut x = u16::from(v_registers[(high & 0x0F) as usize]) % SCREEN_WIDTH; 
                    
                    if y == SCREEN_HEIGHT {
                        break;
                    }

                    let sprite = ram[pos as usize];

                    for n in (0..8).rev() {
                        if x == SCREEN_WIDTH {
                            break;
                        }
                     
                        let bit = (sprite >> n) & 1;
                        
                        let color = match bit ^ pixels[x as usize][y as usize] {
                            1 => 255,
                            0 => 0,
                            _ => 0,
                        };
                        
                        v_registers[0xF] = u8::from(bit & pixels[x as usize][y as usize]);
                        
                        pixels[x as usize][y as usize] ^= bit;
                        
                        canvas.set_draw_color(Color::RGB(color, color, color));
                        canvas.fill_rect(Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE as u32, SCALE as u32))?;
                        
                        x += 1;
                    }
                    y += 1;
                }
            }
            _ => (),
        }

        canvas.present();
        thread::sleep(time::Duration::from_secs_f32(REFRESH_RATE));
    }


    Ok(())
}

fn get_n_bits(n: u16, from: u16, length: u16) -> u16 {
    (n & ((1 << length) - 1) << from) >> from
}
