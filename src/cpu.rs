use std::{fs, io::Write};

use super::fonts;

pub struct CPU {
    ram: Vec<u8>,
    pub vram: Vec<u8>,
    pub vram_updated: bool,
    i: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pc: usize,
    v: Vec<u8>,
    stack: Vec<usize>,
    pub waiting_for_press: bool,
}
pub enum ProgramCounter {
    Next,
    Return,
    Call(u16),
    Jump(u16),
    Skip,
    Wait,
    Unknown,
}

impl CPU {
    pub fn init(rom: &str) -> Result<CPU, String> {
        let mut cpu = CPU {
            ram: vec![0; 4096],
            vram: vec![],
            vram_updated: false,
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            v: vec![0; 15],
            stack: Vec::new(),
            waiting_for_press: false
        };
        
        for (i, data) in fonts::FONTS.into_iter().enumerate() {
            cpu.ram[i] = data.clone();
        }

        fs::read(rom)
            .map_err(|err| err.to_string())?
            .write_all(&cpu.ram[cpu.pc..])
            .map_err(|err| err.to_string())?;
        
        Ok(cpu)
    }

    pub fn tick(&mut self, keypad: &[u8; 17]) -> Result<&Self, String> {
        let counter = self.execute_op(keypad);

        match counter {
            ProgramCounter::Next => self.next_op(),
            ProgramCounter::Return => self.return_from_stack(),
            ProgramCounter::Call(op) => self.call_op(op),
            ProgramCounter::Jump(op) => self.jump_to_op(op),
            ProgramCounter::Skip => self.skip_op(),
            ProgramCounter::Wait => (),
            _ => ()
        }

        Ok(&self)
    }
    
    pub fn next_op(&mut self) -> () {
        self.pc += 2;
    }

    pub fn return_from_stack(&mut self) -> () {   
        self.pc = self.stack.pop().unwrap()
    }

    pub fn call_op (&mut self, op: u16) -> () {
        self.stack.push(self.pc);
        self.pc = op as usize;
    }

    pub fn jump_to_op(&mut self, op: u16) {
        self.pc = op as usize;
    }
    
    pub fn skip_op(&mut self) {
        self.pc += 2;
    }
    
