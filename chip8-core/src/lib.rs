use crate::consts::{FONT_OFFSET, PC_START, SCREEN_HEIGHT, SCREEN_WIDTH};

pub trait RandomSource {
    fn next_u8(&mut self) -> u8;
}

pub struct TickOutput {
    pub screen_updated: bool,
    pub sound_active: bool,
}

pub enum EmulatorError {
    UnknownOpcode(u16),
    StackOverflow,
    StackUnderflow,
    MemoryOutOfBounds { address: u16 },
    InvalidRegister { register: u8 },
    InvalidKey { key: u8 },
}

pub struct Emulator {
    rng: Box<dyn RandomSource>,
    mem: [u8; 4096],
    stack: [u16; 16],
    screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    draw: bool, // whether to update the screen this tick

    keys: [bool; 16], // 0-F keys, true if pressed, false otherwise
    waiting_input: bool,
    waiting_reg: usize, // which register to store key into if waiting for input

    reg: [u8; 16], // V0-VF general purpose registers
    index: u16,
    pc: u16, // program counter
    sp: u8, // stack pointer
    delay_timer: u8,
    sound_timer: u8,
    timer_accum: f64,

    cpu_hz: f64, // CPU clock speed in Hz, default 500
}

impl Emulator {
    pub fn new(rng: Box<dyn RandomSource>) -> Self {
        let mut mem = [0u8; 4096];

        mem[FONT_OFFSET..FONT_OFFSET + font::FONT.len()].copy_from_slice(&font::FONT);

        Emulator {
            rng,
            mem,
            stack: [0u16; 16],
            screen: [0u8; 64 * 32],
            draw: false,
            keys: [false; 16],
            waiting_input: false,
            waiting_reg: 0,
            reg: [0u8; 16],
            index: 0,
            pc: PC_START as u16,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            timer_accum: 0.0,
            cpu_hz: 500.0,
        }
    }
    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), EmulatorError> {
        if data.len() > self.mem.len() - PC_START {
            return Err(EmulatorError::MemoryOutOfBounds { address: (PC_START + data.len()) as u16 });
        }

        self.reset();

        self.mem[PC_START..PC_START + data.len()].copy_from_slice(data);

        Ok(())
    }

    pub fn tick(&mut self, delta_ms: f64) -> Result<TickOutput, EmulatorError> {
        todo!()
    }

    pub fn key_down(&mut self, key: u8) {
        if key > 0xF { return; }

        self.keys[key as usize] = true;
    }

    pub fn key_up(&mut self, key: u8) {
        if key > 0xF { return; }

        self.keys[key as usize] = false;
    }

    pub fn screen(&self) -> &[u8] { // 64 * 32 = 2048 bytes
        &self.screen
    }

    pub fn reset(&mut self) {
        self.mem[FONT_OFFSET..FONT_OFFSET + font::FONT.len()].copy_from_slice(&font::FONT);
        self.pc = PC_START as u16;

        self.keys = [false; 16];
        self.waiting_input = false;
        self.waiting_reg = 0;
        self.reg = [0u8; 16];
        self.stack = [0u16; 16];
        self.sp = 0;
        self.index = 0;
        self.screen = [0u8; 64 * 32];
        self.draw = false;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.timer_accum = 0.0;
    }
}

mod font;
mod consts;
mod cpu;
