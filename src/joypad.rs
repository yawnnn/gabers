#[rustfmt::skip]
#[derive(Clone)]
pub enum JoypadKey {
    Right  = 1 << 0,
    Left   = 1 << 1,
    Up     = 1 << 2,
    Down   = 1 << 3,
    A      = 1 << 4,
    B      = 1 << 5,
    Select = 1 << 6,
    Start  = 1 << 7,
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