    pub fn execute_op(&mut self, keypad: &[u8; 17]) -> ProgramCounter {
            let (high, low)= (self.ram[self.pc], self.ram[self.pc + 2]); 
            
            let op = high >> 4; 
            let x: usize = (high & 0x0F).into();
            let y: usize = (low >> 4).into();
            let n = low & 0x0F;
            let nn = low; 
            let nnn = u16::from(high & 0x0F) << 8 | u16::from(low);
            
            match (op, x, n, y) {
                (0x0, 0x0, 0xe, 0x0) => {
                    self.op_cls()
                }
                (0x0, 0x0, 0xe, 0xe) => { 
                    self.op_ret()
                }
                (0x1, _, _, _) => {
                    self.op_jp_addr(nnn)
                },
                (0x2, _, _, _) => {
                    self.op_call_addr()
                },
                (0x3, _, _, _) => {
                    self.op_3xkk(x, nn)
                },
                (0x4, _, _, _) => { 
                    self.op_4xkk(x, nn)
                },
                (0x5, _, _, 0x0) => {
                    self.op_5xy0(x, y)
                },
                (0x6, _, _, _) => {
                    self.op_6xkk(x, nn)
                },
                (0x7, _, _, _) => {
                    self.op_7xkk(x, nn)
                },
                (0x8, _, _, 0x0) => {
                    self.op_8xy0(x, y)
                },
                (0x8, _, _, 0x1) => {
                    self.op_8xy1(x, y)
                },
                (0x8, _, _, 0x2) => {
                    self.op_8xy2(x, y)
                },
                (0x8, _, _, 0x3) => {
                    self.op_8xy3(x, y)
                },
                (0x8, _, _, 0x4) => {
                    self.op_8xy4(x, y)
                },
                (0x8, _, _, 0x5) => {
                    self.op_8xy5(x, y)
                },
                (0x8, _, _, 0x6) => {
                    self.op_8xy6(x, y)
                },
                (0x8, _, _, 0x7) => {
                    self.op_8xy7(x, y)
                },
                (0x8, _, _, 0xe) => {
                    self.op_8xy0(x, y)
                },
                (0x9, _, _, 0) => {
                    self.op_9xy0(x, y)
                },
                (0xa, _, _, _) => {
                    self.op_annn(nnn)
                }
                (0xb, _, _, _) => {
                    self.op_bnnn(nnn)
                }
                (0xc, _, _, _) => {
                    self.op_cxkk(x, nn)
                }
                (0xd, _, _, _) => {
                    self.op_dxyn(x, y, n)
                }
                (0xe, _, 0x9, 0xe) => {
                    self.op_ex9e(x, keypad)
                }
                (0xe, _, 0xa, 0x1) => {
                    self.op_exa1(x, keypad)
                }
                (0xf, _, 0x0, 0x7) => {
                    self.op_fx07(x)
                }
                (0xf, _, 0x0, 0xa) => {
                    self.op_fx0a(x, keypad)
                }
                (0xf, _, 0x1, 0x5) => {
                    self.op_fx15(x)
                },
                (0xf, _, 0x1, 0x8) => {
                    self.op_fx18(x)
                }
                (0xf, _, 0x1, 0xe) => {
                    self.op_fx1e(x)
                }
                (0xf, _, 0x2, 0x9) => {
                    self.op_fx29(x)
                }
                (0xf, _, 0x3, 0x3) => {
                    self.op_fx33(x)
                }
                (0xf, _, 0x5, 0x5) => {
                    self.op_fx55(x)
                }
                (0xf, _, 0x6, 0x5) => {
                    self.op_fx65(x)
                }
                _ => ProgramCounter::Unknown 
            }
    }
    
    fn get_next_op(&self) {}
    
    fn op_cls(&mut self) -> ProgramCounter {
        self.vram.clear();
        self.vram_updated = true;
        ProgramCounter::Next
    }
    
    fn op_ret(&self) -> ProgramCounter {
        ProgramCounter::Return
    }
    
    fn op_call_addr(&self) -> ProgramCounter {
        ProgramCounter::Call
    }
    
    fn op_jp_addr(&self, nnn: u16) -> ProgramCounter {
        ProgramCounter::Jump(nnn)
    }
    
    fn op_3xkk(&self, x: usize, nn: u8) -> ProgramCounter {
        self.v[x].eq(&nn).then(|| ProgramCounter::Skip).unwrap_or(ProgramCounter::Next)
    }

    fn op_4xkk(&self, x: usize, nn: u8) -> ProgramCounter {
        self.v[x].eq(&nn).then(|| ProgramCounter::Next).unwrap_or(ProgramCounter::Skip)
    }

    fn op_5xy0(&self, x: usize, y: usize) -> ProgramCounter {
        self.v[x].eq(&self.v[y]).then(|| ProgramCounter::Skip).unwrap_or(ProgramCounter::Next)
    }

    fn op_6xkk(&mut self, x: usize, nn: u8) -> ProgramCounter {
        self.v[x] = nn;
        ProgramCounter::Next
    }

    fn op_7xkk(&mut self, x: usize, nn: u8) -> ProgramCounter {
        self.v[x] = self.v[x].wrapping_add(nn);
        ProgramCounter::Next
    }

