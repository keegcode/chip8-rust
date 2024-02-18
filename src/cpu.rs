use std::io::Read;
use std::time::Duration;
use std::{fs, thread};

use super::fonts;

pub struct CPU {
    ram: Vec<u8>,
    pub vram: Vec<Vec<u8>>,
    pub vram_updated: bool,
    i: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pc: usize,
    v: Vec<u8>,
    stack: Vec<usize>,
    pub waiting_for_press: bool,
    window: (u8, u8),
}

pub enum ProgramCounter {
    Next,
    Call(u16),
    Jump(u16),
    Skip,
    Wait,
    Unknown,
}

impl CPU {
    pub fn init(rom: &str, (w, h): (u8, u8)) -> Result<CPU, String> {
        let mut cpu = CPU {
            ram: Vec::with_capacity(4096),
            vram: vec![vec![0; w as usize]; h as usize],
            vram_updated: false,
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            v: vec![0; 16],
            stack: Vec::new(),
            waiting_for_press: false,
            window: (w, h),
        };
        
        cpu.ram.append(&mut Vec::from(fonts::FONTS));
        cpu.ram.resize(cpu.pc, 0);

        fs::File::open(rom)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut cpu.ram)
            .map_err(|e| e.to_string())?;
        
        cpu.ram.resize(4096, 0);
        
