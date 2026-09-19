use crate::interrupt::{Interrupts, InterruptFlags};

#[derive(Clone)]
#[rustfmt::skip]
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
        Joypad {
            state: 0xFF,
            select: 0x00,
        }
    }

    pub fn press(&mut self, inter: &mut Interrupts, key: JoypadKey) {
        self.state &= !(key as u8);
        inter.raise(InterruptFlags::JOYPAD);
    }

    pub fn release(&mut self, key: JoypadKey) {
        self.state |= key as u8;
    }

    fn use_dpad(&self) -> bool {
        self.select & 0x10 == 0
    }

    fn use_buttons(&self) -> bool {
        self.select & 0x20 == 0
    }

    pub fn read8(&self) -> u8 {
        let mut res = 0x0F;
        if self.use_dpad() {
            res &= self.state & 0x0F;
        }
        if self.use_buttons() {
            res &= self.state >> 4;
        }
        0x0C | (self.select & 0x30) | res
    }

    pub fn write8(&mut self, val: u8) {
        self.select = val & 0xF0;
    }
}
