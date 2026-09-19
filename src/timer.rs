use crate::{
    gameboy::*,
    interrupt::{Interrupts, InterruptFlags},
};

pub struct Timer {
    ticks: u16,     // DIV: Divide register
    counter: u8,    // TIMA: Timer counter
    modulo: u8,     // TMA: Timer modulo
    control: u8,    // TAC: Timer control
    line_low: bool, // current status
}

impl Timer {
    pub fn new() -> Self {
        Self {
            ticks: 0x18,
            counter: 0,
            modulo: 0,
            control: 0xF8,
            line_low: false,
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.ticks >> 8) as u8,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            0xFF07 => self.control,
            _ => unreachable!(),
        }
    }

    pub fn write8(&mut self, inter: &mut Interrupts, addr: u16, val: u8) {
        match addr {
            0xFF04 => {
                self.ticks = 0;
                self.check_inter(inter);
            }
            0xFF05 => self.counter = val,
            0xFF06 => self.modulo = val,
            0xFF07 => {
                self.control = val;
                self.check_inter(inter);
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

    fn check_inter(&mut self, inter: &mut Interrupts) {
        let line_low = self.enabled() && self.ticks & self.freq() == 0;
        if !self.line_low && line_low {
            match self.counter.checked_add(1) {
                Some(val) => self.counter = val,
                None => {
                    self.counter = self.modulo;
                    inter.raise(InterruptFlags::TIMER);
                }
            }
        }
        self.line_low = line_low;
    }

    pub fn tick(&mut self, inter: &mut Interrupts, cycles: u32) {
        for _ in 0..cycles {
            self.ticks = self.ticks.wrapping_add(1);
            self.check_inter(inter);
        }
    }
}
