use std::fs::File;
use std::io::Read;
use std::ops::{Div, Not};

use rand;
use sdl2::keyboard::Scancode;

use super::display::{Display, Sprite};
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
    RET,
    CALL,
    JPAddr,
    LDVxByte,
    LDVxVy,
    ORVxVy,
    XORVxVy,
    ANDVxVy,
    ADDVxVy,
    SUBVxVy,
    SHRVxVy,
    SUBNVxVy,
    SHLVxVy,
    ADDVx,
    LDI,
    JPV0,
    RNDVx,
    DRW,
    SKPVx,
    SKNPVx,
    SEVxByte,
    SNEVxByte,
    SEVxVy,
    SNEVxVy,
    LDVxK,
    ADDIVx,
    LDFVx,
    LDBVx,
    LDIVx,
    LDVxI,
    Unknown,
}

pub fn init(display: Display) -> Result<Chip8, String> {
    let pc: u16 = 0x200; 
    let ram = vec![0; pc as usize];
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
        self.ram.resize(4096 as usize, 0);
        Ok(self)
    }
    fn run(&mut self) -> Result<(), String> {
            self.display.start_render(|renderer, keyboard| {
                let high = self.ram[self.pc as usize];
                let low = self.ram[(self.pc + 1) as usize];
                let instruction = parse_instruction(high, low);

                self.pc += 2;

                match instruction {
                    Chip8Instructions::CLS => {
                       renderer.clear();
                    }
                    Chip8Instructions::RET => { 
                        self.pc = self.stack.pop().unwrap();
                    }
                    Chip8Instructions::CALL => {
                        self.stack.push(self.pc);
                        self.pc = u16::from(high & 0x0F) << 8 | u16::from(low);
                    },
                    Chip8Instructions::JPAddr => {
                        self.pc = u16::from(high & 0x0F) << 8 | u16::from(low);
                    },
                    Chip8Instructions::LDVxByte => {
                        let register = high & 0x0F;
                        self.v_registers[register as usize] = low;
                    },
                    Chip8Instructions::LDVxVy => { 
                        self.v_registers[(high & 0x0F) as usize] = self.v_registers[(low >> 4) as usize];
                    },
                    Chip8Instructions::ORVxVy => {
                        self.v_registers[(high & 0x0F) as usize] |= self.v_registers[(low >> 4) as usize];
                    },
                    Chip8Instructions::ANDVxVy => {
                        self.v_registers[(high & 0x0F) as usize] &= self.v_registers[(low >> 4) as usize];
                    },
                    Chip8Instructions::XORVxVy => {
                        self.v_registers[(high & 0x0F) as usize] ^= self.v_registers[(low >> 4) as usize];
                    },
                    Chip8Instructions::ADDVxVy => {
                        let x = self.v_registers[(high & 0x0F) as usize];
                        let y = self.v_registers[(low >> 4) as usize];
                        
                        match x.checked_add(y) {
                            Some(n) => {
                                self.v_registers[(high & 0x0F) as usize] = n;
                                self.v_registers[0xF] = 0;
                            },
                            None => {
                                self.v_registers[(high & 0x0F) as usize] = x.wrapping_add(y);
                                self.v_registers[0xF] = 1; 
                            } 
                        } 
                    },
                    Chip8Instructions::SUBVxVy => {
                        let x = self.v_registers[(high & 0x0F) as usize];
                        let y = self.v_registers[(low >> 4) as usize];
                        
                        self.v_registers[(high & 0x0F) as usize] = x.wrapping_sub(y);
                        self.v_registers[0xF] = u8::from(x >= y);
                    },
                    Chip8Instructions::SHRVxVy => {
                        let x = self.v_registers[(high & 0x0F) as usize];
                        self.v_registers[0xF] = self.v_registers[(high & 0x0F) as usize] & 1;
                        self.v_registers[(high & 0x0F) as usize] = x.wrapping_shr(1);
                    }, 
                    Chip8Instructions::SUBNVxVy => {
                        let x = self.v_registers[(high & 0x0F) as usize];
                        let y = self.v_registers[(low >> 4) as usize];
                        
                        self.v_registers[(high & 0x0F) as usize] = y.wrapping_sub(x);
                        self.v_registers[0xF] = u8::from(y >= x);
                    },
                    Chip8Instructions::SHLVxVy => {
                        let x = self.v_registers[(high & 0x0F) as usize];
                        self.v_registers[0xF] = (self.v_registers[(high & 0x0F) as usize] >> 7) & 1;
                        self.v_registers[(high & 0x0F) as usize] = x.wrapping_shl(1);
                    },
                    Chip8Instructions::SNEVxVy => {
                        let x = self.v_registers[(high & 0x0F) as usize];
                        let y = self.v_registers[(low >> 4) as usize];
                        
                        self.pc += 2*u16::from(x != y);  
                    },
                    Chip8Instructions::ADDVx => {
                        let register = high & 0x0F; 
                        self.v_registers[register as usize] = self.v_registers[register as usize].wrapping_add(low);       
                    },
                    Chip8Instructions::LDI => {
                        self.i = u16::from(high & 0x0F) << 8 | u16::from(low);
                    },
                    Chip8Instructions::JPV0 => {
                        let v0 = self.v_registers[0];
                        let addr = u16::from(high & 0x0F) << 8 | u16::from(low);
                        self.pc = addr.wrapping_add(u16::from(v0));
                    },
                    Chip8Instructions::RNDVx => {
                        let x = high & 0x0F;
                        let r = rand::random::<u8>();
                        self.v_registers[x as usize] = r & low;
                    },
                    Chip8Instructions::DRW => {
                        let x = u16::from(self.v_registers[(high & 0x0F) as usize]);  
                        let y = u16::from(self.v_registers[(low >> 4) as usize]);
                        
                        self.v_registers[0xF] = 0;
                        
                        let rows = u16::from(low & 0x0F);
                        let sprite = Sprite{ x: x as i32, y: y as i32, len: rows, data: &self.ram, addr: self.i };
                        
                        renderer.write_sprite(sprite, self.v_registers.get_mut(0xF).unwrap()).unwrap();
                    },
                    Chip8Instructions::SEVxByte  => {
                        self.pc += 2 * u16::from(low == self.v_registers[(high & 0x0F) as usize]);
                    },
                    Chip8Instructions::SNEVxByte => {
                        self.pc += 2 * u16::from(low != self.v_registers[(high & 0x0F) as usize]);
                    },
                    Chip8Instructions::SEVxVy => {
                        self.pc += 2 * u16::from(self.v_registers[(high & 0x0F) as usize] == self.v_registers[(low >> 4) as usize]);
                    },
                    Chip8Instructions::SKPVx => {
                        let x = high & 0x0F;
                        let scancode = self.v_registers[x as usize]; 
                        self.pc += 2 * u16::from(keyboard.is_key_down(Scancode::from_i32(scancode as i32).unwrap()));
                    },
                    Chip8Instructions::SKNPVx => {
                        let x = high & 0x0F;
                        let scancode = self.v_registers[x as usize]; 
                        self.pc += 2 * u16::from(keyboard.is_key_down(Scancode::from_i32(scancode as i32).unwrap()).not());
                    }
                    Chip8Instructions::LDVxK => {
                        match keyboard.get_recently_pressed_key() {
                            Some(scancode) => {
                                let x = high & 0x0F;
                                self.v_registers[x as usize] = scancode as u8; 
                            },
                            None => {
                                self.pc -= 2;
                                keyboard.wait_for_press();
                            }
                        }
                    }
                    Chip8Instructions::ADDIVx => {
                        let x = high & 0x0F;
                        self.i += u16::from(self.v_registers[x as usize]);
                    }
                    Chip8Instructions::LDFVx => {
                        let x = high & 0x0F;
                        self.i = u16::from(x * 5);
                    }
                    Chip8Instructions::LDBVx => {
                        let x = self.v_registers[(high & 0x0F) as usize];
                        let i = self.i as usize;
                        self.ram[i] = x / 100;
                        self.ram[i + 1] = (x / 10) % 10;
                        self.ram[i + 2] = (x % 100) % 10;
                    }
                    Chip8Instructions::LDIVx => {
                        let x = high & 0x0F;
                        let i = self.i;
                        for pos in 0..=x {
                            self.ram[(i + pos as u16) as usize] = self.v_registers[pos as usize];
                        }
                    }
                    Chip8Instructions::LDVxI => {
                        let x = high & 0x0F;
                        let i = self.i;
                        for pos in 0..=x {
                            self.v_registers[pos as usize] = self.ram[(i + pos as u16) as usize]; 
                        }
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
            0x0 => match low {
                0xE0 => Chip8Instructions::CLS, 
                0xEE => Chip8Instructions::RET,
                _ => Chip8Instructions::Unknown, 
            },
            0x1 => Chip8Instructions::JPAddr,
            0x2 => Chip8Instructions::CALL,
            0x3 => Chip8Instructions::SEVxByte,
            0x4 => Chip8Instructions::SNEVxByte,
            0x5 => Chip8Instructions::SEVxVy,
            0x6 => Chip8Instructions::LDVxByte,
            0x7 => Chip8Instructions::ADDVx,
            0x8 => match low & 0x0F {
                0x0 => Chip8Instructions::LDVxVy,
                0x1 => Chip8Instructions::ORVxVy, 
                0x2 => Chip8Instructions::ANDVxVy,
                0x3 => Chip8Instructions::XORVxVy,
                0x4 => Chip8Instructions::ADDVxVy,
                0x5 => Chip8Instructions::SUBVxVy,
                0x6 => Chip8Instructions::SHRVxVy,
                0x7 => Chip8Instructions::SUBNVxVy,
                0xE => Chip8Instructions::SHLVxVy,
                _ => Chip8Instructions::Unknown,
            }, 
            0x9 => Chip8Instructions::SNEVxVy,
            0xA => Chip8Instructions::LDI,
            0xB => Chip8Instructions::JPV0,
            0xC => Chip8Instructions::RNDVx,
            0xD => Chip8Instructions::DRW, 
            0xE => match low {
                0x9E => Chip8Instructions::SKPVx, 
                0xA1 => Chip8Instructions::SKNPVx, 
                _ => Chip8Instructions::Unknown,
            }, 
            0xF => match low {
                0x0A => Chip8Instructions::LDVxK,
                0x1E => Chip8Instructions::ADDIVx,
                0x29 => Chip8Instructions::LDFVx,
                0x33 => Chip8Instructions::LDBVx,
                0x55 => Chip8Instructions::LDIVx,
                0x65 => Chip8Instructions::LDVxI,
                _ => Chip8Instructions::Unknown,
            }
            _ => Chip8Instructions::Unknown,
        }
}