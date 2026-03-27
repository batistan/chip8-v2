use crate::{Emulator, EmulatorError};
use crate::consts::{ADDR_MASK, FONT_OFFSET, IMM_MASK, OPCODE_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};

impl Emulator {
    pub(crate) fn execute(&mut self, inst: u16) -> Result<(), EmulatorError> {
        let nibs = (
            (inst & 0xF000) >> 12,
            (inst & 0x0F00) >> 8,
            (inst & 0x00F0) >> 4,
            inst & 0x000F
        );

        match nibs {
            (0x0, 0x0, 0xE, 0x0) => { // clear screen
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
            return Err(EmulatorError::MemoryOutOfBounds { address: target_addr as u16 });
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
            return Err(EmulatorError::MemoryOutOfBounds { address: target_addr as u16 });
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

        self.reg[0xF] = if self.reg[reg1 as usize] & 0x80 == 0x80 { 1 } else { 0 };
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
            return Err(EmulatorError::MemoryOutOfBounds { address: target_addr as u16 });
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
                return Err(EmulatorError::MemoryOutOfBounds { address: (start_addr + row) as u16 });
            }

            let byte = self.mem[start_addr + row];

            // notably, one pixel is a bit of the byte in memory,
            // but each pixel is one byte of the screen (yes, a whole byte just for 0 or 1),
            // so we need to shift the byte to the correct position
            for col in 0..8u8 {
                let bit = (byte >> (7 - col)) & 0x1; // & 0x1 will set LSB and 0 all other bits
                // we only need to care about this bit if it's 1
                if bit == 0 { continue; }

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
            return Err(EmulatorError::MemoryOutOfBounds { address: self.index + 2 });
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
            return Err(EmulatorError::MemoryOutOfBounds { address: self.index + reg as u16 });
        }

        for i in 0..=reg as usize {
            self.mem[self.index as usize + i] = self.reg[i];
        }

        Ok(())
    }

    fn load_registers(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.validate_register(reg)?;

        if self.index as usize + reg as usize >= self.mem.len() {
            return Err(EmulatorError::MemoryOutOfBounds { address: self.index + reg as u16 });
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
