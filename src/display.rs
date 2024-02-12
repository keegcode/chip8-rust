use std::thread;
use std::time;
use sdl2::{pixels::Color, rect::Rect, render::WindowCanvas, Sdl, event::Event};

pub struct Display {
    pub width: u16,
    pub height: u16,
    pub scale: u16,
    sdl: Sdl,
}

pub struct DisplayRenderer {
    pixels: Vec<Vec<u8>>,
    canvas: WindowCanvas,
    width: u16,
    height: u16,
    scale: u16,
}

pub struct Sprite<'a> {
    pub x: i32,
    pub y: i32,
    pub len: u16,
    pub data: &'a [u8],
    pub addr: u16,
}

pub fn create(width: u16, height: u16, scale: u16) -> Result<Display, String> {
        let sdl = sdl2::init()?;
        Ok(Display{width, height, scale, sdl}) 
}

impl Display {
    pub fn start_render<F>(&self, mut f: F) -> Result<(), String> where F: FnMut(&mut DisplayRenderer) {
        let video = self.sdl.video()?;

        let window = video
            .window("", (self.width * self.scale) as u32, (self.height * self.scale) as u32)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        
        let mut event_pump = self.sdl
            .event_pump()
            .map_err(|e| e.to_string())?;
        
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        
        let pixels: Vec<Vec<u8>> = vec![vec![0; self.height as usize]; self.width as usize];
        
        let mut renderer = DisplayRenderer{
            pixels,
            canvas, 
            width: self.width,
            height: self.height,
            scale: self.scale,
        };

        'render: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} => break 'render,
                    _ => (),
                }
            }
            f(&mut renderer);
            thread::sleep(time::Duration::from_secs_f32(1.0 / 60.0));
        }

        Ok(())
    }
}

impl DisplayRenderer {
    pub fn write_sprite(&mut self, sprite: Sprite, collision_register: &mut u8) -> Result<(), String> {
        *collision_register = 0;
        let mut y = sprite.y % self.height as i32;
        for pos in sprite.addr..(sprite.addr + sprite.len) {
            let mut x = sprite.x % self.width as i32; 
                    
            if y == self.height as i32 {
                break;
            }

            let sprite = sprite.data[pos as usize];

            for n in (0..8).rev() {
                if x == self.width as i32 {
                    break;
                }
                     
                let bit = (sprite >> n) & 1;
                        
                let color = match bit ^ self.pixels[x as usize][y as usize] {
                    1 => 255,
                    0 => 0,
                    _ => 0,
                };
                        
                *collision_register = u8::from(bit & self.pixels[x as usize][y as usize]);
                        
                self.pixels[x as usize][y as usize] ^= bit;
                self.canvas.set_draw_color(Color::RGB(color, color, color));
                self.canvas.fill_rect(
                    Rect::new(x * self.scale as i32,
                    y * self.scale as i32 , 
                    self.scale as u32, 
                    self.scale as u32
                ))?;

                x += 1;
            }
            y += 1;
        }
        Ok(())
    }
    pub fn clear(&mut self) -> () {
        self.canvas.clear();
    }
    pub fn refresh(&mut self) -> () {
        self.canvas.present();
    }
}