    fn op_8xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] = self.v[y];
        ProgramCounter::Next
    }

    fn op_8xy1(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] |= self.v[y];
        ProgramCounter::Next
    }

    fn op_8xy2(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] &= self.v[y];
        ProgramCounter::Next
    }

    fn op_8xy3(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] ^= self.v[y];
        ProgramCounter::Next
    }

    fn op_8xy4(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = self.v[x].checked_add(self.v[y]).is_none().into();
        self.v[x] = self.v[x].wrapping_add(self.v[y]);
        ProgramCounter::Next
    }

    fn op_8xy5(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = self.v[x].checked_div(self.v[y]).is_some().into();
        self.v[x] = self.v[x].abs_diff(self.v[y]);
        ProgramCounter::Next
    }

    fn op_8xy6(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = self.v[x] & 0x1;
        self.v[x] = self.v[x].wrapping_shr(1);
        ProgramCounter::Next
    }

    fn op_8xy7(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = self.v[x] >> 7;
        self.v[x] = self.v[x].wrapping_shl(1);
        ProgramCounter::Next
    }

    fn op_9xy0(&self, x: usize, y: usize) -> ProgramCounter {
        self.v[x].eq(&self.v[y]).then(|| ProgramCounter::Next).unwrap_or(ProgramCounter::Skip)
    }

    fn op_annn(&mut self, nnn: u16) -> ProgramCounter {
        self.i = nnn;
        ProgramCounter::Next
    }

    fn op_bnnn(&self, nnn: u16) -> ProgramCounter {
        ProgramCounter::Jump(nnn)
    }

    fn op_cxkk(&mut self, x: usize, nn: u8) -> ProgramCounter {
        self.v[x] &= rand::random::<u8>();
        ProgramCounter::Next
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) -> ProgramCounter {
        ProgramCounter::Next
    }

    fn op_ex9e(&self, x: usize, keypad: &[u8; 17]) -> ProgramCounter { 
        keypad[self.v[x] as usize].eq(&0x1).then(|| ProgramCounter::Skip).unwrap_or(ProgramCounter::Next)
    }

    fn op_exa1(&self, x: usize, keypad: &[u8; 17]) -> ProgramCounter {
        keypad[self.v[x] as usize].eq(&0x1).then(|| ProgramCounter::Next).unwrap_or(ProgramCounter::Skip)
    }

    fn op_fx07(&self, x: usize) -> ProgramCounter {
        self.v[x] = self.delay_timer;
        ProgramCounter::Next
    }

    fn op_fx0a(&mut self, x: usize, keypad: &[u8; 17]) -> ProgramCounter {
        match (self.waiting_for_press, keypad.last()) {
            (true, Some(n)) => {
                self.v[x] = n.clone();
                self.waiting_for_press = false;
                ProgramCounter::Next 
            }
            (true, None) => {
                ProgramCounter::Wait
            }
            (false, _) => {
                self.waiting_for_press = true;
                ProgramCounter::Wait
            }
        }
    }

    fn op_fx15(&mut self, x: usize) -> ProgramCounter {
        self.delay_timer = self.v[x];
        ProgramCounter::Next
    }

    fn op_fx18(&mut self, x: usize) -> ProgramCounter {
        self.sound_timer = self.v[x];
        ProgramCounter::Next
    }

    fn op_fx1e(&mut self, x: usize) -> ProgramCounter {
        self.i += self.v[x] as u16; 
        ProgramCounter::Next
    }

    fn op_fx29(&mut self, x: usize) -> ProgramCounter {
        self.i = u16::from(self.v[x]) * 5;
        ProgramCounter::Next
    }

    fn op_fx33(&mut self, x: usize) -> ProgramCounter {
        let x = self.v[x];
        let i = self.i as usize;
        self.ram[i] = x / 100;
        self.ram[i + 1] = (x / 10) % 10;
        self.ram[i + 2] = (x % 100) % 10;
        ProgramCounter::Next
    }

    fn op_fx55(&mut self, x: usize) -> ProgramCounter {
        let i = self.i;
        for pos in 0..=x {
            self.ram[(i + pos as u16) as usize] = self.v[pos as usize];
        }
        ProgramCounter::Next
    }

    fn op_fx65(&mut self, x: usize) -> ProgramCounter {
        let i = self.i;
        for pos in 0..=x {
            self.v[pos as usize] = self.ram[(i + pos as u16) as usize]; 
        }
        ProgramCounter::Next
    }
}