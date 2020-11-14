use crate::util::*;
use crate::util::Wrapable;
use rand::prelude::*;
use std::fmt;

type Sprite = [u8; 5];
type Pixel = bool;
type OverrwritesDisplay = bool;
type Vx = usize;
type Key = u8;
pub type Result<T> = std::result::Result<T, Chip8Error>;


#[derive(Clone, Debug )]
pub enum Chip8Error {
    UnimplementedInstruction(String),
    UnknownInstruction(String),
    BoundsError(String),
}


static CHARS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20,
    0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10,
    0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
    0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 
    0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80
];

#[derive(Copy, Debug, Clone)]
enum Chip8State {
    Running,
    WaitingForKey(Vx),
}

#[derive(Clone)]
pub struct Chip8 {
    pub display: Display,
    memory: [u8; 0xFFF],
    stack: [u16; 0x10],
    v: [u8; 0x10],
    dt: u8,
    st: u8,
    sp: u8,
    pc: u16,
    i: u16,
    pub key: Option<Key>,
    state: Chip8State,
    pub show: bool,
    incr: bool,
}

impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("\nChip8")
         .field("v", &self.v)
         .field("\nstack", &self.stack)
         .field("\npc", &self.pc)
         .field("key", &self.key)
         .field("sp", &self.sp)
         .field("dt", &self.dt)
         .field("st", &self.st)
         .field("i", &self.i)
         .finish()
    }
}

impl Chip8 {
    pub fn new() -> Self {
        let display = Display::new();
        let mut memory = [0; 0xFFF];
        for i in 0..80 {
            memory[i] = CHARS[i];
        }
        Chip8 {
            display,
            memory,
            stack: [0; 0x10],
            v: [0; 0x10], // registers
            dt: 0, // delay timer
            st: 0, // sound timer
            sp: 0, // stack pointer 
            pc: 0x200, // program counter 
            i: 0,
            key: None,
            state: Chip8State::Running,
            show: false, // update display to screen
            incr: true, // increment the PC after instruction
        }
    }

    pub fn load(&mut self, game: Vec<u8>) -> Result<()> {
        for i in 0..game.len() {
            self.memory[i + 0x200 as usize] = game[i];
        }
        Ok(())
    }

    fn pc_incr(&mut self) {
        self.pc = self.pc.wrap_add(2);
    }

    fn set_st(&mut self, x: u8) -> Result<()> {
        Ok(self.st = x)
    }

    fn set_i(&mut self, x: u16) -> Result<()> {
        Ok(self.i = x)
    }

    fn set_dt(&mut self, x: u8) -> Result<()> {
        Ok(self.dt = x)
    }

    fn set_memory_location(&mut self, location: usize, x: u8) -> Result<()> {
        if location > 0xFFF {
            Err(Chip8Error::BoundsError(format!("Index OOB: v{}", x)))
        } else { 
            Ok(self.memory[location] = x)
        }
    }

    fn set_vx(&mut self, vx: usize, y: u8) -> Result<()> {
        if vx > 0x10 {
            Err(Chip8Error::BoundsError(format!("Index OOB: v{}", vx)))
        } else { 
            Ok(self.v[vx] = y)
        }
    }

    fn timers_decr(&mut self) {
        if self.dt > 0 { self.dt -= 1; }
        if self.st > 0 { self.st -= 1; }
    }

    pub fn run_once(&mut self) -> Result<()> {
        self.show = false;
        match self.state {
           Chip8State::Running => {
               self.exec_instruction()?;
               self.timers_decr();
               if self.incr {
                   self.pc_incr();
               }
           },
           Chip8State::WaitingForKey(v) => self.wait_for_key(v)?,
        };
        Ok(())
    }

    pub fn wait_for_key(&mut self, v: usize) -> Result<()> {
        if let Some(k) = self.key {
            self.set_vx(v, k)?;
            self.state = Chip8State::Running;
        }
        Ok(())
    }

