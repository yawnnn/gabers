#![allow(unused)]

use core::fmt;

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

struct _Flags {
    inner: u8,
}

impl _Flags {
    const CARRY: u8      = 1 << 4;
    const HALF_CARRY: u8 = 1 << 5;
    const SUB: u8        = 1 << 6;
    const ZERO: u8       = 1 << 7;
}

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
}

/// implement wide (16 bit) register by using two registers
macro_rules! impl_wide_reg {
    ($high:ident, $low:ident, $read:ident, $write:ident, $desc:expr) => {
        #[doc = concat!("Read ", $desc)]
        pub fn $read(&self) -> u16 {
            ((self.$high as u16) << 8) | (self.$low as u16)
        }

        #[doc = concat!("Read ", $desc)]
        pub fn $write(&mut self, dword: u16) {
            self.$high = (dword >> 8) as u8;
            self.$low = (dword & 0xFF) as u8;
        }
    };
}

// /// implement read and write to specific bit of the flag register
// macro_rules! impl_flag_reg {
//     ($bit:literal, $read:ident, $write:ident, $desc:expr) => {
//         #[doc = concat!("Read ", $desc)]
//         pub fn $read(&self) -> bool {
//             (self.f & (1 << $bit)) != 0
//         }

//         #[doc = concat!("Write ", $desc)]
//         pub fn $write(&mut self, fg: bool) {
//             self.f |= (fg as u8) << $bit;
//         }
//     };
// }

// impl_flag_reg!(4, read_f_cf, write_f_cf, "CF: Carry flag");
// impl_flag_reg!(5, read_f_hf, write_f_hf, "HF: Half-carry flag");
// impl_flag_reg!(6, read_f_sf, write_f_sf, "SF: Subtraction flag");
// impl_flag_reg!(7, read_f_zf, write_f_zf, "ZF: Zero flag");

impl Registers {
    impl_wide_reg!(a, f, read_af, write_af, "AF register");
    impl_wide_reg!(b, c, read_bc, write_bc, "BC register");
    impl_wide_reg!(d, e, read_de, write_de, "DE register");
    impl_wide_reg!(h, l, read_hl, write_hl, "HL register");

    pub fn get_flags(&self) -> Flags {
        Flags::from_byte(self.f)
    }

    pub fn set_flags(&mut self, flags: Flags) {
        self.f = flags.into_byte();
    }
}

pub struct Flags {
    pub carry: bool,
    pub half_carry: bool,
    pub subtraction: bool,
    pub zero: bool,
}

impl Flags {
    fn into_byte(self) -> u8 {
        ((self.carry as u8) << 4)
            | ((self.half_carry as u8) << 5)
            | ((self.subtraction as u8) << 6)
            | ((self.zero as u8) << 7)
    }

    fn from_byte(byte: u8) -> Self {
        Flags {
            carry: (byte & 4) != 0,
            half_carry: (byte & 5) != 0,
            subtraction: (byte & 6) != 0,
            zero: (byte & 7) != 0,
        }
    }
}
