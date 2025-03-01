#![allow(unused)]

use core::panic;
use std::io::Bytes;

use crate::common::*;
use crate::gpu::*;
use crate::instructions::*;
use crate::memory::*;
use crate::registers::*;

pub struct MemoryBus {
    memory: [u8; u16::MAX as usize],
    gpu: Gpu,
}

impl MemoryBus {
    pub fn read8(&self, addr: u16) -> u8 {
        if MM_VRAM.contains(&(addr as usize)) {
            self.gpu.read8(addr)
        } else {
            self.memory[addr as usize]
        }
    }

    pub fn write8(&mut self, addr: u16, byte: u8) {
        if MM_VRAM.contains(&(addr as usize)) {
            self.gpu.write8(addr, byte);
        } else {
            self.memory[addr as usize] = byte;
        }
    }
}

pub struct Cpu {
    pub pc: u16,
    pub sp: u16,
    pub regs: Registers,
    pub bus: MemoryBus,
}

impl Cpu {
    pub fn read8(&self, addr: u16) -> u8 {
        self.bus.read8(addr)
    }

    pub fn fetch8(&mut self) -> u8 {
        let byte = self.read8(self.pc);
        self.pc = self.pc.wrapping_add(1);

        byte
    }

    pub fn fetch16(&mut self) -> u16 {
        let lo = self.fetch8();
        let hi = self.fetch8();

        u16::from_le_bytes([lo, hi])
    }

    fn ex(&mut self) {
        todo!()
    }

    fn ex2(&mut self) {
        todo!()
    }

    fn exec(&mut self, instr: Instruction) -> u16 {
        match instr {
            Instruction::ADD() => {
                let src = self.regs.c;
                let res = self.add(src);
                self.regs.a = res;
                self.pc.wrapping_add(1)
            }
            _ => todo!(),
        }
    }

    fn read_op(&self, addr: u16) -> Opcode {
        let mut byte = self.bus.read8(addr);
        let mut extended = false;

        if byte == 0xCB {
            extended = true;
            byte = self.bus.read8(addr + 1);
        }

        Opcode { byte, extended }
    }

    fn step(&mut self) {
        let opcode = self.read_op(self.pc);
        let instr = todo!();
        let next_pc = self.exec(instr);
        self.pc = next_pc;
    }

    fn jump(&self) -> u16 {
        let lo = self.bus.read8(self.pc + 1);
        let hi = self.bus.read8(self.pc + 2);

        u16::from_le_bytes([lo, hi])
    }

    fn add(&mut self, src: u8) -> u8 {
        let (result, carry) = self.regs.a.overflowing_add(src);
        let half_carry = (self.regs.a & 0x0f).checked_add(src | 0xf0).is_none();

        self.regs.f.set(Flag::Carry, carry);
        self.regs.f.set(Flag::HalfCarry, half_carry);
        self.regs.f.set(Flag::Subtraction, false);
        self.regs.f.set(Flag::Zero, result == 0);

        result
    }
}
