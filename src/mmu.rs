use crate::{common::Span, gameboy::Gameboy};

/// 16 KiB ROM bank 00 (From cartridge, usually a fixed bank) + 16 KiB ROM Bank 01–NN (From cartridge, switchable bank via mapper (if any))
macro_rules! rom_range {
    () => {
        0x0000..=0x7FFF
    };
}
/// 8 KiB Video RAM (VRAM) - In CGB mode, switchable bank 0/1 - blocks
macro_rules! tiles_range {
    () => {
        0x8000..=0x97FF
    };
}
pub(crate) use tiles_range;
/// 8 KiB Video RAM (VRAM) - In CGB mode, switchable bank 0/1 - maps
macro_rules! tilemaps_range {
    () => {
        0x9800..=0x9FFF
    };
}
pub(crate) use tilemaps_range;
/// 8 KiB External RAM - From cartridge, switchable bank if any
macro_rules! eram_range {
    () => {
        0xA000..=0xBFFF
    };
}
/// 4 KiB Work RAM (WRAM)
macro_rules! wram_range {
    () => {
        0xC000..=0xCFFF
    };
}
/// 4 KiB Work RAM (WRAM) - In CGB mode, switchable bank 1–7
macro_rules! wram_cgb_range {
    () => {
        0xD000..=0xDFFF
    };
}
/// Echo RAM (mirror of C000–DDFF) - Nintendo says use of this area is prohibited.
macro_rules! echo_ram_range {
    () => {
        0xE000..=0xFDFF
    };
}
/// Object attribute memory (OAM)
macro_rules! oam_range {
    () => {
        0xFE00..=0xFE9F
    };
}
pub(crate) use oam_range;
/// Not Usable - Nintendo says use of this area is prohibited.
macro_rules! unusable_range {
    () => {
        0xFEA0..=0xFEFF
    };
}
/// I/O Registers
macro_rules! io_regs_range {
    () => {
        0xFF00..=0xFF7F
    };
}
/// High RAM (HRAM)
macro_rules! hram_range {
    () => {
        0xFF80..=0xFFFE
    };
}
/// Interrupt Enable register (IE)
macro_rules! ie_reg {
    () => {
        0xFFFF
    };
}

impl Gameboy {
    pub fn dma_transfer(&mut self, val: u8) {
        let src_base = (val as u16) << 8;
        let dst_base = *oam_range!().start() as u16;
        let nbytes = oam_range!().span() as u16;
        for i in 0..nbytes {
            let byte = self.read8(src_base + i);
            self.write8(dst_base + i, byte);
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            rom_range!() => self.cartridge.read(addr),
            tiles_range!() => self.gpu.read8(addr),
            tilemaps_range!() => self.gpu.read8(addr),
            eram_range!() => self.cartridge.read(addr),
            wram_range!() => todo!(),
            wram_cgb_range!() => todo!(),
            echo_ram_range!() => self.read8(addr - 0x2000),
            oam_range!() => self.gpu.read8(addr),
            unusable_range!() => 0xFF,
            io_regs_range!() => match addr {
                0xFF00 => self.joypad.read8(),             // HWReg::P1_JOYP
                0xFF01..=0xFF02 => todo!(),                // HWReg::SB, HWReg::SC
                0xFF04..=0xFF07 => self.timer.read8(addr), // HWReg::TIMA, HWReg::TMA, HWReg::TAC
                0xFF0F => *self.inter_flag,                // HWReg::IF
                0xFF10..=0xFF3F => todo!(), // HWReg::NR10..HWReg::NR52, HWReg::WAVE_RAM
                0xFF40..=0xFF45 => self.gpu.read8(addr), // HWReg::LCDC, HWReg::STAT, HWReg::SCY, HWReg::SCX, HWReg::LY, HWReg::LYC
                0xFF46 => todo!(),                       // HWReg::DMA
                0xFF47..=0xFF4B => todo!(), // HWReg::BGP, HWReg::OBP0, HWReg::OBP1, HWReg::WY, HWReg::WX
                _ => todo!(),
            },
            hram_range!() => todo!(),
            ie_reg!() => *self.inter_enable, // HWReg::IE
        }
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            rom_range!() => self.cartridge.write(addr, val),
            tiles_range!() => self.gpu.write8(addr, val),
            tilemaps_range!() => self.gpu.write8(addr, val),
            eram_range!() => self.cartridge.write(addr, val),
            wram_range!() => todo!(),
            wram_cgb_range!() => todo!(),
            echo_ram_range!() => self.write8(addr - 0x2000, val),
            oam_range!() => self.gpu.write8(addr, val),
            unusable_range!() => (),
            io_regs_range!() => match addr {
                0xFF00 => self.joypad.write8(val),               // HWReg::P1_JOYP
                0xFF01..=0xFF02 => todo!(),                      // HWReg::SB, HWReg::SC
                0xFF04..=0xFF07 => self.timer.write8(addr, val), // HWReg::TIMA, HWReg::TMA, HWReg::TAC
                0xFF0F => *self.inter_flag = val,                // HWReg::IF
                0xFF40..=0xFF45 => self.gpu.write8(addr, val), // HWReg::LCDC, HWReg::STAT, HWReg::SCY, HWReg::SCX, HWReg::LY, HWReg::LYC
                0xFF10..=0xFF3F => todo!(), // HWReg::NR10..HWReg::NR52, HWReg::WAVE_RAM
                0xFF46 => self.dma_transfer(val), // HWReg::DMA
                0xFF47..=0xFF4B => todo!(), // HWReg::BGP, HWReg::OBP0, HWReg::OBP1, HWReg::WY, HWReg::WX
                _ => todo!(),
            },
            hram_range!() => todo!(),
            ie_reg!() => *self.inter_enable = val, // HWReg::IE
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
