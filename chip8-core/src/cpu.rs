use crate::{Emulator, EmulatorError};
use crate::consts::{ADDR_MASK, IMM_MASK, OPCODE_SIZE};

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
                self.screen = [0u8; 64 * 32];
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
        if register > (self.reg.len() - 1) as u8 {
            return Err(EmulatorError::InvalidRegister { register });
        }

        self.reg[register as usize] = imm;
        Ok(())
    }

    fn add_imm(&mut self, register: u8, imm: u8) -> Result<(), EmulatorError> {
        if register > (self.reg.len() - 1) as u8 {
            return Err(EmulatorError::InvalidRegister { register });
        }

        self.reg[register as usize] = self.reg[register as usize].wrapping_add(imm);
        Ok(())
    }

    fn skip_if_eq(&mut self, register: u8, imm: u8) -> Result<(), EmulatorError> {
        if register > (self.reg.len() - 1) as u8 {
            return Err(EmulatorError::InvalidRegister { register });
        }

        if self.reg[register as usize] == imm {
            self.pc += OPCODE_SIZE as u16;
        }

        if self.pc > (self.mem.len() - 1) as u16 {
            return Err(EmulatorError::MemoryOutOfBounds { address: self.pc });
        }

        Ok(())
    }

    fn skip_if_not_eq(&mut self, register: u8, imm: u8) -> Result<(), EmulatorError> {
        if register > (self.reg.len() - 1) as u8 {
            return Err(EmulatorError::InvalidRegister { register });
        }

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
        if register > (self.reg.len() - 1) as u8 {
            return Err(EmulatorError::InvalidRegister { register });
        }

        self.reg[register as usize] = self.rng.next_u8() & mask;
        Ok(())
    }

    fn draw(&mut self, reg1: u8, reg2: u8, height: u8) -> Result<(), EmulatorError> {
        // self.validate_registers(reg1, reg2)?;
        //
        // let buff = &self.mem[self.index as usize.. height as usize];
        //
        // buff.iter().enumerate().for_each(|(y, row)| {
        // });
        // Ok(())
        todo!()
    }

    fn validate_registers(&self, reg1: u8, reg2: u8) -> Result<(), EmulatorError> {
        if reg1 > (self.reg.len() - 1) as u8 {
            return Err(EmulatorError::InvalidRegister { register: reg1 });
        }
        if reg2 > (self.reg.len() - 1) as u8 {
            return Err(EmulatorError::InvalidRegister { register: reg2 });
        }

        Ok(())
    }
}
