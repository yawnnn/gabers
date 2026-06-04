pub struct Timer {
    raw_counter: u8, // DIV: Divide register
    counter: u8,     // TIMA: Timer counter
    reset: u8,       // TMA: Timer modulo
    control: u8,     // TAC: Timer control
}

impl Timer {
    pub fn new() -> Self {
        Self {
            raw_counter: 0x18,
            counter: 0,
            reset: 0,
            control: 0xF8,
        }
    }

    pub fn read8(&self, addr: usize) -> u8 {
        match addr {
            0xFF04 => self.raw_counter,
            0xFF05 => self.counter,
            0xFF06 => self.reset,
            0xFF07 => self.control,
            _ => unreachable!(),
        }
    }

    pub fn write8(&mut self, addr: usize, val: u8) {
        match addr {
            0xFF04 => self.raw_counter = 0,
            0xFF05 => self.counter = val,
            0xFF06 => self.reset = val,
            0xFF07 => self.control = val,
            _ => unreachable!(),
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        let raw_counter = self.raw_counter as u16 + cycles as u16;
        if self.control & 0b100 != 0 {
            let modulo: u16 = [256, 4, 16, 64][(self.control & 0b11) as usize] * 4;
            if raw_counter.is_multiple_of(modulo) {
                match self.counter.checked_add(1) {
                    Some(val) => self.counter = val,
                    None => self.counter = self.reset,
                }
            }
        }
        self.raw_counter = self.raw_counter.wrapping_add(cycles);
    }
}
