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
        let pc: u16 = self.regs.pc();
        let byte = self.bus.read8(pc);
        self.regs.set_pc(pc.checked_add(1).unwrap()); // TODO: checked or wrapping?

        byte
    }

    pub fn fetch16(&mut self) -> u16 {
        let lo = self.fetch8();
        let hi = self.fetch8();

        u16::from_le_bytes([lo, hi])
    }

    pub fn step(&mut self) -> usize {
        self.exec_next_instr();
        1
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
        let cf = self.regs.flag(Flag::CF);
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
        let cf = self.regs.flag(Flag::CF);
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
        let reg_sp = self.regs.sp();
        self.bus.write16(reg_sp, value);
        self.regs.set_sp(reg_sp.checked_add(2).unwrap()); // TODO: checked or wrapping?
    }

    pub fn stack_pop(&mut self) -> u16 {
        let reg_sp = self.regs.sp();
        let res = self.bus.read16(reg_sp);
        self.regs.set_sp(reg_sp.checked_sub(2).unwrap()); // TODO: checked or wrapping?

        res
    }

    pub fn check_condition(&self, cc: ConditionCode) -> bool {
        match cc {
            ConditionCode::CF => self.regs.flag(Flag::CF),
            ConditionCode::NoCF => !self.regs.flag(Flag::CF),
            ConditionCode::ZF => self.regs.flag(Flag::ZF),
            ConditionCode::NoZF => !self.regs.flag(Flag::ZF),
        }
    }

    pub fn call(&mut self, addr: u16) {
        self.stack_push(self.regs.pc());
        self.regs.set_pc(addr);
    }

    pub fn jump_rel(&mut self, offset: i8) {
        let reg_pc = self.regs.pc();
        self.regs
            .set_pc(reg_pc.checked_add_signed(offset as i16).unwrap()); // TODO: checked or wrapping?
    }
}
