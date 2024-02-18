use std::{thread, time::Duration};

use sdl2::audio::{AudioCallback, AudioSpecDesired};

const FREQUENCY: i32 = 44100;

pub struct Audio {
    device: sdl2::audio::AudioDevice<SquareWave>,
}

impl Audio {
    pub fn create(sdl: &sdl2::Sdl) -> Result<Audio, String> {
        let audio = sdl.audio()?; 
        let audio_spec= AudioSpecDesired{
            freq: Some(FREQUENCY),
            samples: Some(2048),
            channels: Some(1),
        };
        
        let device = audio
        .open_playback(None, &audio_spec, |spec| {
            SquareWave {
                phase_inc: 240.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        })
        .map_err(|err| err.to_string())?;

        Ok(Audio{device})
    }
    pub fn play_audio(&self, mut duration: u8) {
        while duration > 0 {
            self.device.resume();
            duration -= 1;
            thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
        }
        self.device.pause();
    }
}

pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = self.volume * if self.phase < 0.5 { 1.0 } else { -1.0 };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}