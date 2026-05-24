#![allow(unused)]
use std::ops::Deref;
use std::ops::DerefMut;
use std::path;

use crate::cartridge::Cartridge;
use crate::constants::*;
use crate::gpu::*;

pub struct Interrupt(u8);

impl Interrupt {
    pub const VBLANK: u8 = 0x01;
    pub const LCD: u8 = 0x02;
    pub const TIMER: u8 = 0x04;
    pub const SERIAL: u8 = 0x08;
    pub const JOYPAD: u8 = 0x10;
    pub const BITMASK: u8 = Self::VBLANK | Self::LCD | Self::TIMER | Self::SERIAL | Self::JOYPAD; // 0x1F or 0b0001_1111

    fn new() -> Self {
        Interrupt(0)
    }
}

impl Deref for Interrupt {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Interrupt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[rustfmt::skip]
#[derive(Clone)]
pub enum JoypadKey {
    Right  = 0b0000_0001,
    Left   = 0b0000_0010,
    Up     = 0b0000_0100,
    Down   = 0b0000_1000,
    A      = 0b0001_0000,
    B      = 0b0010_0000,
    Select = 0b0100_0000,
    Start  = 0b1000_0000,
}

pub struct Joypad {
    state: u8,
    select: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad { state: 0xFF, select: 0x00 }
    }

    pub fn press(&mut self, key: JoypadKey) {
        self.state &= !(key as u8);
    }

    pub fn release(&mut self, key: JoypadKey) {
        self.state |= key as u8;
    }

    pub const SELECT_BUTTONS_BIT: u8 = 0x20;
    pub const SELECT_DPAD_BIT: u8 = 0x10;

    pub fn read8(&self) -> u8 {
        if self.select & Self::SELECT_BUTTONS_BIT == 0 {
            return 0xC0 | self.state & 0x0F;
        }
        if self.select & Self::SELECT_DPAD_BIT == 0 {
            return 0xC0 | (self.state >> 4) & 0x0F;
        }
        0xFF
    }

    pub fn write8(&mut self, val: u8) {
        self.select = val;
    }
}

pub struct Mmu {
    pub cartridge: Cartridge,
    pub gpu: Box<Gpu>,
    pub joypad: Joypad,
    pub inter_enable: Interrupt,
    pub inter_flag: Interrupt,
}

/// 16 KiB ROM bank 00 (From cartridge, usually a fixed bank) + 16 KiB ROM Bank 01–NN (From cartridge, switchable bank via mapper (if any))
macro_rules! rom_range {
    () => {
        0x0000..=0x7FFF
    };
}
/// 8 KiB Video RAM (VRAM) - In CGB mode, switchable bank 0/1
macro_rules! vram_range {
    () => {
        0x8000..=0x9FFF
    };
}
pub(crate) use vram_range;
/// 8 KiB External RAM - From cartridge, switchable bank if any
macro_rules! eram_range {
    () => {
        0xA000..=0xBFFF
    };
}
/// 4 KiB Work RAM (WRAM)
macro_rules! wram_range {
    () => {
        0xC000..=0xCFFF
    };
}
/// 4 KiB Work RAM (WRAM) - In CGB mode, switchable bank 1–7
macro_rules! wram_cgb_range {
    () => {
        0xD000..=0xDFFF
    };
}
/// Echo RAM (mirror of C000–DDFF) - Nintendo says use of this area is prohibited.
macro_rules! echo_ram_range {
    () => {
        0xE000..=0xFDFF
    };
}
/// Object attribute memory (OAM)
macro_rules! oam_range {
    () => {
        0xFE00..=0xFE9F
    };
}
/// Not Usable - Nintendo says use of this area is prohibited.
macro_rules! unusable_range {
    () => {
        0xFEA0..=0xFEFF
    };
}
/// I/O Registers
macro_rules! io_regs_range {
    () => {
        0xFF00..=0xFF7F
    };
}
/// High RAM (HRAM)
macro_rules! hram_range {
    () => {
        0xFF80..=0xFFFE
    };
}
/// Interrupt Enable register (IE)
macro_rules! ie_reg {
    () => {
        0xFFFF
    };
}

impl Mmu {
    const RAM_SIZE: usize = u16::MAX as usize;

    pub fn new(path: &path::Path) -> Self {
        let cartridge = Cartridge::new(path);
        let gpu = Box::<Gpu>::default();

        Mmu {
            cartridge,
            gpu,
            joypad: Joypad::new(),
            inter_enable: Interrupt(0),
            inter_flag: Interrupt(0),
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            rom_range!() => self.cartridge.read(addr as usize),
            vram_range!() => self.gpu.read(addr as usize),
            eram_range!() => self.cartridge.read(addr as usize),
            wram_range!() => todo!(),
            wram_cgb_range!() => todo!(),
            echo_ram_range!() => self.read8(addr - 0x2000),
            oam_range!() => self.gpu.read(addr as usize),
            unusable_range!() => 0xFF,
            io_regs_range!() => match addr {
                0xFF00 => self.joypad.read8(), // HWReg::P1_JOYP
                0xFF0F => *self.inter_flag,    // HWReg::IF
                _ => todo!(),
            },
            hram_range!() => todo!(),
            ie_reg!() => *self.inter_enable, // HWReg::IE
        }
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            rom_range!() => self.cartridge.write(addr as usize, val),
            vram_range!() => self.gpu.write(addr as usize, val),
            eram_range!() => self.cartridge.write(addr as usize, val),
            wram_range!() => todo!(),
            wram_cgb_range!() => todo!(),
            echo_ram_range!() => self.write8(addr - 0x2000, val),
            oam_range!() => self.gpu.write(addr as usize, val),
            unusable_range!() => (),
            io_regs_range!() => match addr {
                0xFF00 => self.joypad.write8(val), // HWReg::P1_JOYP
                0xFF0F => *self.inter_flag = val,  // HWReg::IF
                _ => todo!(),
            },
            hram_range!() => todo!(),
            ie_reg!() => *self.inter_enable = val, // HWReg::IE
        }
    }

    pub fn read16(&self, addr: u16) -> u16 {
        let lo = self.read8(addr);
        let hi = self.read8(addr.checked_add(1).unwrap()); // TODO: checked or wrapping?

        u16::from_le_bytes([lo, hi])
    }

    pub fn write16(&mut self, addr: u16, value: u16) {
        let [lo, hi] = u16::to_le_bytes(value);
        self.write8(addr, lo);
        self.write8(addr.checked_add(1).unwrap(), hi); // TODO: checked or wrapping?
    }
}