        Ok(cpu)
    }

    pub fn tick(&mut self, keypad: &[u8; 17], key: Option<u8>) -> Result<&mut Self, String> {
        self.vram_updated = false; 
        
        let counter = self.execute_op(keypad, key);
        
        match counter {
            ProgramCounter::Next => self.increment_op(),
            ProgramCounter::Call(op) => self.call_op(op),
            ProgramCounter::Jump(op) => self.jump_to_op(op),
            ProgramCounter::Skip => self.skip_op(),
            ProgramCounter::Wait => (),
            _ => ()
        }

        Ok(self)
    }
    
    pub fn increment_op(&mut self) -> () {
        self.pc += 2;
    }

    pub fn jump_to_op(&mut self, op: u16) {
        self.increment_op();
        self.pc = op as usize;
    }
    
    pub fn skip_op(&mut self) {
        self.increment_op();
        self.increment_op();
    }

    pub fn call_op(&mut self, op: u16) {
        self.increment_op();
        self.stack.push(self.pc);
        self.pc = op as usize;
    }
    
    pub fn delay(&mut self) {
        while self.delay_timer > 0 {
            self.delay_timer -= 1;
            thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
        }
    }

    pub fn reset_sound_timer(&mut self) {
        self.sound_timer = 0;
    }

    pub fn execute_op(&mut self, keypad: &[u8; 17], key: Option<u8>) -> ProgramCounter {
            let (high, low)= (self.ram[self.pc], self.ram[self.pc + 1]); 
            
            //println!("high: {:#4x}, low: {:#4x}, pc: {}", high, low, self.pc);

            let op = high >> 4; 
            let x: usize = (high & 0x0F).into();
            let y: usize = (low >> 4).into();
            let n = low & 0x0F;
            let nn = low; 
            let nnn = u16::from(high & 0x0F) << 8 | u16::from(low);

            match (op, x, y, n) {
                (0x0, 0x0, 0xe, 0x0) => self.op_cls(),
                (0x0, 0x0, 0xe, 0xe) => self.op_ret(),
                (0x1, _, _, _) => self.op_jp_addr(nnn),
                (0x2, _, _, _) => self.op_call_addr(nnn),
                (0x3, _, _, _) => self.op_3xkk(x, nn),
                (0x4, _, _, _) => self.op_4xkk(x, nn),
                (0x5, _, _, 0x0) => self.op_5xy0(x, y),
                (0x6, _, _, _) => self.op_6xkk(x, nn),
                (0x7, _, _, _) => self.op_7xkk(x, nn),
                (0x8, _, _, 0x0) => self.op_8xy0(x, y),
                (0x8, _, _, 0x1) => self.op_8xy1(x, y),
                (0x8, _, _, 0x2) => self.op_8xy2(x, y),
                (0x8, _, _, 0x3) => self.op_8xy3(x, y),
                (0x8, _, _, 0x4) => self.op_8xy4(x, y),
                (0x8, _, _, 0x5) => self.op_8xy5(x, y),
                (0x8, _, _, 0x6) => self.op_8xy6(x, y),
                (0x8, _, _, 0x7) => self.op_8xy7(x, y),
                (0x8, _, _, 0xe) => self.op_8xye(x, y),
                (0x9, _, _, 0) => self.op_9xy0(x, y),
                (0xa, _, _, _) => self.op_annn(nnn),
                (0xb, _, _, _) => self.op_bnnn(nnn),
                (0xc, _, _, _) => self.op_cxkk(x, nn),
                (0xd, _, _, _) => self.op_dxyn(x, y, n),
                (0xe, _, 0x9, 0xe) => self.op_ex9e(x, keypad),
                (0xe, _, 0xa, 0x1) => self.op_exa1(x, keypad),
                (0xf, _, 0x0, 0x7) => self.op_fx07(x),
                (0xf, _, 0x0, 0xa) => self.op_fx0a(x, key),
                (0xf, _, 0x1, 0x5) => self.op_fx15(x),
                (0xf, _, 0x1, 0x8) => self.op_fx18(x),
                (0xf, _, 0x1, 0xe) => self.op_fx1e(x),
                (0xf, _, 0x2, 0x9) => self.op_fx29(x),
                (0xf, _, 0x3, 0x3) => self.op_fx33(x),
                (0xf, _, 0x5, 0x5) => self.op_fx55(x),
                (0xf, _, 0x6, 0x5) => self.op_fx65(x),
                _ => ProgramCounter::Unknown,
            }
    }
    
    fn op_cls(&mut self) -> ProgramCounter {
        self.vram.iter_mut().for_each(|v| v.fill(0));
        self.vram_updated = true;
        ProgramCounter::Next
    }
    
    fn op_ret(&mut self) -> ProgramCounter {
        ProgramCounter::Jump(self.stack.pop().unwrap() as u16)
    }
    
    fn op_call_addr(&mut self, nnn: u16) -> ProgramCounter {
        ProgramCounter::Call(nnn)
    }
    
    fn op_jp_addr(&self, nnn: u16) -> ProgramCounter {
        ProgramCounter::Jump(nnn)
    }
    
    fn op_3xkk(&self, x: usize, nn: u8) -> ProgramCounter {
        if self.v[x] == nn {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    fn op_4xkk(&self, x: usize, nn: u8) -> ProgramCounter {
        if self.v[x] != nn {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    fn op_5xy0(&self, x: usize, y: usize) -> ProgramCounter {
        if self.v[x] == self.v[y] {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
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
        self.v[0xF] = 0;
        ProgramCounter::Next
    }

    fn op_8xy2(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] &= self.v[y];
        self.v[0xF] = 0;
        ProgramCounter::Next
    }

    fn op_8xy3(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] ^= self.v[y];
        self.v[0xF] = 0;
        ProgramCounter::Next
    }

    fn op_8xy4(&mut self, x: usize, y: usize) -> ProgramCounter {
        let (sum, overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = sum;
        self.v[0xF] = overflow.into();
        ProgramCounter::Next
    }

    fn op_8xy5(&mut self, x: usize, y: usize) -> ProgramCounter {
        let (sub, overflow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = sub;
        self.v[0xF] = (!overflow).into();
        ProgramCounter::Next
    }

    fn op_8xy6(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = self.v[x] & 1;
        self.v[x] >>= 1;
        ProgramCounter::Next
    }

    fn op_8xy7(&mut self, x: usize, y: usize) -> ProgramCounter {
        let (sub, overflow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = sub;
        self.v[0xF] = (!overflow).into();
        ProgramCounter::Next
    }
    
    fn op_8xye(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = (u8::from(self.v[x]) >> 7) & 1;
        self.v[x] <<= 1;
        ProgramCounter::Next
    }

    fn op_9xy0(&self, x: usize, y: usize) -> ProgramCounter {
        if self.v[x] != self.v[y] {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    fn op_annn(&mut self, nnn: u16) -> ProgramCounter {
        self.i = nnn;
        ProgramCounter::Next
    }

    fn op_bnnn(&self, nnn: u16) -> ProgramCounter {
        ProgramCounter::Jump(nnn + self.v[0] as u16)
    }

    fn op_cxkk(&mut self, x: usize, nn: u8) -> ProgramCounter {
        self.v[x] = rand::random::<u8>() & nn;
        ProgramCounter::Next
    }

    fn op_dxyn(&mut self, vx: usize, vy: usize, n: u8) -> ProgramCounter {
        let (w, h) = self.window;
        self.v[0xF] = 0;
        
        for i in 0..n {
            let y = (self.v[vy] + i) % h;
            let sprite = self.ram[(self.i + i as u16) as usize];
            for j in 0..8 {
                let x = (self.v[vx] + j) % w;
                let bit = (sprite >> (7 - j)) & 1;
                self.v[0xF] = bit & self.vram[y as usize][x as usize];
                self.vram[y as usize][x as usize] ^= bit; 
            }
        }

        self.vram_updated = true;
        ProgramCounter::Next
    }

    fn op_ex9e(&self, x: usize, keypad: &[u8; 17]) -> ProgramCounter { 
        if keypad[self.v[x] as usize] == 1 {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    fn op_exa1(&self, x: usize, keypad: &[u8; 17]) -> ProgramCounter {
        if keypad[self.v[x] as usize] == 1 {
            ProgramCounter::Next
        } else {
            ProgramCounter::Skip
        }
    }

    fn op_fx07(&mut self, x: usize) -> ProgramCounter {
        self.v[x] = self.delay_timer;
        ProgramCounter::Next
    }

    fn op_fx0a(&mut self, x: usize, key: Option<u8>) -> ProgramCounter {
        match (self.waiting_for_press, key) {
            (true, Some(n)) => {
                self.v[x] = n;
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