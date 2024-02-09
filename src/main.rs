use core::time;

use sdl2::{event::Event, pixels::Color, rect::Rect};

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("CHIP-8", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut surface = window.surface(&event_pump)?;
    
    surface
        .fill_rect(
            Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT), 
            Color::RGB(251, 124, 4)
        )?;

    surface.update_window()?;

    'render: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'render,
                _ => (),
            }
        }

        std::thread::sleep(time::Duration::from_secs(5));
    }

    Ok(())
}
