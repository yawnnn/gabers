#![allow(unused)]
use crate::gpu::*;
use crate::constants::*;


#[derive(Clone, Copy, Debug)]
pub struct MemoryBus {
    pub ram: [u8; MemoryBus::RAM_SIZE],
    pub gpu: Gpu,
}

impl MemoryBus {
    const RAM_SIZE: usize = u16::MAX as usize;

    pub fn read8(&self, addr: u16) -> u8 {
        if MM::VRAM.contains(&(addr as usize)) {
            self.gpu.read(addr as usize - MM::VRAM.start)
        } else {
            self.ram[addr as usize]
        }
    }

    pub fn write8(&mut self, addr: u16, value: u8) {
        if MM::VRAM.contains(&(addr as usize)) {
            self.gpu.write(addr as usize - MM::VRAM.start, value);
        } else {
            self.ram[addr as usize] = value;
        }
    }

    pub fn read16(&self, addr: u16) -> u16 {
        let lo = self.read8(addr);
        let hi = self.read8(addr.checked_add(1).unwrap()); // TODO: checked or wrapping?

        u16::from_le_bytes([lo, hi])
    }

    pub fn write16(&mut self, addr: u16, value: u16) {
        let [lo, hi] = u16::to_le_bytes(value);
        self.write8(addr, lo);
        self.write8(addr.checked_add(1).unwrap(), hi); // TODO: checked or wrapping?
    }
}

impl Default for MemoryBus {
    fn default() -> Self {
        MemoryBus {
            ram: [0; MemoryBus::RAM_SIZE],
            gpu: Default::default(),
        }
    }
}
