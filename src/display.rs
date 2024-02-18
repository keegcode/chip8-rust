use sdl2::{event::{Event, EventPollIterator}, pixels::Color, rect::Rect};

pub struct Display {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
    scale: u16,
    height: u8,
    width: u8,
}

impl Display {
    pub fn create(sdl: &sdl2::Sdl, (width, height, scale): (u8, u8, u16)) -> Result<Display, String> {
            let video = sdl.video()?;

            let window = video
                .window("", (width as u16 * scale) as u32, (height as u16 * scale) as u32)
                .position_centered()
                .build()
                .map_err(|e| e.to_string())?;
        
            let event_pump = sdl
                .event_pump()
                .map_err(|e| e.to_string())?;
        
            let canvas = window.into_canvas().present_vsync().build().map_err(|e| e.to_string())?;

            Ok(Display{canvas, event_pump, scale, height, width}) 
    }
    pub fn poll(&mut self) -> EventPollIterator {
        self.event_pump.poll_iter()
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
    pub fn clear(&mut self) -> () {
        self.canvas.clear()
    }
}



