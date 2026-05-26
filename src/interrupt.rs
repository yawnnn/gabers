use std::ops;

pub struct Interrupt(u8);

impl Interrupt {
    pub const VBLANK: u8 = 0x01;
    pub const LCD: u8 = 0x02;
    pub const TIMER: u8 = 0x04;
    pub const SERIAL: u8 = 0x08;
    pub const JOYPAD: u8 = 0x10;
    pub const BITMASK: u8 = Self::VBLANK | Self::LCD | Self::TIMER | Self::SERIAL | Self::JOYPAD; // 0x1F or 0b0001_1111

    pub fn new() -> Self {
        Interrupt(0)
    }
}

impl ops::Deref for Interrupt {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Interrupt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}