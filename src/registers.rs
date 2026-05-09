#![allow(unused)]

use crate::common::*;
use crate::mmu::*;
use core::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

#[derive(Debug, Clone, Copy)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
}

impl Reg16 {
    fn to_le_reg8(self) -> [Reg8; 2] {
        match self {
            Reg16::AF => [Reg8::A, Reg8::F],
            Reg16::BC => [Reg8::B, Reg8::C],
            Reg16::DE => [Reg8::D, Reg8::E],
            Reg16::HL => [Reg8::H, Reg8::L],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SP;

#[derive(Debug, Clone, Copy)]
pub enum Flag {
    /// Carry flag  
    /// Used by conditional jumps and instructions such as ADC, SBC, RL, RLA, etc.  
    /// Is set when:  
    /// - When the result of an 8-bit addition is higher than $FF.
    /// - When the result of a 16-bit addition is higher than $FFFF.
    /// - When the result of a subtraction or comparison is lower than zero (like in Z80 and x86 CPUs, but unlike in 65XX and ARM CPUs).
    /// - When a rotate/shift operation shifts out a “1” bit.
    CF,

    /// Half-carry flag  
    /// These flags are used by the DAA instruction only.  
    /// Indicates carry for the lower 4 bits of the result.  
    HF,

    /// Subtraction flag  
    /// These flags are used by the DAA instruction only.  
    /// Indicates whether the previous instruction has been a subtraction.  
    NF,

    /// Zero flag  
    /// Is set if and only if the result of an operation is zero. Used by conditional jumps.  
    ZF,
}

impl Flag {
    /// Get corresponding bitmask
    pub fn bitmask(&self) -> u8 {
        match self {
            Flag::CF => 1 << 4,
            Flag::HF => 1 << 5,
            Flag::NF => 1 << 6,
            Flag::ZF => 1 << 7,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Registers {
    pub pc: u16,
    pub sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
}

impl Registers {
    pub fn read8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::F => self.f,
            Reg8::H => self.h,
            Reg8::L => self.l,
        }
    }

    pub fn write8(&mut self, reg: Reg8, byte: u8) {
        match reg {
            Reg8::A => self.a = byte,
            Reg8::B => self.b = byte,
            Reg8::C => self.c = byte,
            Reg8::D => self.d = byte,
            Reg8::E => self.e = byte,
            Reg8::F => self.f = byte,
            Reg8::H => self.h = byte,
            Reg8::L => self.l = byte,
        };
    }

    pub fn read16(&self, reg: Reg16) -> u16 {
        let [reg_lo, reg_hi] = Reg16::to_le_reg8(reg);
        let lo = self.read8(reg_lo);
        let hi = self.read8(reg_hi);

        u16::from_le_bytes([lo, hi])
    }

    pub fn write16(&mut self, reg: Reg16, word: u16) {
        let [lo, hi] = u16::to_le_bytes(word);
        let [reg_lo, reg_hi] = Reg16::to_le_reg8(reg);

        self.write8(reg_lo, lo);
        self.write8(reg_hi, hi);
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        self.f & flag.bitmask() != 0
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.f |= flag.bitmask();
        } else {
            self.f &= !flag.bitmask();
        }
    }
}
