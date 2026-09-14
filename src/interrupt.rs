use std::ops;

pub struct Interrupt(u8);

#[rustfmt::skip]
impl Interrupt {
    pub const VBLANK:  u8 = 1 << 0;
    pub const LCD:     u8 = 1 << 1;
    pub const TIMER:   u8 = 1 << 2;
    pub const SERIAL:  u8 = 1 << 3;
    pub const JOYPAD:  u8 = 1 << 4;
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

impl Interrupt {
    pub fn raise(&mut self, flag: u8) {
        self.0 |= flag;
    }
}
