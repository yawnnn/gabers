#![allow(unused)]

use crate::common::*;
use crate::memory::*;
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
    SP,
}

#[derive(Debug, Clone, Copy)]
pub enum Flag {
    /// Used by conditional jumps and instructions such as ADC, SBC, RL, RLA, etc.
    /// Is set in these cases:
    /// - When the result of an 8-bit addition is higher than $FF.
    /// - When the result of a 16-bit addition is higher than $FFFF.
    /// - When the result of a subtraction or comparison is lower than zero (like in Z80 and x86 CPUs, but unlike in 65XX and ARM CPUs).
    /// - When a rotate/shift operation shifts out a “1” bit.
    Carry,

    /// These flags are used by the DAA instruction only.
    /// N indicates whether the previous instruction has been a subtraction, and H indicates carry for the lower 4 bits of the result.
    /// DAA also uses the C flag, which must indicate carry for the upper 4 bits.
    /// After adding/subtracting two BCD numbers, DAA is used to convert the result to BCD format.
    /// BCD numbers range from $00 to $99 rather than $00 to $FF.
    /// Because only two flags (C and H) exist to indicate carry-outs of BCD digits,
    /// DAA is ineffective for 16-bit operations (which have 4 digits), and use for INC/DEC operations (which do not affect C-flag) has limits.
    HalfCarry,
    Subtraction,

    /// Is set if and only if the result of an operation is zero. Used by conditional jumps.
    Zero,
}

impl From<Flag> for u8 {
    fn from(value: Flag) -> Self {
        match value {
            Flag::Carry => 1 << 4,
            Flag::HalfCarry => 1 << 5,
            Flag::Subtraction => 1 << 6,
            Flag::Zero => 1 << 7,
        }
    }
}

impl From<u8> for Flag {
    fn from(value: u8) -> Self {
        match value {
            b if b == 1 << 4 => Flag::Carry,
            b if b == 1 << 5 => Flag::HalfCarry,
            b if b == 1 << 6 => Flag::Subtraction,
            b if b == 1 << 7 => Flag::Zero,
            _ => panic!(),
        }
    }
}

pub struct FlagReg(u8);

impl FlagReg {
    pub fn get(&self, flag: Flag) -> bool {
        self.0 & u8::from(flag) != 0
    }

    pub fn set(&mut self, flag: Flag, value: bool) {
        if value {
            self.0 |= u8::from(flag);
        } else {
            self.0 &= !u8::from(flag);
        }
    }

    pub fn toggle(&mut self, flag: Flag) {
        self.0 ^= u8::from(flag)
    }

    fn inner(&self) -> u8 {
        self.0
    }

    fn set_inner(&mut self, value: u8) {
        self.0 = value;
    }
}

pub struct Registers {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagReg,
    pub h: u8,
    pub l: u8,
}

impl Registers {
    pub fn read8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::F => self.f.inner(),
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
            Reg8::F => self.f.set_inner(byte),
            Reg8::H => self.h = byte,
            Reg8::L => self.l = byte,
        };
    }

    pub fn read16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::AF => u16::from_le_bytes([self.f.inner(), self.a]),
            Reg16::BC => u16::from_le_bytes([self.c, self.b]),
            Reg16::DE => u16::from_le_bytes([self.e, self.d]),
            Reg16::HL => u16::from_le_bytes([self.l, self.h]),
            Reg16::SP => self.sp,
        }
    }

    pub fn write16(&mut self, reg: Reg16, word: u16) {
        let [low, high] = u16::to_le_bytes(word);

        match reg {
            Reg16::AF => {
                self.a = high;
                self.f.set_inner(low);
            }
            Reg16::BC => {
                self.b = high;
                self.c = low
            }
            Reg16::DE => {
                self.d = high;
                self.e = low
            }
            Reg16::HL => {
                self.h = high;
                self.l = low
            }
            Reg16::SP => self.sp = word,
        }
    }
}
