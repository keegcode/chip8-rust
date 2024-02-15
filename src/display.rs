use std::thread::yield_now;

use sdl2::{pixels::Color, rect::Rect, Sdl, event::Event};

pub struct Display {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
    scale: u16,
    height: u8,
    width: u8,
}

impl Display {
    pub fn create(width: u8, height: u8, scale: u16) -> Result<Display, String> {
            let sdl = sdl2::init()?;
            let video = sdl.video()?;

            let window = video
                .window("", (width as u16 * scale) as u32, (height as u16 * scale) as u32)
                .position_centered()
                .build()
                .map_err(|e| e.to_string())?;
        
            let event_pump = sdl
                .event_pump()
                .map_err(|e| e.to_string())?;
        
            let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

            Ok(Display{canvas, event_pump, scale, height, width}) 
    }
    pub fn poll(&mut self) -> Option<Event> {
        self.event_pump.poll_event()
    }
    pub fn draw(&mut self, vram: &Vec<Vec<u8>>)-> Result<(), String> {
        for (y, row) in vram.iter().enumerate() {
            for (x, col)  in row.iter().enumerate() {
                self.canvas.set_draw_color(col.eq(&1).then(|| Color::WHITE).unwrap_or(Color::BLACK));
                self.canvas.fill_rect(Rect::new((x * self.scale as usize) as i32, (y * self.scale as usize) as i32, self.scale as u32, self.scale as u32))?;
            }
        }
        self.canvas.present();
        Ok(())
    }
    pub fn get_window_size(&self) -> (u8, u8) {
        (self.width, self.height)
    }
}



