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