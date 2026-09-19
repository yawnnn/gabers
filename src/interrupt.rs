use std::ops::{Deref, DerefMut};

pub struct InterruptFlags(u8);

pub struct Interrupts {
    pub enable: u8,
    pub flags: InterruptFlags,
}

impl Interrupts {
    pub fn new() -> Self {
        Interrupts {
            enable: 0,
            flags: InterruptFlags(0),
        }
    }

    pub fn enabled(&self) -> u8 {
        self.enable & self.flags.0 & InterruptFlags::BITMASK
    }

    pub fn raise(&mut self, flag: u8) {
        self.flags.0 |= flag;
    }

    pub fn lower(&mut self, flag: u8) {
        self.flags.0 &= !flag;
    }
}

#[rustfmt::skip]
impl InterruptFlags {
    pub const VBLANK:  u8 = 1 << 0;
    pub const LCD:     u8 = 1 << 1;
    pub const TIMER:   u8 = 1 << 2;
    pub const SERIAL:  u8 = 1 << 3;
    pub const JOYPAD:  u8 = 1 << 4;
    pub const BITMASK: u8 = Self::VBLANK | Self::LCD | Self::TIMER | Self::SERIAL | Self::JOYPAD;
}

impl Deref for InterruptFlags {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InterruptFlags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
