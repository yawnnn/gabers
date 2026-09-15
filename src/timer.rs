use crate::{gameboy::*, interrupt::Interrupt};

pub struct Timer {
    raw_counter: u16, // DIV: Divide register
    counter: u8,      // TIMA: Timer counter
    reset: u8,        // TMA: Timer modulo
    control: u8,      // TAC: Timer control
    line_low: bool,   // current status
    pub gb: *mut Gameboy,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            raw_counter: 0x18,
            counter: 0,
            reset: 0,
            control: 0xF8,
            line_low: false,
            gb: std::ptr::null_mut(),
        }
    }

    fn gb(&mut self) -> &mut Gameboy {
        // SAFETY: this is used to access data inside gb that's not already "in scope" (eg. cpu.gb().timer), so aliasing *shouldn't* be an issue
        // TODO: this is still pretty unsafe and should be removed
        unsafe { self.gb.as_mut().unwrap() }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.raw_counter >> 8) as u8,
            0xFF05 => self.counter,
            0xFF06 => self.reset,
            0xFF07 => self.control,
            _ => unreachable!(),
        }
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => {
                self.raw_counter = 0;
                self.check_inter();
            }
            0xFF05 => self.counter = val,
            0xFF06 => self.reset = val,
            0xFF07 => {
                self.control = val;
                self.check_inter();
            }
            _ => unreachable!(),
        }
    }

    fn enabled(&self) -> bool {
        self.control & 0b100 != 0
    }

    fn freq(&self) -> u16 {
        [256, 4, 16, 64][(self.control & 0b11) as usize] * MASTER_SYSTEM_CLOCK_RATIO as u16
    }

    fn check_inter(&mut self) {
        let line_low = self.enabled() && self.raw_counter & self.freq() == 0;
        if !self.line_low && line_low {
            match self.counter.checked_add(1) {
                Some(val) => self.counter = val,
                None => {
                    self.counter = self.reset;
                    self.gb().inter_flag.raise(Interrupt::TIMER);
                }
            }
        }
        self.line_low = line_low;
    }

    pub fn tick(&mut self, cycles: u32) {
        for _ in 0..cycles {
            self.raw_counter = self.raw_counter.wrapping_add(1);
            self.check_inter();
        }
    }
}
