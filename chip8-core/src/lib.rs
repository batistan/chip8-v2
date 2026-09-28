use crate::consts::{FONT_OFFSET, PC_START, SCREEN_HEIGHT, SCREEN_WIDTH};

pub trait RandomSource {
    fn next_u8(&mut self) -> u8;
}

#[derive(Debug)]
pub struct TickOutput {
    pub screen_updated: bool,
    pub sound_active: bool,
}

#[derive(Debug)]
pub enum EmulatorError {
    UnknownOpcode(u16),
    StackOverflow,
    StackUnderflow,
    MemoryOutOfBounds { address: u16 },
    InvalidRegister { register: u8 },
    InvalidKey { key: u8 },
}

impl std::fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmulatorError::UnknownOpcode(inst) => write!(f, "Unknown opcode: {:04X}", inst),
            EmulatorError::StackOverflow => write!(f, "Stack overflow"),
            EmulatorError::StackUnderflow => write!(f, "Stack underflow"),
            EmulatorError::MemoryOutOfBounds { address } => {
                write!(f, "Memory out of bounds: {:04X}", address)
            }
            EmulatorError::InvalidRegister { register } => {
                write!(f, "Invalid register: {}", register)
            }
            EmulatorError::InvalidKey { key } => write!(f, "Invalid key: {}", key),
        }
    }
}

impl std::error::Error for EmulatorError {}

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
    sp: u8,  // stack pointer
    delay_timer: u8,
    sound_timer: u8,
    timer_accum: f64,
    cycle_accum: f64, // fractional cycles carried between ticks

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
            screen: [0u8; SCREEN_WIDTH * SCREEN_HEIGHT],
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
            cycle_accum: 0.0,
            cpu_hz: 500.0,
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), EmulatorError> {
        if data.len() > self.mem.len() - PC_START {
            return Err(EmulatorError::MemoryOutOfBounds {
                address: (PC_START + data.len()) as u16,
            });
        }

        self.reset();

        self.mem[PC_START..PC_START + data.len()].copy_from_slice(data);

        Ok(())
    }

    pub fn tick(&mut self, delta_ms: f64) -> Result<TickOutput, EmulatorError> {
        // number of cycles is number seconds * cpu clock speed (cycles / second);
        // the fractional remainder carries over so small deltas still add up
        self.cycle_accum += (delta_ms / 1000.0) * self.cpu_hz;
        let num_cycles = self.cycle_accum as u32;
        self.cycle_accum -= num_cycles as f64;

        let mut should_update_screen = false;

        for _ in 0..num_cycles {
            should_update_screen |= self.cycle()?;
        }

        self.update_timers(delta_ms);

        Ok(TickOutput {
            screen_updated: should_update_screen,
            sound_active: self.sound_timer > 0,
        })
    }

    /// Runs exactly one CPU cycle, advancing the timers by one cycle's worth
    /// of time so they stay in step with the CPU.
    pub fn step(&mut self) -> Result<TickOutput, EmulatorError> {
        let screen_updated = self.cycle()?;
        self.update_timers(1000.0 / self.cpu_hz);

        Ok(TickOutput {
            screen_updated,
            sound_active: self.sound_timer > 0,
        })
    }

    // returns whether the screen changed
    fn cycle(&mut self) -> Result<bool, EmulatorError> {
        if self.waiting_input {
            self.check_keys();
            return Ok(false);
        }

        self.draw = false;
        self.execute(self.next_instruction()?)?;
        self.pc += 2;

        Ok(self.draw)
    }

    fn next_instruction(&self) -> Result<u16, EmulatorError> {
        if self.pc + 2 > self.mem.len() as u16 {
            return Err(EmulatorError::MemoryOutOfBounds { address: self.pc });
        }

        // instructions are 2 bytes long, so we read 2 bytes at a time
        let inst_msb = self.mem[self.pc as usize] as u16;
        let inst_lsb = self.mem[(self.pc + 1) as usize] as u16;

        Ok((inst_msb << 8) | inst_lsb)
    }

    fn check_keys(&mut self) {
        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.reg[self.waiting_reg] = i as u8;
                self.waiting_input = false;
                return;
            }
        }
    }

    // timers decrement at 60 Hz regardless of CPU speed
    fn update_timers(&mut self, delta_ms: f64) {
        self.timer_accum += delta_ms / 1000.0;
        while self.timer_accum >= 1.0 / 60.0 {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            };
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            };

            self.timer_accum -= 1.0 / 60.0;
        }
    }

    pub fn key_down(&mut self, key: u8) {
        if key > 0xF {
            return;
        }

        self.keys[key as usize] = true;
    }

    pub fn key_up(&mut self, key: u8) {
        if key > 0xF {
            return;
        }

        self.keys[key as usize] = false;
    }

    pub fn screen(&self) -> &[u8] {
        // 64 * 32 = 2048 bytes
        &self.screen
    }

    pub fn screen_width(&self) -> usize {
        SCREEN_WIDTH
    }
    pub fn screen_height(&self) -> usize {
        SCREEN_HEIGHT
    }

    pub fn registers(&self) -> &[u8] {
        &self.reg
    }

    pub fn memory(&self) -> &[u8] {
        &self.mem
    }

    pub fn stack(&self) -> &[u16] {
        &self.stack
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn index(&self) -> u16 {
        self.index
    }

    pub fn sp(&self) -> u8 {
        self.sp
    }

    pub fn delay_timer(&self) -> u8 {
        self.delay_timer
    }

    pub fn sound_timer(&self) -> u8 {
        self.sound_timer
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
        self.cycle_accum = 0.0;
    }
}

