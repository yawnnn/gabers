use crate::{gameboy::Gameboy, interrupt::Interrupt};

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
    pub gb: *mut Gameboy,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            state: 0xFF,
            select: 0x00,
            gb: std::ptr::null_mut(),
        }
    }

    fn gb(&mut self) -> &mut Gameboy {
        // SAFETY: this is used to access data inside gb that's not already "in scope" (eg. cpu.gb().timer), so aliasing *shouldn't* be an issue
        // TODO: this is still pretty unsafe and should be removed
        unsafe { self.gb.as_mut().unwrap() }
    }

    pub fn press(&mut self, key: JoypadKey) {
        self.state &= !(key as u8);
        self.gb().inter_flag.raise(Interrupt::JOYPAD);
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
