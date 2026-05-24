use crate::consts::{ADDR_MASK, FONT_OFFSET, IMM_MASK, OPCODE_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::{Emulator, EmulatorError};

impl Emulator {
    pub(crate) fn execute(&mut self, inst: u16) -> Result<(), EmulatorError> {
        let nibs = (
            (inst & 0xF000) >> 12,
            (inst & 0x0F00) >> 8,
            (inst & 0x00F0) >> 4,
            inst & 0x000F,
        );

        match nibs {
            (0x0, 0x0, 0xE, 0x0) => {
                // clear screen
                self.screen = [0u8; SCREEN_WIDTH * SCREEN_HEIGHT];
                self.draw = true;
                Ok(())
            }
            (0x0, 0x0, 0xE, 0xE) => self.ret(),
            (0x1, _, _, _) => self.jump((inst & ADDR_MASK) as usize),
            (0x2, _, _, _) => self.call_subroutine((inst & ADDR_MASK) as usize),
            (0x3, reg, _, _) => self.skip_if_eq(reg as u8, (inst & IMM_MASK) as u8),
            (0x4, reg, _, _) => self.skip_if_not_eq(reg as u8, (inst & IMM_MASK) as u8),
            (0x5, reg1, reg2, 0x0) => self.skip_if_compare(reg1 as u8, reg2 as u8),
            // load the last two nibs into the register given by nib 2
            (0x6, reg, _, _) => self.load_imm(reg as u8, (inst & IMM_MASK) as u8),
            // add the last two nibs to the register given by nib 2
            (0x7, reg, _, _) => self.add_imm(reg as u8, (inst & IMM_MASK) as u8),
            (0x8, reg1, reg2, 0x0) => self.copy_reg(reg1 as u8, reg2 as u8),
            (0x8, reg1, reg2, 0x1) => self.or_reg(reg1 as u8, reg2 as u8),
            (0x8, reg1, reg2, 0x2) => self.and_reg(reg1 as u8, reg2 as u8),
            (0x8, reg1, reg2, 0x3) => self.xor_reg(reg1 as u8, reg2 as u8),
            (0x8, reg1, reg2, 0x4) => self.add_reg(reg1 as u8, reg2 as u8),
            (0x8, reg1, reg2, 0x5) => self.sub_reg(reg1 as u8, reg2 as u8),
            (0x8, reg1, reg2, 0x6) => self.shift_right_reg(reg1 as u8, reg2 as u8),
            (0x8, reg1, reg2, 0x7) => self.sub_reg_inverse(reg1 as u8, reg2 as u8),
            (0x8, reg1, reg2, 0xE) => self.shift_left_reg(reg1 as u8, reg2 as u8),
            (0x9, reg1, reg2, 0x0) => self.skip_if_not_compare(reg1 as u8, reg2 as u8),
            (0xA, _, _, _) => self.set_index_imm(inst & ADDR_MASK),
            (0xB, _, _, _) => self.jmp_imm(inst & ADDR_MASK),
            (0xC, reg, _, _) => self.rand(reg as u8, (inst & IMM_MASK) as u8),
            (0xD, reg1, reg2, height) => self.draw(reg1 as u8, reg2 as u8, height as u8),
            (0xE, reg, 0x9, 0xE) => self.skip_if_pressed(reg as u8),
            (0xE, reg, 0xA, 0x1) => self.skip_if_not_pressed(reg as u8),
            (0xF, reg, 0x0, 0x7) => self.load_delay_timer(reg as u8),
            (0xF, reg, 0x0, 0xA) => self.await_key_press(reg as u8),
            (0xF, reg, 0x1, 0x5) => self.set_delay_timer(reg as u8),
            (0xF, reg, 0x1, 0x8) => self.set_sound_timer(reg as u8),
            (0xF, reg, 0x1, 0xE) => self.add_index(reg as u8),
            (0xF, reg, 0x2, 0x9) => self.set_index_for_font(reg as u8),
            (0xF, reg, 0x3, 0x3) => self.store_bcd(reg as u8),
            (0xF, reg, 0x5, 0x5) => self.dump_registers(reg as u8),
            (0xF, reg, 0x6, 0x5) => self.load_registers(reg as u8),

            _ => Err(EmulatorError::UnknownOpcode(inst)),
        }
    }

    fn ret(&mut self) -> Result<(), EmulatorError> {
        if self.sp < 1 {
            return Err(EmulatorError::StackUnderflow);
        }

        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];

        Ok(())
    }

    fn jump(&mut self, addr: usize) -> Result<(), EmulatorError> {
        // note: if jumping to address 0 or 1, this will underflow
        // shouldn't happen with valid ROMs (programs should start at address 0x200)
        // and will be caught by the memory check below
        // but worth calling out here in case we missed something
        let target_addr = addr - OPCODE_SIZE;
        if target_addr > self.mem.len() - 1 - OPCODE_SIZE {
            return Err(EmulatorError::MemoryOutOfBounds {
                address: target_addr as u16,
            });
        }

        self.pc = target_addr as u16;
        Ok(())
    }

    fn call_subroutine(&mut self, addr: usize) -> Result<(), EmulatorError> {
        if self.sp == self.stack.len() as u8 {
            return Err(EmulatorError::StackOverflow);
        }

        // see the above note on underflows
        let target_addr = addr - OPCODE_SIZE;
        if target_addr > self.mem.len() - OPCODE_SIZE {
            return Err(EmulatorError::MemoryOutOfBounds {
                address: target_addr as u16,
            });
        }

        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;

        self.pc = target_addr as u16;
        Ok(())
    }

    fn load_imm(&mut self, register: u8, imm: u8) -> Result<(), EmulatorError> {
        self.validate_register(register)?;
        self.reg[register as usize] = imm;
        Ok(())
    }

    fn add_imm(&mut self, register: u8, imm: u8) -> Result<(), EmulatorError> {
        self.validate_register(register)?;
        self.reg[register as usize] = self.reg[register as usize].wrapping_add(imm);
        Ok(())
    }

    fn skip_if_eq(&mut self, register: u8, imm: u8) -> Result<(), EmulatorError> {
        self.validate_register(register)?;
        if self.reg[register as usize] == imm {
            self.pc += OPCODE_SIZE as u16;
        }

        if self.pc > (self.mem.len() - 1) as u16 {
            return Err(EmulatorError::MemoryOutOfBounds { address: self.pc });
        }

        Ok(())
    }

    fn skip_if_not_eq(&mut self, register: u8, imm: u8) -> Result<(), EmulatorError> {
        self.validate_register(register)?;
        if self.reg[register as usize] != imm {
            self.pc += OPCODE_SIZE as u16;
        }

        if self.pc > (self.mem.len() - 1) as u16 {
            return Err(EmulatorError::MemoryOutOfBounds { address: self.pc });
        }

        Ok(())
    }

    fn skip_if_compare(&mut self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;
        if self.reg[reg1 as usize] == self.reg[reg2 as usize] {
            self.pc += OPCODE_SIZE as u16;
        }

        if self.pc > (self.mem.len() - 1) as u16 {
            return Err(EmulatorError::MemoryOutOfBounds { address: self.pc });
        }

        Ok(())
    }

    fn copy_reg(&mut self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;
        self.reg[reg1 as usize] = self.reg[reg2 as usize];
        Ok(())
    }

    fn or_reg(&mut self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;

        self.reg[reg1 as usize] |= self.reg[reg2 as usize];
        Ok(())
    }

    fn and_reg(&mut self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;

        self.reg[reg1 as usize] &= self.reg[reg2 as usize];
        Ok(())
    }

    fn xor_reg(&mut self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;

        self.reg[reg1 as usize] ^= self.reg[reg2 as usize];
        Ok(())
    }

    fn add_reg(&mut self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;

        let (result, overflow) = self.reg[reg1 as usize].overflowing_add(self.reg[reg2 as usize]);
        self.reg[reg1 as usize] = result;
        self.reg[0xF] = if overflow { 1 } else { 0 };
        Ok(())
    }

    fn sub_reg(&mut self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;

        let (result, overflow) = self.reg[reg1 as usize].overflowing_sub(self.reg[reg2 as usize]);
        self.reg[reg1 as usize] = result;
        self.reg[0xF] = if overflow { 0 } else { 1 };
        Ok(())
    }

    fn shift_right_reg(&mut self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;

        self.reg[0xF] = self.reg[reg1 as usize] & 0x1;
        self.reg[reg1 as usize] >>= 1;
        Ok(())
    }

    fn sub_reg_inverse(&mut self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;

        let (result, overflow) = self.reg[reg2 as usize].overflowing_sub(self.reg[reg1 as usize]);
        self.reg[reg1 as usize] = result;
        self.reg[0xF] = if overflow { 0 } else { 1 };
        Ok(())
    }

    fn shift_left_reg(&mut self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;

        self.reg[0xF] = if self.reg[reg1 as usize] & 0x80 == 0x80 {
            1
        } else {
            0
        };
        self.reg[reg1 as usize] <<= 1;

        Ok(())
    }

    fn skip_if_not_compare(&mut self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;

        if self.reg[reg1 as usize] != self.reg[reg2 as usize] {
            self.pc += OPCODE_SIZE as u16;
        }

        if self.pc > (self.mem.len() - 1) as u16 {
            return Err(EmulatorError::MemoryOutOfBounds { address: self.pc });
        }

        Ok(())
    }

    fn set_index_imm(&mut self, imm: u16) -> Result<(), EmulatorError> {
        if imm > (self.mem.len() - 1) as u16 {
            return Err(EmulatorError::MemoryOutOfBounds { address: imm });
        }

        self.index = imm;
        Ok(())
    }

    fn jmp_imm(&mut self, imm: u16) -> Result<(), EmulatorError> {
        if imm > (self.mem.len() - 1) as u16 {
            return Err(EmulatorError::MemoryOutOfBounds { address: imm });
        }

        let target_addr = ((imm - OPCODE_SIZE as u16) + (self.reg[0] as u16)) as usize;

        if target_addr > self.mem.len() - 1 - OPCODE_SIZE {
            return Err(EmulatorError::MemoryOutOfBounds {
                address: target_addr as u16,
            });
        }

        self.pc = target_addr as u16;

        Ok(())
    }

    fn rand(&mut self, register: u8, mask: u8) -> Result<(), EmulatorError> {
        self.validate_register(register)?;
        self.reg[register as usize] = self.rng.next_u8() & mask;
        Ok(())
    }

    fn draw(&mut self, reg1: u8, reg2: u8, height: u8) -> Result<(), EmulatorError> {
        self.validate_registers(reg1, reg2)?;
        let x = self.reg[reg1 as usize] as usize;
        let y = self.reg[reg2 as usize] as usize;

        let start_addr = self.index as usize;

        self.reg[0xF] = 0;
        for row in 0..height as usize {
            if start_addr + row > self.mem.len() - 1 {
                return Err(EmulatorError::MemoryOutOfBounds {
                    address: (start_addr + row) as u16,
                });
            }

            let byte = self.mem[start_addr + row];

            // notably, one pixel is a bit of the byte in memory,
            // but each pixel is one byte of the screen (yes, a whole byte just for 0 or 1),
            // so we need to shift the byte to the correct position
            for col in 0..8u8 {
                let bit = (byte >> (7 - col)) & 0x1; // & 0x1 will set LSB and 0 all other bits
                // we only need to care about this bit if it's 1
                if bit == 0 {
                    continue;
                }

                // modulo to allow screen wrapping,
                // expected behavior by many ROMs
                let pixel_x = (x + col as usize) % SCREEN_WIDTH;
                let pixel_y = (y + row) % SCREEN_HEIGHT;

                let screen_idx = pixel_y * SCREEN_WIDTH + pixel_x;

                // we will only get here if the bit is 1, so we don't need to compare before/after
                if self.screen[screen_idx] == 1 {
                    self.reg[0xF] = 1;
                }

                // see comment above; this line only runs if the bit is 1
                self.screen[screen_idx] ^= 1;
            }
        }

        // if we drew anything, we need to update the screen
        // we don't need to check every pixel; just update the screen if we tried to draw anything
        // if redraws are very expensive, consider adding a flag in the loop for this
        self.draw = height > 0;
        Ok(())
    }

    fn skip_if_pressed(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;
        let key = self.reg[reg as usize];

        if key as usize >= self.keys.len() {
            return Err(EmulatorError::InvalidKey { key });
        }

        if self.keys[key as usize] {
            self.pc += OPCODE_SIZE as u16;
        }

        Ok(())
    }

    fn skip_if_not_pressed(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;
        let key = self.reg[reg as usize];

        if key as usize >= self.keys.len() {
            return Err(EmulatorError::InvalidKey { key });
        }

        if !self.keys[key as usize] {
            self.pc += OPCODE_SIZE as u16;
        }

        Ok(())
    }

    fn load_delay_timer(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;
        self.reg[reg as usize] = self.delay_timer;
        Ok(())
    }

    fn await_key_press(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;
        self.waiting_input = true;
        self.waiting_reg = reg as usize;

        Ok(())
    }

    fn set_delay_timer(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;
        self.delay_timer = self.reg[reg as usize];
        Ok(())
    }

    fn set_sound_timer(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;
        self.sound_timer = self.reg[reg as usize];
        Ok(())
    }

    fn add_index(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;
        self.index += self.reg[reg as usize] as u16;
        Ok(())
    }

    fn set_index_for_font(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;
        // each character is 5 bytes (8 px wide, 5 px tall).
        // they're all packed together in the font set,
        // so the memory index is the index of the first byte of the first character
        // plus the index of the register (0-F) times 5 (the number of bytes in a character).
        // see font.rs for a visual representation
        // cast self.reg[reg as usize] to usize to avoid overflow or panic on multiplying by 5
        // 0x33 * 5 is 0xFF, max value for u8
        // so if self.reg[reg] has any value greater than 0x33, this line would overflow.
        // a ROM doing this is a bug, since characters can only be 0-F, but it can happen.
        self.index = (FONT_OFFSET + (self.reg[reg as usize] as usize * 5)) as u16;

        Ok(())
    }

    fn store_bcd(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;
        if self.index as usize + 2 >= self.mem.len() {
            return Err(EmulatorError::MemoryOutOfBounds {
                address: self.index + 2,
            });
        }

        let bcd = self.reg[reg as usize];
        self.mem[self.index as usize] = bcd / 100;
        self.mem[self.index as usize + 1] = (bcd / 10) % 10;
        self.mem[self.index as usize + 2] = bcd % 10;

        Ok(())
    }

    fn dump_registers(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;

        if self.index as usize + reg as usize >= self.mem.len() {
            return Err(EmulatorError::MemoryOutOfBounds {
                address: self.index + reg as u16,
            });
        }

        for i in 0..=reg as usize {
            self.mem[self.index as usize + i] = self.reg[i];
        }

        Ok(())
    }

    fn load_registers(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;

        if self.index as usize + reg as usize >= self.mem.len() {
            return Err(EmulatorError::MemoryOutOfBounds {
                address: self.index + reg as u16,
            });
        }

        for i in 0..=reg as usize {
            self.reg[i] = self.mem[self.index as usize + i];
        }

        Ok(())
    }

    fn validate_register(&self, reg: u8) -> Result<(), EmulatorError> {
        if reg as usize >= self.reg.len() {
            return Err(EmulatorError::InvalidRegister { register: reg });
        }
        Ok(())
    }

    fn validate_registers(&self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg1)?;
        self.validate_register(reg2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RandomSource;

    struct FixedRng(u8);

    impl RandomSource for FixedRng {
        fn next_u8(&mut self) -> u8 {
            self.0
        }
    }

    fn emulator(fixed_rng: FixedRng) -> Emulator {
        Emulator::new(Box::new(fixed_rng))
    }

    // --- Clear Screen (00E0) ---

    #[test]
    fn test_clear() {
        let mut emulator = emulator(FixedRng(0));

        emulator.execute(0x00E0).unwrap();
        assert_eq!(emulator.screen, [0; SCREEN_WIDTH * SCREEN_HEIGHT]);

        emulator.screen[5] = 1;
        emulator.screen[100] = 1;

        emulator.execute(0x00E0).unwrap();

        assert_eq!(emulator.screen, [0; SCREEN_WIDTH * SCREEN_HEIGHT]);
    }

    // --- Draw (DXYN) ---

    #[test]
    fn test_draw() {
        let mut emulator = emulator(FixedRng(0));

        // draw 8 pixels starting at (x, y) = (1, 1)
        emulator.reg[0x5] = 0x1;
        emulator.reg[0x6] = 0x1;
        emulator.reg[0xF] = 0;

        emulator.index = 0x200;
        emulator.mem[0x200] = 0b11111111;

        emulator.execute(0xD561).unwrap();

        // second row starts at idx 64
        // second col starts at idx 1
        // 64 + 1 = 65, and we drew 8 bits of 1 starting at that index
        assert_eq!(&emulator.screen[65..73], &[1; 8]);
        // collision flag should NOT be set
        assert_eq!(emulator.reg[0xF], 0);
        assert!(emulator.draw);

        // draw those pixels again, this time we expect the screen to be turned to 0
        emulator.execute(0xD561).unwrap();

        assert_eq!(&emulator.screen[65..73], &[0; 8]);
        // since the pixels were already drawn, they should have been unset
        // so the collision flag should be set
        assert_eq!(emulator.reg[0xF], 1);
        assert!(emulator.draw);
    }

    #[test]
    fn test_draw_wrap() {
        let mut emulator = emulator(FixedRng(0));

        // x-wrap: draw at (x, y) = (64, 1), wraps to (0, 1)
        emulator.reg[0x5] = SCREEN_WIDTH as u8;
        emulator.reg[0x6] = 0x1;

        emulator.index = 0x200;
        emulator.mem[0x200] = 0b11111111;

        emulator.execute(0xD561).unwrap();

        assert_eq!(&emulator.screen[64..72], &[1; 8]);
        assert_eq!(emulator.reg[0xF], 0);
        assert!(emulator.draw);

        // y-wrap: draw at (x, y) = (1, 32), wraps to (1, 0)
        emulator.screen = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
        emulator.reg[0x5] = 0x1;
        emulator.reg[0x6] = SCREEN_HEIGHT as u8;

        emulator.execute(0xD561).unwrap();

        // row 0, starting at column 1
        assert_eq!(&emulator.screen[1..9], &[1; 8]);
        assert_eq!(emulator.reg[0xF], 0);
    }

    #[test]
    fn test_draw_oob_memory() {
        let mut emulator = emulator(FixedRng(0));
        emulator.reg[0] = 0;
        emulator.reg[1] = 0;
        emulator.index = 4095;
        // height=2: row 0 reads mem[4095] (valid), row 1 reads mem[4096] (OOB)
        assert!(matches!(
            emulator.execute(0xD012),
            Err(EmulatorError::MemoryOutOfBounds { .. })
        ));
    }

    // --- Stack: Call (2NNN) + Ret (00EE) ---

    #[test]
    fn test_call_and_ret() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;

        emu.execute(0x2300).unwrap();
        assert_eq!(emu.sp, 1);
        assert_eq!(emu.stack[0], initial_pc);
        assert_eq!(emu.pc, 0x300 - OPCODE_SIZE as u16);

        emu.execute(0x00EE).unwrap();
        assert_eq!(emu.sp, 0);
        assert_eq!(emu.pc, initial_pc);
    }

    #[test]
    fn test_call_nested() {
        let mut emu = emulator(FixedRng(0));

        emu.execute(0x2300).unwrap();
        emu.execute(0x2400).unwrap();
        assert_eq!(emu.sp, 2);

        emu.execute(0x00EE).unwrap();
        assert_eq!(emu.sp, 1);
        assert_eq!(emu.pc, 0x300 - OPCODE_SIZE as u16);

        emu.execute(0x00EE).unwrap();
        assert_eq!(emu.sp, 0);
    }

    #[test]
    fn test_stack_overflow() {
        let mut emu = emulator(FixedRng(0));
        emu.sp = 16;
        assert!(matches!(
            emu.execute(0x2300),
            Err(EmulatorError::StackOverflow)
        ));
    }

    #[test]
    fn test_stack_underflow() {
        let mut emu = emulator(FixedRng(0));
        assert!(matches!(
            emu.execute(0x00EE),
            Err(EmulatorError::StackUnderflow)
        ));
    }

    // --- Jump (1NNN) ---

    #[test]
    fn test_jump() {
        let mut emu = emulator(FixedRng(0));
        emu.execute(0x1300).unwrap();
        // jump subtracts OPCODE_SIZE so tick's pc += 2 lands on the target
        assert_eq!(emu.pc, 0x300 - OPCODE_SIZE as u16);
    }

    // --- Jump + V0 (BNNN) ---

    #[test]
    fn test_jmp_imm() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x10;
        emu.execute(0xB300).unwrap();
        assert_eq!(emu.pc, 0x300 - OPCODE_SIZE as u16 + 0x10);
    }

    #[test]
    fn test_jmp_imm_oob() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0xFF;
        // 0xFFF is valid alone, but 0xFFF - 2 + 0xFF = 4348 exceeds memory
        assert!(matches!(
            emu.execute(0xBFFF),
            Err(EmulatorError::MemoryOutOfBounds { .. })
        ));
    }

    // --- Skip Instructions ---

    #[test]
    fn test_skip_if_eq_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x42;
        emu.execute(0x3042).unwrap();
        assert_eq!(emu.pc, initial_pc + OPCODE_SIZE as u16);
    }

    #[test]
    fn test_skip_if_eq_no_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x41;
        emu.execute(0x3042).unwrap();
        assert_eq!(emu.pc, initial_pc);
    }

    #[test]
    fn test_skip_if_not_eq_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x41;
        emu.execute(0x4042).unwrap();
        assert_eq!(emu.pc, initial_pc + OPCODE_SIZE as u16);
    }

    #[test]
    fn test_skip_if_not_eq_no_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x42;
        emu.execute(0x4042).unwrap();
        assert_eq!(emu.pc, initial_pc);
    }

    #[test]
    fn test_skip_if_compare_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x42;
        emu.reg[1] = 0x42;
        emu.execute(0x5010).unwrap();
        assert_eq!(emu.pc, initial_pc + OPCODE_SIZE as u16);
    }

    #[test]
    fn test_skip_if_compare_no_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x42;
        emu.reg[1] = 0x43;
        emu.execute(0x5010).unwrap();
        assert_eq!(emu.pc, initial_pc);
    }

    #[test]
    fn test_skip_if_not_compare_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x42;
        emu.reg[1] = 0x43;
        emu.execute(0x9010).unwrap();
        assert_eq!(emu.pc, initial_pc + OPCODE_SIZE as u16);
    }

    #[test]
    fn test_skip_if_not_compare_no_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x42;
        emu.reg[1] = 0x42;
        emu.execute(0x9010).unwrap();
        assert_eq!(emu.pc, initial_pc);
    }

    #[test]
    fn test_skip_oob() {
        let mut emu = emulator(FixedRng(0));
        emu.pc = 4094;
        emu.reg[0] = 0x42;
        // skip condition met → pc becomes 4096 > 4095 → OOB
        assert!(matches!(
            emu.execute(0x3042),
            Err(EmulatorError::MemoryOutOfBounds { .. })
        ));
    }

    // --- Load Immediate (6XNN) ---

    #[test]
    fn test_load_imm() {
        let mut emu = emulator(FixedRng(0));
        emu.execute(0x6042).unwrap();
        assert_eq!(emu.reg[0], 0x42);
    }

    // --- Add Immediate (7XNN) ---

    #[test]
    fn test_add_imm() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x10;
        emu.execute(0x7020).unwrap();
        assert_eq!(emu.reg[0], 0x30);
    }

    #[test]
    fn test_add_imm_wraps() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0xFF;
        emu.execute(0x7001).unwrap();
        assert_eq!(emu.reg[0], 0x00);
    }

    // --- Register Operations (8XY_) ---

    #[test]
    fn test_copy_reg() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[1] = 0x42;
        emu.execute(0x8010).unwrap();
        assert_eq!(emu.reg[0], 0x42);
    }

    #[test]
    fn test_or_reg() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x0F;
        emu.reg[1] = 0xF0;
        emu.execute(0x8011).unwrap();
        assert_eq!(emu.reg[0], 0xFF);
    }

    #[test]
    fn test_and_reg() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x0F;
        emu.reg[1] = 0xF0;
        emu.execute(0x8012).unwrap();
        assert_eq!(emu.reg[0], 0x00);
    }

    #[test]
    fn test_xor_reg() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0xFF;
        emu.reg[1] = 0x0F;
        emu.execute(0x8013).unwrap();
        assert_eq!(emu.reg[0], 0xF0);
    }

    // --- Arithmetic with Flags (8XY4, 8XY5, 8XY7) ---

    #[test]
    fn test_add_reg_no_carry() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x10;
        emu.reg[1] = 0x20;
        emu.execute(0x8014).unwrap();
        assert_eq!(emu.reg[0], 0x30);
        assert_eq!(emu.reg[0xF], 0);
    }

    #[test]
    fn test_add_reg_carry() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0xFF;
        emu.reg[1] = 0x02;
        emu.execute(0x8014).unwrap();
        assert_eq!(emu.reg[0], 0x01);
        assert_eq!(emu.reg[0xF], 1);
    }

    #[test]
    fn test_sub_reg_no_borrow() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x20;
        emu.reg[1] = 0x10;
        emu.execute(0x8015).unwrap();
        assert_eq!(emu.reg[0], 0x10);
        assert_eq!(emu.reg[0xF], 1); // VF=1 means no borrow
    }

    #[test]
    fn test_sub_reg_borrow() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x10;
        emu.reg[1] = 0x20;
        emu.execute(0x8015).unwrap();
        assert_eq!(emu.reg[0], 0xF0);
        assert_eq!(emu.reg[0xF], 0); // VF=0 means borrow occurred
    }

    #[test]
    fn test_sub_reg_inverse_no_borrow() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x10;
        emu.reg[1] = 0x20;
        // VX = VY - VX = 0x20 - 0x10 = 0x10
        emu.execute(0x8017).unwrap();
        assert_eq!(emu.reg[0], 0x10);
        assert_eq!(emu.reg[0xF], 1);
    }

    #[test]
    fn test_sub_reg_inverse_borrow() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x20;
        emu.reg[1] = 0x10;
        // VX = VY - VX = 0x10 - 0x20 = underflow
        emu.execute(0x8017).unwrap();
        assert_eq!(emu.reg[0], 0xF0);
        assert_eq!(emu.reg[0xF], 0);
    }

    // --- Shifts (8XY6, 8XYE) ---

    #[test]
    fn test_shift_right_lsb_set() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0b00000011;
        emu.execute(0x8016).unwrap();
        assert_eq!(emu.reg[0], 0b00000001);
        assert_eq!(emu.reg[0xF], 1);
    }

    #[test]
    fn test_shift_right_lsb_clear() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0b00000010;
        emu.execute(0x8016).unwrap();
        assert_eq!(emu.reg[0], 0b00000001);
        assert_eq!(emu.reg[0xF], 0);
    }

    #[test]
    fn test_shift_left_msb_set() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0b10000001;
        emu.execute(0x801E).unwrap();
        assert_eq!(emu.reg[0], 0b00000010);
        assert_eq!(emu.reg[0xF], 1);
    }

    #[test]
    fn test_shift_left_msb_clear() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0b01000000;
        emu.execute(0x801E).unwrap();
        assert_eq!(emu.reg[0], 0b10000000);
        assert_eq!(emu.reg[0xF], 0);
    }

    // --- Set Index (ANNN) ---

    #[test]
    fn test_set_index_imm() {
        let mut emu = emulator(FixedRng(0));
        emu.execute(0xA300).unwrap();
        assert_eq!(emu.index, 0x300);
    }

    // --- Random (CXNN) ---

    #[test]
    fn test_rand() {
        let mut emu = emulator(FixedRng(0xFF));
        // rng returns 0xFF, mask is 0x0F → result = 0xFF & 0x0F = 0x0F
        emu.execute(0xC00F).unwrap();
        assert_eq!(emu.reg[0], 0x0F);
    }

    // --- Key Skip Instructions (EX9E, EXA1) ---

    #[test]
    fn test_skip_if_pressed_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x5;
        emu.keys[0x5] = true;
        emu.execute(0xE09E).unwrap();
        assert_eq!(emu.pc, initial_pc + OPCODE_SIZE as u16);
    }

    #[test]
    fn test_skip_if_pressed_no_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x5;
        emu.keys[0x5] = false;
        emu.execute(0xE09E).unwrap();
        assert_eq!(emu.pc, initial_pc);
    }

    #[test]
    fn test_skip_if_not_pressed_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x5;
        emu.keys[0x5] = false;
        emu.execute(0xE0A1).unwrap();
        assert_eq!(emu.pc, initial_pc + OPCODE_SIZE as u16);
    }

    #[test]
    fn test_skip_if_not_pressed_no_match() {
        let mut emu = emulator(FixedRng(0));
        let initial_pc = emu.pc;
        emu.reg[0] = 0x5;
        emu.keys[0x5] = true;
        emu.execute(0xE0A1).unwrap();
        assert_eq!(emu.pc, initial_pc);
    }

    #[test]
    fn test_skip_if_pressed_invalid_key() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x10; // key value > 0xF
        assert!(matches!(
            emu.execute(0xE09E),
            Err(EmulatorError::InvalidKey { key: 0x10 })
        ));
    }

    #[test]
    fn test_skip_if_not_pressed_invalid_key() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0xFF;
        assert!(matches!(
            emu.execute(0xE0A1),
            Err(EmulatorError::InvalidKey { key: 0xFF })
        ));
    }

    // --- Timers (FX07, FX15, FX18) ---

    #[test]
    fn test_load_delay_timer() {
        let mut emu = emulator(FixedRng(0));
        emu.delay_timer = 0x42;
        emu.execute(0xF007).unwrap();
        assert_eq!(emu.reg[0], 0x42);
    }

    #[test]
    fn test_set_delay_timer() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x30;
        emu.execute(0xF015).unwrap();
        assert_eq!(emu.delay_timer, 0x30);
    }

    #[test]
    fn test_set_sound_timer() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x20;
        emu.execute(0xF018).unwrap();
        assert_eq!(emu.sound_timer, 0x20);
    }

    // --- Await Key Press (FX0A) ---

    #[test]
    fn test_await_key_press() {
        let mut emu = emulator(FixedRng(0));
        emu.execute(0xF30A).unwrap();
        assert!(emu.waiting_input);
        assert_eq!(emu.waiting_reg, 3);
    }

    // --- Add Index (FX1E) ---

    #[test]
    fn test_add_index() {
        let mut emu = emulator(FixedRng(0));
        emu.index = 0x100;
        emu.reg[0] = 0x50;
        emu.execute(0xF01E).unwrap();
        assert_eq!(emu.index, 0x150);
    }

    // --- Set Index for Font (FX29) ---

    #[test]
    fn test_set_index_for_font() {
        let mut emu = emulator(FixedRng(0));
        emu.reg[0] = 0x0;
        emu.execute(0xF029).unwrap();
        assert_eq!(emu.index, FONT_OFFSET as u16);

        emu.reg[0] = 0xF;
        emu.execute(0xF029).unwrap();
        assert_eq!(emu.index, (FONT_OFFSET + 15 * 5) as u16);
    }

    // --- Store BCD (FX33) ---

    #[test]
    fn test_store_bcd() {
        let mut emu = emulator(FixedRng(0));
        emu.index = 0x300;
        emu.reg[0] = 254;
        emu.execute(0xF033).unwrap();
        assert_eq!(emu.mem[0x300], 2); // hundreds
        assert_eq!(emu.mem[0x301], 5); // tens
        assert_eq!(emu.mem[0x302], 4); // ones
    }

    #[test]
    fn test_store_bcd_zero() {
        let mut emu = emulator(FixedRng(0));
        emu.index = 0x300;
        emu.reg[0] = 0;
        emu.execute(0xF033).unwrap();
        assert_eq!(emu.mem[0x300], 0);
        assert_eq!(emu.mem[0x301], 0);
        assert_eq!(emu.mem[0x302], 0);
    }

    #[test]
    fn test_store_bcd_oob() {
        let mut emu = emulator(FixedRng(0));
        // needs 3 bytes at index: I, I+1, I+2
        // index 4094: I+2 = 4096 >= 4096 → OOB
        emu.index = 4094;
        assert!(matches!(
            emu.execute(0xF033),
            Err(EmulatorError::MemoryOutOfBounds { .. })
        ));
    }

    // --- Dump/Load Registers (FX55, FX65) ---

    #[test]
    fn test_dump_registers() {
        let mut emu = emulator(FixedRng(0));
        emu.index = 0x300;
        emu.reg[0] = 0xAA;
        emu.reg[1] = 0xBB;
        emu.reg[2] = 0xCC;
        emu.reg[3] = 0xDD;
        // dump V0-V3
        emu.execute(0xF355).unwrap();
        assert_eq!(emu.mem[0x300], 0xAA);
        assert_eq!(emu.mem[0x301], 0xBB);
        assert_eq!(emu.mem[0x302], 0xCC);
        assert_eq!(emu.mem[0x303], 0xDD);
    }

    #[test]
    fn test_load_registers() {
        let mut emu = emulator(FixedRng(0));
        emu.index = 0x300;
        emu.mem[0x300] = 0x11;
        emu.mem[0x301] = 0x22;
        emu.mem[0x302] = 0x33;
        // load V0-V2
        emu.execute(0xF265).unwrap();
        assert_eq!(emu.reg[0], 0x11);
        assert_eq!(emu.reg[1], 0x22);
        assert_eq!(emu.reg[2], 0x33);
    }

    #[test]
    fn test_dump_load_round_trip() {
        let mut emu = emulator(FixedRng(0));
        emu.index = 0x300;
        emu.reg[0] = 0xAA;
        emu.reg[1] = 0xBB;
        emu.reg[2] = 0xCC;

        emu.execute(0xF255).unwrap(); // dump V0-V2

        emu.reg[0] = 0;
        emu.reg[1] = 0;
        emu.reg[2] = 0;

        emu.execute(0xF265).unwrap(); // load V0-V2

        assert_eq!(emu.reg[0], 0xAA);
        assert_eq!(emu.reg[1], 0xBB);
        assert_eq!(emu.reg[2], 0xCC);
    }

    #[test]
    fn test_dump_registers_oob() {
        let mut emu = emulator(FixedRng(0));
        emu.index = 4094;
        // dump V0-V3: needs indices 4094..4097, but 4094+3 = 4097 >= 4096 → OOB
        assert!(matches!(
            emu.execute(0xF355),
            Err(EmulatorError::MemoryOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_load_registers_oob() {
        let mut emu = emulator(FixedRng(0));
        emu.index = 4094;
        assert!(matches!(
            emu.execute(0xF365),
            Err(EmulatorError::MemoryOutOfBounds { .. })
        ));
    }

    // --- Unknown Opcode ---

    #[test]
    fn test_unknown_opcode() {
        let mut emu = emulator(FixedRng(0));
        assert!(matches!(
            emu.execute(0x0000),
            Err(EmulatorError::UnknownOpcode(0x0000))
        ));
    }

    #[test]
    fn test_unknown_opcode_bad_suffix() {
        let mut emu = emulator(FixedRng(0));
        // 0x5XY0 is valid, but 0x5XY1 is not
        assert!(matches!(
            emu.execute(0x5001),
            Err(EmulatorError::UnknownOpcode(0x5001))
        ));
    }
}
