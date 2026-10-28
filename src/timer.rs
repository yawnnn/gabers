use crate::{
    gameboy::*,
    interrupt::{InterruptFlags, Interrupts},
};

const DIV_CLOCK: usize = 16_384;
const SYSTEM_DIV_RATIO: usize = SYSTEM_CLOCK / DIV_CLOCK;

pub struct Timer {
    div: u16,       // DIV: Divide register
    tima: u8,       // TIMA: Timer counter
    tma: u8,        // TMA: Timer modulo
    tac: u8,        // TAC: Timer control
    line_low: bool, // current status
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0xAB,
            tima: 0,
            tma: 0,
            tac: 0xF8,
            line_low: false,
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.div / SYSTEM_DIV_RATIO as u16) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => unreachable!(),
        }
    }

    pub fn write8(&mut self, inter: &mut Interrupts, addr: u16, val: u8) {
        match addr {
            0xFF04 => {
                self.div = 0;
                self.check_inter(inter);
            }
            0xFF05 => self.tima = val,
            0xFF06 => self.tma = val,
            0xFF07 => {
                self.tac = val;
                self.check_inter(inter);
            }
            _ => unreachable!(),
        }
    }

    fn enabled(&self) -> bool {
        self.tac & 0b100 != 0
    }

    fn bitmask(&self) -> u16 {
        match self.tac & 0b11 {
            0b00 => 1 << 9,
            0b01 => 1 << 3,
            0b10 => 1 << 5,
            0b11 => 1 << 7,
            _ => unreachable!(),
        }
    }

    fn check_inter(&mut self, inter: &mut Interrupts) {
        // instead of DIV % cycles == 0, i'm actually monitoring the bit before the target cycles
        // eg. TAC=01 => 16 cycles (bit 4) => checking falling edge of 8 (bit 3)
        // this accounts for "bugs" like zeroing DIV when line was high, causing spurious increments
        let line_low = self.enabled() && self.div & self.bitmask() == 0;
        if !self.line_low && line_low {
            match self.tima.checked_add(1) {
                Some(val) => self.tima = val,
                None => {
                    self.tima = self.tma;
                    inter.raise(InterruptFlags::TIMER);
                }
            }
        }
        self.line_low = line_low;
    }

    pub fn tick(&mut self, inter: &mut Interrupts, cycles: TCycle) {
        for _ in 0..cycles {
            self.div = self.div.wrapping_add(1);
            self.check_inter(inter);
        }
    }
}
