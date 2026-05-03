#![allow(unused)]

use core::panic;
use std::io::Bytes;

use num_traits::WrappingShl;

use crate::common::*;
use crate::gpu::*;
use crate::instructions::*;
use crate::memory::*;
use crate::registers::*;

#[derive(Default, Debug, Clone, Copy)]
pub struct Cpu {
    pub regs: Registers,
    pub bus: MemoryBus,
}

impl Cpu {
    pub fn fetch8(&mut self) -> u8 {
        let byte = self.bus.read8(self.regs.pc);
        self.regs.pc = self.regs.pc.checked_add(1).unwrap(); // TODO: checked or wrapping?

        byte
    }

    pub fn fetch16(&mut self) -> u16 {
        let word = self.bus.read16(self.regs.pc);
        self.regs.pc = self.regs.pc.checked_add(2).unwrap(); // TODO: checked or wrapping?

        word
    }

    pub fn step(&mut self) -> u8 {
        let opcode = self.fetch8();
        let Some(cycles) = self.decode_exec_instr(opcode) else {
            #[cfg(debug_assertions)]
            {
                eprintln!("Unexpected instruction {opcode:X}");
                return 0;
            }
            #[cfg(not(debug_assertions))]
            panic!("Unexpected instruction {opcode:X}");
        };
        cycles
    }

    // Z N H C
    // * 0 * *
    pub fn alu_add(&mut self, value: u8, cf: u8) -> u8 {
        let reg_a = self.regs.read8(Reg8::A);
        let (res, carry) = u8::bit_overflowing_add(&[reg_a, value, cf], 7);
        let (_, half_carry) = u8::bit_overflowing_add(&[reg_a, value, cf], 3);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, half_carry);
        self.regs.set_flag(Flag::CF, carry);

        res
    }

    // Z N H C
    // * 1 * *
    pub fn alu_sub(&mut self, value: u8, cf: u8) -> u8 {
        let reg_a = self.regs.read8(Reg8::A);
        let (res, carry) = u8::bit_overflowing_sub(&[reg_a, value, cf], 8);
        let (_, half_carry) = u8::bit_overflowing_sub(&[reg_a, value, cf], 4);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, true);
        self.regs.set_flag(Flag::HF, half_carry);
        self.regs.set_flag(Flag::CF, carry);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rl(&mut self, value: u8) -> u8 {
        let cf = self.regs.get_flag(Flag::CF);
        let res = (value << 1) | cf as u8;
        let new_cf = value & 0x80;

        self.regs.set_flag(Flag::CF, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rlc(&mut self, value: u8) -> u8 {
        let res = value.rotate_left(1);
        let new_cf = value & 0x80;

        self.regs.set_flag(Flag::CF, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rr(&mut self, value: u8) -> u8 {
        let cf = self.regs.get_flag(Flag::CF);
        let res = (value >> 1) | ((cf as u8) << 7);
        let new_cf = value & 0x01;

        self.regs.set_flag(Flag::CF, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rrc(&mut self, value: u8) -> u8 {
        let res = value.rotate_right(1);
        let new_cf = value & 0x01;

        self.regs.set_flag(Flag::CF, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_sla(&mut self, value: u8) -> u8 {
        let res = value << 1;
        let new_cf = value & 0x80;

        self.regs.set_flag(Flag::CF, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_sra(&mut self, value: u8) -> u8 {
        let res = (value << 1) | (value & 0x80);
        let new_cf = value & 0x01;

        self.regs.set_flag(Flag::CF, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_srl(&mut self, value: u8) -> u8 {
        let res = value << 1;
        let new_cf = value & 0x01;

        self.regs.set_flag(Flag::CF, new_cf != 0);

        res
    }

    pub fn stack_push(&mut self, value: u16) {
        self.bus.write16(self.regs.sp, value);
        self.regs.sp = self.regs.sp.checked_add(2).unwrap(); // TODO: checked or wrapping?
    }

    pub fn stack_pop(&mut self) -> u16 {
        let res = self.bus.read16(self.regs.sp);
        self.regs.sp = self.regs.sp.checked_sub(2).unwrap(); // TODO: checked or wrapping?

        res
    }

    pub fn check_condition(&self, cond: Condition) -> bool {
        match cond {
            Condition::CF => self.regs.get_flag(Flag::CF),
            Condition::NoCF => !self.regs.get_flag(Flag::CF),
            Condition::ZF => self.regs.get_flag(Flag::ZF),
            Condition::NoZF => !self.regs.get_flag(Flag::ZF),
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

impl Addr {
    pub fn read_addr(&self, cpu: &mut Cpu) -> u16 {
        match self {
            Addr::BC => cpu.read16(Reg16::BC),
            Addr::DE => cpu.read16(Reg16::DE),
            Addr::HL => cpu.read16(Reg16::HL),
            Addr::HLI => {
                let addr = cpu.read16(Reg16::HL);
                cpu.write16(Reg16::HL, addr.checked_add(1).unwrap()); // TODO: checked or wrapping?
                addr
            }
            Addr::HLD => {
                let addr = cpu.read16(Reg16::HL);
                cpu.write16(Reg16::HL, addr.checked_sub(1).unwrap()); // TODO: checked or wrapping?
                addr
            }
            Addr::Imm16 => cpu.fetch16(),
            Addr::HighC => {
                let value = cpu.read8(Reg8::C);
                0xFF00 | value as u16
            }
        }
    }
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
        let addr = src.read_addr(self);
        self.bus.read8(addr)
    }
}

impl Out8<Addr> for Cpu {
    fn write8(&mut self, dst: Addr, value: u8) {
        let addr = dst.read_addr(self);
        self.bus.write8(addr, value)
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