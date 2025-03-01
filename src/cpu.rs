#![allow(unused)]

use core::panic;
use std::io::Bytes;

use crate::common::*;
use crate::gpu::*;
use crate::instructions::*;
use crate::registers::*;

struct MemoryBus {
    memory: [u8; u16::MAX as usize],
    gpu: GPU,
}

impl MemoryBus {
    fn read_byte(&self, addr: u16) -> u8 {
        if (VRAM_BEG..VRAM_END).contains(&addr) {
            self.gpu.read_byte(addr)
        } else {
            self.memory[addr as usize]
        }
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        if (VRAM_BEG..VRAM_END).contains(&addr) {
            self.gpu.write_byte(addr, byte);
        } else {
            self.memory[addr as usize] = byte;
        }
    }
}

struct CPU {
    pc: u16,
    sp: u16,
    regs: Registers,
    bus: MemoryBus,
}

impl CPU {
    fn exec(&mut self, instr: Instruction) -> u16 {
        match instr {
            Instruction::ADD() => {
                let src = self.regs.c;
                let res = self.add(src);
                self.regs.a = res;
                self.pc.wrapping_add(1)
            }
            Instruction::JP(jk) => {
                let check = match jk {
                    JumpKind::Unconditional => true,
                    JumpKind::Carry => self.regs.get_flags().carry,
                    JumpKind::NotCarry => !self.regs.get_flags().carry,
                    JumpKind::Zero => self.regs.get_flags().zero,
                    JumpKind::NotZero => !self.regs.get_flags().zero,
                };

                match check {
                    true => self.jump(),
                    false => 3,
                }
            }
            Instruction::LD(kind) => match kind {
                LoadKind::Byte(src_reg, dst_reg) => {
                    let src = self.read_reg(src_reg);
                    self.write_reg(dst_reg, src);

                    match src_reg {
                        RegKind::D8 => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
                _ => todo!()
            },
            _ => todo!(),
        }
    }

    fn read_op(&self, addr: u16) -> Opcode {
        let mut byte = self.bus.read_byte(addr);
        let mut extended = false;

        if byte == 0xCB {
            extended = true;
            byte = self.bus.read_byte(addr + 1);
        }

        Opcode { byte, extended }
    }

    fn step(&mut self) {
        let opcode = self.read_op(self.pc);

        let next_pc = match Instruction::from_opcode(&opcode) {
            Some(instruction) => self.exec(instruction),
            _ => {
                panic!("Unkown instruction found for: {:?}", opcode);
            }
        };

        self.pc = next_pc;
    }

    fn jump(&self) -> u16 {
        // little-endian
        let low = self.bus.read_byte(self.pc + 1) as u16;
        let high = self.bus.read_byte(self.pc + 2) as u16;
        (high << 8) | low
    }

    fn add(&mut self, src: u8) -> u8 {
        let (res, cf) = self.regs.a.overflowing_add(src);

        self.regs.set_flags(Flags {
            carry: cf,
            half_carry: (self.regs.a & 0xF) + (res & 0xF) > 0xF,
            subtraction: false,
            zero: res == 0,
        });

        res
    }

    pub fn read_reg(&self, kind: RegKind) -> u16 {
        match kind {
            RegKind::A => self.regs.a as u16,
            RegKind::B => self.regs.b as u16,
            RegKind::C => self.regs.c as u16,
            RegKind::D => self.regs.d as u16,
            RegKind::E => self.regs.e as u16,
            RegKind::F => self.regs.f as u16,
            RegKind::H => self.regs.h as u16,
            RegKind::L => self.regs.l as u16,
            RegKind::AF => self.regs.read_af(),
            RegKind::BC => self.regs.read_bc(),
            RegKind::DE => self.regs.read_de(),
            RegKind::HL => self.regs.read_hl(),
            RegKind::D8 => self.bus.read_byte(self.pc) as u16,
            RegKind::HLI => self.bus.read_byte(self.regs.read_hl()) as u16,
        }
    }

    pub fn write_reg(&mut self, kind: RegKind, value: u16) {
        match kind {
            RegKind::A => self.regs.a = value as u8,
            RegKind::B => self.regs.b = value as u8,
            RegKind::C => self.regs.c = value as u8,
            RegKind::D => self.regs.d = value as u8,
            RegKind::E => self.regs.e = value as u8,
            RegKind::F => self.regs.f = value as u8,
            RegKind::H => self.regs.h = value as u8,
            RegKind::L => self.regs.l = value as u8,
            RegKind::AF => self.regs.write_af(value),
            RegKind::BC => self.regs.write_bc(value),
            RegKind::DE => self.regs.write_de(value),
            RegKind::HL => self.regs.write_hl(value),
            RegKind::HLI => self.bus.write_byte(self.regs.read_hl(), value as u8),
            RegKind::D8 => panic!("Can't write to register D8"),
        }
    }
}