    fn exec_instruction(&mut self) -> Result<()> {
        self.incr = true;
        let instruction = 
            ((self.memory[self.pc as usize] as u16) << 8) + (self.memory[(self.pc + 1) as usize] as u16);

        // reference: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#3.0
        let opcode = instruction >> 12;
        let addr = instruction & 0xFFF;
        let n = (instruction & 0xF) as u8;
        let x = ((instruction & 0xF00) >> 8) as usize;
        let y = ((instruction & 0xF0) >> 4) as usize;
        let kk = (instruction & 0xFF) as u8;

        let mut rng = thread_rng();

        let vx = self.v[x];
        let vy = self.v[y];
        return match opcode {
            // misc
            0x0 => match instruction {
                // clear display 
                0x00E0 => self.display.clear(),

                // return from subroutine
                0x00EE => {
                    self.sp = self.sp.wrap_sub(1);
                    self.pc = self.stack[self.sp as usize];
                    Ok(())
                }
                _ => Err(Chip8Error::UnknownInstruction(format!("{:x}", instruction)))
            },

            // jump
            0x1 => {
                self.incr = false;
                Ok(self.pc = addr)
            },

            // call subroutine
            0x2 => { 
                self.incr = false;
                self.stack[self.sp as usize] = self.pc;
                self.sp = self.sp.wrap_add(1);
                self.pc = addr;
                Ok(())
            },

            // Skip next instruction if Vx = kk.
            0x3 => {
                if vx == kk {
                    self.pc_incr();
                }
                Ok(())
            },

            // Skip next instruction if Vx != kk.
            0x4 => {
                if vx != kk {
                    self.pc_incr();
                }
                Ok(())
            },

            // Skip next instruction if Vx = Vy.
            0x5 => {
                if vx == vy {
                    self.pc_incr();
                }
                Ok(())
            },

            // The interpreter puts the value kk into register Vx.
            0x6 => self.set_vx(x, kk),

            // Adds the value kk to the value of register Vx, then stores the result in Vx. 
            0x7 => self.set_vx(x, vx.wrap_add(kk)),

            0x8 => match n {
                // Set/LD
                0x0 => self.set_vx(x, vy),

                // or
                0x1 => self.set_vx(x, vx | vy),

                // and
                0x2 => self.set_vx(x, vx & vy),

                // xor
                0x3 => self.set_vx(x, vx ^ vy),
                
                // add
                //  The values of Vx and Vy are added together. If the result is greater than 8 
                //  bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of 
                //  the result are kept, and stored in Vx.
                0x4 => {
                    self.set_vx(x, vx.wrap_add(vy))?;
                    self.set_vx(0xf, (vx as usize + vy as usize > 255) as u8)?;
                    Ok(())
                },

                // sub
                0x5 => {
                    self.set_vx(x, vx.wrap_sub(vy))?;
                    if vx > vy {
                        self.set_vx(0xF, 1)
                    } else {
                        self.set_vx(0xF, 0)
                    }
                }

                // shr
                0x6 => {
                    self.set_vx(0xF, get_bit_at(vx, 0))?;
                    self.set_vx(x, vx >> 1)
                }

                // subn
                0x7 => {
                    self.set_vx(x, vy - vx)?;
                    if vy > vx {
                        self.set_vx(0xF, 1)
                    } else {
                        self.set_vx(0xF, 0)
                    }
                }

                // shl
                0xE => {
                    self.set_vx(0xF, get_bit_at(vx, 7))?;
                    self.set_vx(x, vx << 1)
                }

                _ => Err(Chip8Error::UnknownInstruction(format!("{:x}", instruction)))
            },

            // Skip next instruction if Vx != Vy.
            0x9 => {
                if vx != vy {
                    self.pc_incr();
                }
                Ok(())
            },

            // LD I, addr
            0xA => {
                Ok(self.i = addr)
            },

            //  Jump to location nnn + V0.
            0xB => {
                self.incr = false;
                Ok(self.pc = addr.wrap_add(self.v[0] as u16))
            }

            // random byte AND-ed with kk
            0xC => self.set_vx(x, rng.gen::<u8>() & kk),

            // display
            0xD => {
                for i in 0..n {
                    let byte = self.memory[(self.i as usize) + (i as usize)];
                    if self.display.draw_byte(vx as usize, vy as usize + i as usize, byte) {
                        self.set_vx(0xF, 1)?;
                    } else {
                        self.set_vx(0xF, 0)?;
                    }
                }
                self.show = true;
                Ok(())
            },


            0xE => match kk {
                0x9E => {
                    if self.key == Some(vx) {
                        self.pc_incr();
                    }
                    Ok(())
                }

                0xA1 => {
                    if self.key != Some(vx) {
                        self.pc_incr();
                    }
                    Ok(())
                }
                _ => Err(Chip8Error::UnknownInstruction(format!("{:x}", instruction))),
            },
            
            0xF => match kk {
                0x07 => self.set_vx(x, self.dt),
                0x0A => {
                    self.state = Chip8State::WaitingForKey(x as usize);
                    Ok(())
                },
                0x15 => self.set_dt(vx),
                0x18 => self.set_st(vx),
                0x1E => self.set_i(self.i + vx as u16),
                0x29 => self.set_i(5 * vx as u16),
                0x33 => {
                    let ones = vx % 10;
                    let tens = (vx / 10) % 10;
                    let hundreds = (vx / 100);
                    self.set_memory_location(self.i as usize, hundreds)?;
                    self.set_memory_location(1+self.i as usize, tens)?;
                    self.set_memory_location(2+self.i as usize, ones)?;
                    Ok(())
                },

                0x55 => {
                    for r in 0..=x {
                        self.set_memory_location((self.i as usize) + r, self.v[r])?;
                    }
                    Ok(())
                },

                0x65 => {
                    for r in 0..=x {
                        self.set_vx(r, self.memory[(self.i as usize) + r])?;
                    }
                    Ok(())
                },

                _ => Err(Chip8Error::UnknownInstruction(format!("{:x}", instruction))),
            },

            _ => Err(Chip8Error::UnknownInstruction(format!("{:x}", instruction)))
        }
    } 
}


#[derive (Clone, Debug)]
pub struct Display {
    screen: [[Pixel; 64]; 32]
}

impl Display {
    pub fn new() -> Self {
        return Display{ 
            screen: [[false; 64]; 32] 
        }
    }

    pub fn draw_byte(&mut self, x: usize, y: usize, byte: u8) -> OverrwritesDisplay {
        let mut overwrite = false;
        for i in 0..8 {
            let pixel: Pixel = get_bit_at(byte, 7 - i) != 0;
            overwrite |= self.set_pixel(x + i as usize, y, pixel) 
        }
        overwrite
    }

    fn set_pixel(&mut self, x: usize, y: usize, p: Pixel) -> OverrwritesDisplay {
        let x = x % 64;
        let y = y % 32;
        let overwrite = p & self.screen[y][x];
        self.screen[y][x] = self.screen[y][x] ^ p;
        overwrite
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        self.screen[y][x]
    }
    
    pub fn clear(&mut self) -> Result<()> {
        Ok(self.screen = [[false; 64]; 32])
    }

    pub fn debug_display(&self) {
        for y in 0..32 {
            for x in 0..64 {
                if self.get_pixel(x, y) {
                    print!("#");
                } else {
                    print!(".");
                }
            }
            println!();
        }
    }
}
