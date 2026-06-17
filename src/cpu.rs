use crate::common::*;
use crate::gameboy::Gameboy;
use crate::interrupt::Interrupt;
use crate::registers::*;

pub struct Cpu {
    pub regs: Registers,
    pub ime: bool,
    pub low_power_mode: bool,
    pub halt_bug: bool,
    pub pending_enable_ime: bool,
    pub gb: *mut Gameboy,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            regs: Default::default(),
            ime: false,
            low_power_mode: false,
            halt_bug: false,
            pending_enable_ime: false,
            gb: std::ptr::null_mut(),
        }
    }

    pub fn gb(&mut self) -> &mut Gameboy {
        assert!(!self.gb.is_null(), "Expected valid pointer");
        // SAFETY: this is used to access data inside gb that's not already "in scope" (eg. cpu.gb().timer), so aliasing *shouldn't* be an issue
        // TODO: this is still pretty unsafe and should be removed
        unsafe { self.gb.as_mut().unwrap() }
    }

    pub fn fetch8(&mut self) -> u8 {
        let pc = self.regs.pc;
        let byte = self.gb().read8(pc);
        self.regs.pc = pc.checked_add(1).unwrap(); // TODO: checked or wrapping?

        byte
    }

    pub fn fetch16(&mut self) -> u16 {
        let pc = self.regs.pc;
        let word = self.gb().read16(pc);
        self.regs.pc = pc.checked_add(2).unwrap(); // TODO: checked or wrapping?

        word
    }

    pub fn read_addr(&mut self, addr: Addr) -> u16 {
        match addr {
            Addr::BC => self.read16(Reg16::BC),
            Addr::DE => self.read16(Reg16::DE),
            Addr::HL => self.read16(Reg16::HL),
            Addr::HLI => {
                let addr = self.read16(Reg16::HL);
                self.write16(Reg16::HL, addr.checked_add(1).unwrap()); // TODO: checked or wrapping?
                addr
            }
            Addr::HLD => {
                let addr = self.read16(Reg16::HL);
                self.write16(Reg16::HL, addr.checked_sub(1).unwrap()); // TODO: checked or wrapping?
                addr
            }
            Addr::Imm16 => self.fetch16(),
            Addr::HighC => {
                let value = self.read8(Reg8::C);
                0xFF00 | value as u16
            }
        }
    }

    fn handle_interrupts(&mut self) -> Option<u8> {
        let bits = *self.gb().inter_enable & *self.gb().inter_flag & Interrupt::BITMASK;
        let inter = (1 << bits.trailing_zeros()) as u8; // in case of multiple interrupts, the lowest bit has priority
        if !self.ime || inter == 0 {
            return None;
        }
        self.ime = false;
        *self.gb().inter_flag &= !inter;
        let addr = match inter {
            0x01 => 0x40,
            0x02 => 0x48,
            0x04 => 0x50,
            0x08 => 0x58,
            0x10 => 0x60,
            _ => unreachable!(),
        };
        self.call(addr);

        Some(5)
    }

    pub fn step(&mut self) -> u8 {
        if let Some(cycles) = self.handle_interrupts() {
            return cycles;
        }
        if self.low_power_mode {
            return 1;   // NOOP
        }

        let opcode = self.fetch8();
        let cycles = self.decode_exec_instr(opcode);

        if self.pending_enable_ime {
            self.pending_enable_ime = false;
            self.ime = true;
        }
        if self.halt_bug {
            self.halt_bug = false;
            self.regs.pc = self.regs.pc.wrapping_sub(1);
        }

        cycles
    }

    // Z N H C
    // * 0 * *
    pub fn alu_add(&mut self, value: u8, cf: u8) -> u8 {
        let reg_a = self.regs.read8(Reg8::A);
        let (res, carry) = u8::bit_overflowing_add(&[reg_a, value, cf], 7);
        let (_, half_carry) = u8::bit_overflowing_add(&[reg_a, value, cf], 3);

        self.regs.set_flag(Flag::Z, res == 0);
        self.regs.set_flag(Flag::N, false);
        self.regs.set_flag(Flag::H, half_carry);
        self.regs.set_flag(Flag::C, carry);

        res
    }

    // Z N H C
    // * 1 * *
    pub fn alu_sub(&mut self, value: u8, cf: u8) -> u8 {
        let reg_a = self.regs.read8(Reg8::A);
        let (res, carry) = u8::bit_overflowing_sub(&[reg_a, value, cf], 8);
        let (_, half_carry) = u8::bit_overflowing_sub(&[reg_a, value, cf], 4);

        self.regs.set_flag(Flag::Z, res == 0);
        self.regs.set_flag(Flag::N, true);
        self.regs.set_flag(Flag::H, half_carry);
        self.regs.set_flag(Flag::C, carry);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rl(&mut self, value: u8) -> u8 {
        let cf = self.regs.get_flag(Flag::C);
        let res = (value << 1) | cf as u8;
        let new_cf = value & 0x80;

        self.regs.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rlc(&mut self, value: u8) -> u8 {
        let res = value.rotate_left(1);
        let new_cf = value & 0x80;

        self.regs.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rr(&mut self, value: u8) -> u8 {
        let cf = self.regs.get_flag(Flag::C);
        let res = (value >> 1) | ((cf as u8) << 7);
        let new_cf = value & 0x01;

        self.regs.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rrc(&mut self, value: u8) -> u8 {
        let res = value.rotate_right(1);
        let new_cf = value & 0x01;

        self.regs.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_sla(&mut self, value: u8) -> u8 {
        let res = value << 1;
        let new_cf = value & 0x80;

        self.regs.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_sra(&mut self, value: u8) -> u8 {
        let res = (value << 1) | (value & 0x80);
        let new_cf = value & 0x01;

        self.regs.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_srl(&mut self, value: u8) -> u8 {
        let res = value << 1;
        let new_cf = value & 0x01;

        self.regs.set_flag(Flag::C, new_cf != 0);

        res
    }

    pub fn stack_push(&mut self, value: u16) {
        let sp = self.regs.sp;
        self.gb().write16(sp, value);
        self.regs.sp = sp.checked_add(2).unwrap(); // TODO: checked or wrapping?
    }

    pub fn stack_pop(&mut self) -> u16 {
        let sp = self.regs.sp;
        let res = self.gb().read16(sp);
        self.regs.sp = sp.checked_sub(2).unwrap(); // TODO: checked or wrapping?

        res
    }

    pub fn check_condition(&self, cond: Condition) -> bool {
        match cond {
            Condition::CF => self.regs.get_flag(Flag::C),
            Condition::NoCF => !self.regs.get_flag(Flag::C),
            Condition::ZF => self.regs.get_flag(Flag::Z),
            Condition::NoZF => !self.regs.get_flag(Flag::Z),
        }
    }

    pub fn jump_rel(&mut self, offset: i8) {
        self.regs.pc = self.regs.pc.checked_add_signed(offset as i16).unwrap(); // TODO: checked or wrapping?
    }

    pub fn jump_abs(&mut self, addr: u16) {
        self.regs.pc = addr;
    }

    pub fn call(&mut self, addr: u16) {
        self.stack_push(self.regs.pc);
        self.jump_abs(addr);
    }
}

pub enum Condition {
    CF,
    NoCF,
    ZF,
    NoZF,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Addr {
    BC,
    DE,
    HL,
    HLI,
    HLD,
    Imm16,
    HighC,
}

#[derive(Clone, Copy)]
pub struct Imm8;

#[derive(Clone, Copy)]
pub struct Imm16;

pub trait In8<T: Copy> {
    fn read8(&mut self, src: T) -> u8;
}

pub trait Out8<T: Copy> {
    fn write8(&mut self, dst: T, value: u8);
}

pub trait In16<T: Copy> {
    fn read16(&mut self, src: T) -> u16;
}

pub trait Out16<T: Copy> {
    fn write16(&mut self, dst: T, value: u16);
}

impl In8<Reg8> for Cpu {
    fn read8(&mut self, src: Reg8) -> u8 {
        self.regs.read8(src)
    }
}

impl Out8<Reg8> for Cpu {
    fn write8(&mut self, dst: Reg8, value: u8) {
        self.regs.write8(dst, value);
    }
}

impl In16<Reg16> for Cpu {
    fn read16(&mut self, src: Reg16) -> u16 {
        self.regs.read16(src)
    }
}

impl Out16<Reg16> for Cpu {
    fn write16(&mut self, dst: Reg16, value: u16) {
        self.regs.write16(dst, value);
    }
}

impl In8<Addr> for Cpu {
    fn read8(&mut self, src: Addr) -> u8 {
        let addr = self.read_addr(src);
        self.gb().read8(addr)
    }
}

impl Out8<Addr> for Cpu {
    fn write8(&mut self, dst: Addr, value: u8) {
        let addr = self.read_addr(dst);
        self.gb().write8(addr, value)
    }
}

impl In16<SP> for Cpu {
    fn read16(&mut self, _: SP) -> u16 {
        self.regs.sp
    }
}

impl Out16<SP> for Cpu {
    fn write16(&mut self, _: SP, value: u16) {
        self.regs.sp = value;
    }
}

impl In8<Imm8> for Cpu {
    fn read8(&mut self, _: Imm8) -> u8 {
        self.fetch8()
    }
}

impl In16<Imm16> for Cpu {
    fn read16(&mut self, _: Imm16) -> u16 {
        self.fetch16()
    }
}
