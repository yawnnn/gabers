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
    /// Is set in these cases:  
    /// - When the result of an 8-bit addition is higher than $FF.
    /// - When the result of a 16-bit addition is higher than $FFFF.
    /// - When the result of a subtraction or comparison is lower than zero (like in Z80 and x86 CPUs, but unlike in 65XX and ARM CPUs).
    /// - When a rotate/shift operation shifts out a “1” bit.
    CF,

    /// Half-carry flag  
    /// These flags are used by the DAA instruction only.  
    /// indicates carry for the lower 4 bits of the result.  
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
    /// Get corresponding bit-mask
    pub fn get_mask(&self) -> u8 {
        match *self {
            Flag::CF => 1 << 4,
            Flag::HF => 1 << 5,
            Flag::NF => 1 << 6,
            Flag::ZF => 1 << 7,
        }
    }
}

// impl From<Flag> for u8 {
//     #[rustfmt::skip]
//     fn from(value: Flag) -> Self {
//         match value {
//             Flag::CF => 1 << 4,
//             Flag::HF => 1 << 5,
//             Flag::NF => 1 << 6,
//             Flag::ZF => 1 << 7,
//         }
//     }
// }

pub struct FlagsRegister(u8);

impl FlagsRegister {
    fn get(&self, flag: Flag) -> bool {
        self.0 & flag.get_mask() != 0
    }

    fn set(&mut self, flag: Flag, value: bool) {
        if value {
            self.0 |= flag.get_mask();
        } else {
            self.0 &= !flag.get_mask();
        }
    }

    fn toggle(&mut self, flag: Flag) {
        self.0 ^= flag.get_mask()
    }

    fn inner(&self) -> u8 {
        self.0
    }

    fn set_inner(&mut self, value: u8) {
        self.0 = value;
    }
}

/// Implement getter and setter for flag
macro_rules! flag_impl {
    ($flag:ident, $getter:ident, $setter:ident) => {
        pub fn $getter(&self) -> bool {
            self.f.get(Flag::$flag)
        }

        pub fn $setter(&mut self, fg: bool) {
            self.f.set(Flag::$flag, fg);
        }
    };
}

pub struct Registers {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: FlagsRegister,
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

    pub fn flag(&self, flag: Flag) -> bool {
        self.f.get(flag)
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        self.f.set(flag, value);
    }

    pub fn sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, word: u16) {
        self.sp = word;
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn set_pc(&mut self, word: u16) {
        self.pc = word;
    }
}
