use std::fs::File;
use std::io::Read;

use crate::display::Sprite;

use super::display::Display;
use super::emulator::Emulator;

pub struct Chip8 {
    display: Display,
    ram: Vec<u8>,
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    v_registers: Vec<u8>,
    stack: Vec<u16>,
}

enum Chip8Instructions {
    CLS,
    JPAddr,
    LDVx,
    ADDVx,
    LDI,
    DRW,
    Unknown,
}

pub fn init(display: Display) -> Result<Chip8, String> {
    let ram = vec![0; 4096];
    let pc: u16 = 0x200; 
    let i: u16 = 0;
    let delay_timer: u8 = 255;
    let sound_timer: u8 = 255;
    let v_registers: Vec<u8> = vec![0; 16];
    let stack: Vec<u16> = Vec::new();

    Ok(Chip8{ ram, pc, i, delay_timer, sound_timer, v_registers, stack, display })
}

impl Emulator for Chip8 {
    fn load_fonts(&mut self, fonts: &[u8]) -> Result<&mut Self, String> {
        for (pos, e) in fonts.iter().enumerate() { 
            self.ram[pos] = *e;   
        }
        self.ram.resize(self.pc as usize, 0);
        Ok(self)
    }
    fn load_rom(&mut self, path: &str) -> Result<&mut Self, String> {
        File::open(path)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut self.ram)
            .map_err(|e| e.to_string())?;
        Ok(self)
    }
    fn run(&mut self) -> Result<(), String> {
            self.display.start_render(|renderer| {
                let high = self.ram[self.pc as usize];
                let low = self.ram[(self.pc + 1) as usize];
                let instruction = parse_instruction(high, low);

                self.pc += 2;

                match instruction {
                    Chip8Instructions::CLS => renderer.clear(),
                    Chip8Instructions::JPAddr => self.pc = u16::from(high & 0x0F) << 8 | u16::from(low),
                    Chip8Instructions::LDVx => {
                        let register = high & 0x0F;
                        self.v_registers[register as usize] = low;
                    }
                    Chip8Instructions::ADDVx => {
                        let register = high & 0x0F; 
                        self.v_registers[register as usize] += low;            
                    },
                    Chip8Instructions::LDI => self.i = u16::from(high & 0x0F) << 8 | u16::from(low),
                    Chip8Instructions::DRW => {
                        let x = u16::from(self.v_registers[(high & 0x0F) as usize]);  
                        let y = u16::from(self.v_registers[(low >> 4) as usize]);
                        self.v_registers[0xF] = 0;
                        let rows = u16::from(low & 0x0F);
                        let sprite = Sprite{ x: x as i32, y: y as i32, len: rows, data: &self.ram, addr: self.i };
                        renderer.write_sprite(sprite, self.v_registers.get_mut(0xF).unwrap()).unwrap();
                    }
                    _ => (),
                }

                renderer.refresh();    
            })?; 
            Ok(())
    }
}

fn parse_instruction(high: u8, low: u8) -> Chip8Instructions {
        match high >> 4 {
            0x0 => {
                match low {
                    0xE0 => Chip8Instructions::CLS, 
                    _ => Chip8Instructions::Unknown, 
                }
            },
            0x1 => Chip8Instructions::JPAddr,
            0x6 => Chip8Instructions::LDVx,
            0x7 => Chip8Instructions::ADDVx,
            0xA => Chip8Instructions::LDI,
            0xD => Chip8Instructions::DRW, 
            _ => Chip8Instructions::Unknown,
        }
}