mod consts;
mod cpu;
mod font;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::PC_START;

    struct FixedRng(u8);

    impl RandomSource for FixedRng {
        fn next_u8(&mut self) -> u8 {
            self.0
        }
    }

    fn emu() -> Emulator {
        Emulator::new(Box::new(FixedRng(0)))
    }

    // --- load_rom ---

    #[test]
    fn test_load_rom() {
        let mut emu = emu();
        let rom = [0x60, 0x42, 0x61, 0x10]; // two instructions
        emu.load_rom(&rom).unwrap();

        assert_eq!(emu.mem[PC_START], 0x60);
        assert_eq!(emu.mem[PC_START + 1], 0x42);
        assert_eq!(emu.mem[PC_START + 2], 0x61);
        assert_eq!(emu.mem[PC_START + 3], 0x10);
    }

    #[test]
    fn test_load_rom_resets_state() {
        let mut emu = emu();
        emu.reg[0] = 0xFF;
        emu.sp = 5;
        emu.delay_timer = 10;

        emu.load_rom(&[0x00, 0xE0]).unwrap();

        assert_eq!(emu.reg[0], 0);
        assert_eq!(emu.sp, 0);
        assert_eq!(emu.delay_timer, 0);
        assert_eq!(emu.pc, PC_START as u16);
    }

    #[test]
    fn test_load_rom_too_large() {
        let mut emu = emu();
        let rom = vec![0u8; 4096 - PC_START + 1]; // one byte too many
        assert!(matches!(
            emu.load_rom(&rom),
            Err(EmulatorError::MemoryOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_load_rom_max_size() {
        let mut emu = emu();
        let rom = vec![0u8; 4096 - PC_START]; // exactly fills available memory
        emu.load_rom(&rom).unwrap();
    }

    // --- tick ---

    #[test]
    fn test_tick_executes_instruction() {
        let mut emu = emu();
        // load a ROM: V0 = 0x42 (0x6042)
        emu.load_rom(&[0x60, 0x42]).unwrap();

        // 2ms at 500 Hz = 1 cycle
        emu.tick(2.0).unwrap();

        assert_eq!(emu.reg[0], 0x42);
        assert_eq!(emu.pc, PC_START as u16 + 2);
    }

    #[test]
    fn test_tick_accumulates_fractional_cycles() {
        let mut emu = emu();
        // V0 += 1, three times
        emu.load_rom(&[0x70, 0x01, 0x70, 0x01, 0x70, 0x01]).unwrap();

        // 1ms at 500 Hz = half a cycle; two of them make one full cycle
        emu.tick(1.0).unwrap();
        assert_eq!(emu.pc, PC_START as u16);
        emu.tick(1.0).unwrap();
        assert_eq!(emu.pc, PC_START as u16 + 2);

        // 3ms = 1.5 cycles → one now, the remaining half carries over
        emu.tick(3.0).unwrap();
        assert_eq!(emu.pc, PC_START as u16 + 4);
        emu.tick(1.0).unwrap();
        assert_eq!(emu.pc, PC_START as u16 + 6);
        assert_eq!(emu.reg[0], 3);
    }

    #[test]
    fn test_reset_clears_cycle_accum() {
        let mut emu = emu();
        emu.load_rom(&[0x70, 0x01]).unwrap();
        emu.tick(1.0).unwrap();

        emu.reset();
        emu.tick(1.0).unwrap();
        assert_eq!(emu.pc, PC_START as u16);
    }

    #[test]
    fn test_step_executes_one_instruction() {
        let mut emu = emu();
        emu.load_rom(&[0x60, 0x42, 0x61, 0x10]).unwrap();

        emu.step().unwrap();
        assert_eq!(emu.reg[0], 0x42);
        assert_eq!(emu.reg[1], 0x00);
        assert_eq!(emu.pc, PC_START as u16 + 2);

        emu.step().unwrap();
        assert_eq!(emu.reg[1], 0x10);
        assert_eq!(emu.pc, PC_START as u16 + 4);
    }

    #[test]
    fn test_step_reports_screen_updated() {
        let mut emu = emu();
        // CLS, then V0 = 0
        emu.load_rom(&[0x00, 0xE0, 0x60, 0x00]).unwrap();

        assert!(emu.step().unwrap().screen_updated);
        assert!(!emu.step().unwrap().screen_updated);
    }

    #[test]
    fn test_step_advances_timers_per_cycle() {
        let mut emu = emu();
        // JP 0x200: loop forever
        emu.load_rom(&[0x12, 0x00]).unwrap();
        emu.delay_timer = 5;

        // at 500 Hz a timer tick (1/60 s) spans 8.33 cycles
        for _ in 0..8 {
            emu.step().unwrap();
        }
        assert_eq!(emu.delay_timer, 5);
        emu.step().unwrap();
        assert_eq!(emu.delay_timer, 4);
    }

    #[test]
    fn test_step_waiting_for_input() {
        let mut emu = emu();
        emu.load_rom(&[0x60, 0x42]).unwrap();
        emu.waiting_input = true;
        emu.waiting_reg = 3;

        emu.step().unwrap();
        assert_eq!(emu.pc, PC_START as u16);

        emu.key_down(0x7);
        emu.step().unwrap();
        assert_eq!(emu.reg[3], 0x7);
        assert!(!emu.waiting_input);
    }

    #[test]
    fn test_tick_screen_updated() {
        let mut emu = emu();
        // load a clear screen instruction (00E0)
        emu.load_rom(&[0x00, 0xE0]).unwrap();
        let output = emu.tick(2.0).unwrap();
        assert!(output.screen_updated);
    }

    #[test]
    fn test_tick_sound_active() {
        let mut emu = emu();
        // load NOP-like instruction that won't error (V0 = 0)
        emu.load_rom(&[0x60, 0x00]).unwrap();
        emu.sound_timer = 5;

        let output = emu.tick(2.0).unwrap();
        assert!(output.sound_active);
    }

    #[test]
    fn test_tick_waiting_for_input() {
        let mut emu = emu();
        emu.load_rom(&[0x60, 0x42]).unwrap();
        emu.waiting_input = true;
        emu.waiting_reg = 3;

        // no keys pressed, tick should not execute instructions
        emu.tick(2.0).unwrap();
        assert_eq!(emu.reg[0], 0x00); // instruction was NOT executed
        assert_eq!(emu.pc, PC_START as u16); // PC didn't advance
    }

    // --- update_timers ---

    #[test]
    fn test_timers_decrement_at_60hz() {
        let mut emu = emu();
        emu.delay_timer = 2;
        emu.sound_timer = 2;

        // 1/60th of a second in ms ≈ 16.667ms → one decrement
        emu.update_timers(16.667);
        assert_eq!(emu.delay_timer, 1);
        assert_eq!(emu.sound_timer, 1);
    }

    #[test]
    fn test_timers_stop_at_zero() {
        let mut emu = emu();
        emu.delay_timer = 0;
        emu.sound_timer = 0;

        emu.update_timers(16.667);
        assert_eq!(emu.delay_timer, 0);
        assert_eq!(emu.sound_timer, 0);
    }

    #[test]
    fn test_timers_multiple_decrements() {
        let mut emu = emu();
        emu.delay_timer = 10;

        // 5/60ths of a second → 5 decrements
        emu.update_timers(5.0 * 1000.0 / 60.0);
        assert_eq!(emu.delay_timer, 5);
    }

    // --- check_keys ---

    #[test]
    fn test_check_keys_stores_first_pressed() {
        let mut emu = emu();
        emu.waiting_input = true;
        emu.waiting_reg = 5;
        emu.keys[3] = true;
        emu.keys[7] = true;

        emu.check_keys();

        assert_eq!(emu.reg[5], 3); // first pressed key (lowest index)
        assert!(!emu.waiting_input);
    }

    #[test]
    fn test_check_keys_no_key_pressed() {
        let mut emu = emu();
        emu.waiting_input = true;
        emu.waiting_reg = 5;

        emu.check_keys();

        assert!(emu.waiting_input); // still waiting
    }

    // --- key_down / key_up ---

    #[test]
    fn test_key_down_and_up() {
        let mut emu = emu();
        emu.key_down(0x5);
        assert!(emu.keys[0x5]);

        emu.key_up(0x5);
        assert!(!emu.keys[0x5]);
    }

    #[test]
    fn test_key_down_ignores_invalid() {
        let mut emu = emu();
        emu.key_down(0x10); // > 0xF, should be ignored
        assert_eq!(emu.keys, [false; 16]);
    }

    #[test]
    fn test_key_up_ignores_invalid() {
        let mut emu = emu();
        emu.keys[0] = true;
        emu.key_up(0x10); // > 0xF, should be ignored
        assert!(emu.keys[0]); // unchanged
    }

    // --- reset ---

    #[test]
    fn test_reset() {
        let mut emu = emu();
        emu.reg[0] = 0xFF;
        emu.sp = 5;
        emu.index = 0x300;
        emu.delay_timer = 10;
        emu.sound_timer = 10;
        emu.keys[5] = true;
        emu.screen[100] = 1;
        emu.pc = 0x400;

        emu.reset();

        assert_eq!(emu.reg, [0; 16]);
        assert_eq!(emu.sp, 0);
        assert_eq!(emu.index, 0);
        assert_eq!(emu.delay_timer, 0);
        assert_eq!(emu.sound_timer, 0);
        assert_eq!(emu.keys, [false; 16]);
        assert_eq!(emu.screen, [0; SCREEN_WIDTH * SCREEN_HEIGHT]);
        assert_eq!(emu.pc, PC_START as u16);
    }

    // --- screen ---

    #[test]
    fn test_screen_returns_buffer() {
        let mut emu = emu();
        emu.screen[0] = 1;
        emu.screen[100] = 1;

        let screen = emu.screen();
        assert_eq!(screen.len(), SCREEN_WIDTH * SCREEN_HEIGHT);
        assert_eq!(screen[0], 1);
        assert_eq!(screen[100], 1);
    }

    // --- internals accessors ---

    #[test]
    fn test_registers_accessor() {
        let mut emu = emu();
        emu.reg[0] = 0xAA;
        emu.reg[0xF] = 0xBB;

        let regs = emu.registers();
        assert_eq!(regs.len(), 16);
        assert_eq!(regs[0], 0xAA);
        assert_eq!(regs[0xF], 0xBB);
    }

    #[test]
    fn test_memory_accessor() {
        let mut emu = emu();
        emu.load_rom(&[0x60, 0x42]).unwrap();

        let mem = emu.memory();
        assert_eq!(mem.len(), 4096);
        assert_eq!(mem[PC_START], 0x60);
        assert_eq!(mem[PC_START + 1], 0x42);
    }

    #[test]
    fn test_stack_accessor() {
        let mut emu = emu();
        emu.stack[0] = 0x300;
        emu.stack[1] = 0x400;

        let stack = emu.stack();
        assert_eq!(stack.len(), 16);
        assert_eq!(stack[0], 0x300);
        assert_eq!(stack[1], 0x400);
    }

    #[test]
    fn test_scalar_accessors() {
        let mut emu = emu();
        emu.pc = 0x250;
        emu.index = 0x300;
        emu.sp = 3;
        emu.delay_timer = 30;
        emu.sound_timer = 20;

        assert_eq!(emu.pc(), 0x250);
        assert_eq!(emu.index(), 0x300);
        assert_eq!(emu.sp(), 3);
        assert_eq!(emu.delay_timer(), 30);
        assert_eq!(emu.sound_timer(), 20);
    }
}
