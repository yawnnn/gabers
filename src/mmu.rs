use crate::{gameboy::Gameboy, gpu::Gpu};

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
/// Joypad input
macro_rules! joypad_addr {
    () => { 
        0xFF00
    };
}
/// Serial transfer
macro_rules! serial_range {
    () => { 
        0xFF01..=0xFF02
    };
}
/// Timer and divider
macro_rules! timer_range {
    () => { 
        0xFF04..=0xFF07
    };
}
/// Interrupts
macro_rules! inter_flag_addr {
    () => { 
        0xFF0F
    };
}
/// Audio
macro_rules! audio_range {
    () => { 
        0xFF10..=0xFF3F
    };
}
/// LCD Control, Status, Position and Scorlling
macro_rules! lcd_range {
    () => { 
        0xFF40..=0xFF45
    };
}
/// OAM DMA transfer
macro_rules! dma_addr {
    () => { 
        0xFF46
    };
}
/// Palettes
macro_rules! palette_range {
    () => { 
        0xFF47..=0xFF49
    };
}
/// Window position
macro_rules! window_range {
    () => { 
        0xFF4A..=0xFF4B
    };
}

impl Gameboy {
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
                joypad_addr!() => self.joypad.read8(),
                serial_range!() => todo!(),
                timer_range!() => self.timer.read8(addr),
                inter_flag_addr!() => *self.inter_flag,
                audio_range!() => todo!(),
                lcd_range!() => self.gpu.read8(addr),
                dma_addr!() => todo!(),
                palette_range!() => self.gpu.read8(addr),
                window_range!() => self.gpu.read8(addr),
                _ => todo!(),
            },
            hram_range!() => todo!(),
            ie_reg!() => *self.inter_enable,
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
                joypad_addr!() => self.joypad.write8(val),
                serial_range!() => todo!(),
                timer_range!() => self.timer.write8(addr, val),
                inter_flag_addr!() => *self.inter_flag = val,
                audio_range!() => todo!(),
                lcd_range!() => self.gpu.write8(addr, val),
                dma_addr!() => Gpu::dma_transfer(self, val),
                palette_range!() => self.gpu.write8(addr, val),
                window_range!() => self.gpu.write8(addr, val),
                _ => todo!(),
            },
            hram_range!() => todo!(),
            ie_reg!() => *self.inter_enable = val,
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
