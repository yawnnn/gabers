#![allow(unused)]
use std::ops::Range;

use crate::gpu::Gpu;

/*
 * Memory map
 */
pub struct MemoryMap;
pub type MM = MemoryMap;

impl MM {
    pub const ROM_0: Range<usize> = 0x0000..0x3FFF + 1; // 16 KiB ROM bank 00 - From cartridge, usually a fixed bank
    pub const ROM_N: Range<usize> = 0x4000..0x7FFF + 1; // 16 KiB ROM Bank 01–NN - From cartridge, switchable bank via mapper (if any)
    pub const VRAM: Range<usize> = 0x8000..0x9FFF + 1; // 8 KiB Video RAM (VRAM) - In CGB mode, switchable bank 0/1
    pub const ERAM: Range<usize> = 0xA000..0xBFFF + 1; // 8 KiB External RAM - From cartridge, switchable bank if any
    pub const WRAM: Range<usize> = 0xC000..0xCFFF + 1; // 4 KiB Work RAM (WRAM)
    pub const WRAM_CGB: Range<usize> = 0xD000..0xDFFF + 1; // 4 KiB Work RAM (WRAM) - In CGB mode, switchable bank 1–7
    pub const ECHO_RAM: Range<usize> = 0xE000..0xFDFF + 1; // Echo RAM (mirror of C000–DDFF) - Nintendo says use of this area is prohibited.
    pub const OAM: Range<usize> = 0xFE00..0xFE9F + 1; // Object attribute memory (OAM)
    pub const UNUSABLE: Range<usize> = 0xFEA0..0xFEFF + 1; // Not Usable - Nintendo says use of this area is prohibited.
    pub const IO_REGS: Range<usize> = 0xFF00..0xFF7F + 1; // I/O Registers
    pub const HRAM: Range<usize> = 0xFF80..0xFFFE + 1; // High RAM (HRAM)
    pub const IE_REGS: Range<usize> = 0xFFFF..0xFFFF + 1; // Interrupt Enable register (IE)
}

/*
 * I/O 
 */

pub struct IORegs;
pub type IO = IORegs;

impl IO {
    pub const JOYPAD: Range<usize> = 0xFF00..0xFF00 + 1; // Joypad input
    pub const SERIAL: Range<usize> = 0xFF01..0xFF02 + 1; // Serial transfer
    pub const TIMER_DIV: Range<usize> = 0xFF04..0xFF07 + 1; // Timer and divider
    pub const INTER: Range<usize> = 0xFF0F..0xFF0F + 1; // Interrupts
    pub const AUDIO: Range<usize> = 0xFF10..0xFF26 + 1; // Audio
    pub const WAVE_PATT: Range<usize> = 0xFF30..0xFF3F + 1; // Wave pattern
    pub const LCD_CTRL: Range<usize> = 0xFF40..0xFF4B + 1; // LCD Control, Status, Position, Scrolling, and Palettes
    pub const VRAM_BLANK: Range<usize> = 0xFF4F..0xFF4F + 1; // VRAM Bank Select
    pub const NONZERO: Range<usize> = 0xFF50..0xFF50 + 1; // Set to non-zero to disable boot ROM
    pub const VRAM_DMA: Range<usize> = 0xFF51..0xFF55 + 1; // VRAM DMA
    pub const BG_OBJ: Range<usize> = 0xFF68..0xFF6B + 1; // BG / OBJ Palettes
    pub const WRAM_BLANK: Range<usize> = 0xFF70..0xFF70 + 1; // WRAM Bank Select
}

/*
 * VRAM
 */
#[allow(clippy::upper_case_acronyms)]
pub struct VRAM;

impl VRAM {
    pub const TILE_BLOCKS: Range<usize> = 0x0000..0x1800;
    pub const TILE_MAPS: Range<usize> = 0x1800..0x1FFF;

    pub const TBLOCK0: Range<usize> = 0x0000..0x0800;
    pub const TBLOCK1: Range<usize> = 0x0800..0x1000;
    pub const TBLOCK2: Range<usize> = 0x1000..0x1800;
    pub const TMAP0: Range<usize> = 0x1800..0x1C00;
    pub const TMAP1: Range<usize> = 0x1C00..0x1FFF;
}

/*
 * Jump vectors
 *
 * RST instructions: 0x0000, 0x0008, 0x0010, 0x0018, 0x0020, 0x0028, 0x0030, 0x0038
 * Interrupts: 0x0040, 0x0048, 0x0050, 0x0058, 0x0060
 */
pub const JUMP_VECTORS: Range<usize> = 0x0000..0x00FF;

/*
 * Cartridge header
 */
//pub const BOOT_ROM: Range<usize> = 0x0000..0x00FF;
//pub const INTERRUPT_TABLE: Range<usize> = 0x0000..0x00FF;
pub const CARTRIDGE_HEADER: Range<usize> = 0x0100..0x014F;
//pub const CARTRIDGE_BODY: Range<usize> = 0x014F..0x3FFF;

/*
 * Hardware registers
 */
pub struct HardwareRegisters;
pub type HWReg = HardwareRegisters;

impl HWReg {
    pub const JOYP: usize = 0xFF00; // Joypad - Mixed
    pub const SB: usize = 0xFF01; // Serial transfer data - R/W
    pub const SC: usize = 0xFF02; // Serial transfer control - R/W
    pub const DIV: usize = 0xFF04; // Divider register - R/W
    pub const TIMA: usize = 0xFF05; // Timer counter - R/W
    pub const TMA: usize = 0xFF06; // Timer modulo - R/W
    pub const TAC: usize = 0xFF07; // Timer control - R/W
    pub const IF: usize = 0xFF0F; // Interrupt flag - R/W
    pub const NR10: usize = 0xFF10; // Sound channel 1 sweep - R/W
    pub const NR11: usize = 0xFF11; // Sound channel 1 length timer & duty cycle - Mixed
    pub const NR12: usize = 0xFF12; // Sound channel 1 volume & envelope - R/W
    pub const NR13: usize = 0xFF13; // Sound channel 1 period low - W
    pub const NR14: usize = 0xFF14; // Sound channel 1 period high & control - Mixed
    pub const NR21: usize = 0xFF16; // Sound channel 2 length timer & duty cycle - Mixed
    pub const NR22: usize = 0xFF17; // Sound channel 2 volume & envelope - R/W
    pub const NR23: usize = 0xFF18; // Sound channel 2 period low - W
    pub const NR24: usize = 0xFF19; // Sound channel 2 period high & control - Mixed
    pub const NR30: usize = 0xFF1A; // Sound channel 3 DAC enable - R/W
    pub const NR31: usize = 0xFF1B; // Sound channel 3 length timer - W
    pub const NR32: usize = 0xFF1C; // Sound channel 3 output level - R/W
    pub const NR33: usize = 0xFF1D; // Sound channel 3 period low - W
    pub const NR34: usize = 0xFF1E; // Sound channel 3 period high & control - Mixed
    pub const NR41: usize = 0xFF20; // Sound channel 4 length timer - W
    pub const NR42: usize = 0xFF21; // Sound channel 4 volume & envelope - R/W
    pub const NR43: usize = 0xFF22; // Sound channel 4 frequency & randomness - R/W
    pub const NR44: usize = 0xFF23; // Sound channel 4 control - Mixed
    pub const NR50: usize = 0xFF24; // Master volume & VIN panning - R/W
    pub const NR51: usize = 0xFF25; // Sound panning - R/W
    pub const NR52: usize = 0xFF26; // Sound on/off - Mixed
    pub const WAVE_RAM: Range<usize> = 0xFF30..0xFF3F + 1; // Storage for one of the sound channels’ waveform - R/W
    pub const LCDC: usize = 0xFF40; // LCD control - R/W
    pub const STAT: usize = 0xFF41; // LCD status - Mixed
    pub const SCY: usize = 0xFF42; // Viewport Y position - R/W
    pub const SCX: usize = 0xFF43; // Viewport X position - R/W
    pub const LY: usize = 0xFF44; // LCD Y coordinate - R
    pub const LYC: usize = 0xFF45; // LY compare - R/W
    pub const DMA: usize = 0xFF46; // OAM DMA source address & start - R/W
    pub const BGP: usize = 0xFF47; // BG palette data - R/W
    pub const OBP0: usize = 0xFF48; // OBJ palette 0 data - R/W
    pub const OBP1: usize = 0xFF49; // OBJ palette 1 data - R/W
    pub const WY: usize = 0xFF4A; // Window Y position - R/W
    pub const WX: usize = 0xFF4B; // Window X position plus 7 - R/W
    pub const KEY1: usize = 0xFF4D; // Prepare speed switch - Mixed
    pub const VBK: usize = 0xFF4F; // VRAM bank - R/W
    pub const HDMA1: usize = 0xFF51; // VRAM DMA source high - W
    pub const HDMA2: usize = 0xFF52; // VRAM DMA source low - W
    pub const HDMA3: usize = 0xFF53; // VRAM DMA destination high - W
    pub const HDMA4: usize = 0xFF54; // VRAM DMA destination low - W
    pub const HDMA5: usize = 0xFF55; // VRAM DMA length/mode/start - R/W
    pub const RP: usize = 0xFF56; // Infrared communications port - Mixed
    pub const BCPS_BGPI: usize = 0xFF68; // Background color palette specification / Background palette index - R/W
    pub const BCPD_BGPD: usize = 0xFF69; // Background color palette data / Background palette data - R/W
    pub const OCPS_OBPI: usize = 0xFF6A; // OBJ color palette specification / OBJ palette index - R/W
    pub const OCPD_OBPD: usize = 0xFF6B; // OBJ color palette data / OBJ palette data - R/W
    pub const OPRI: usize = 0xFF6C; // Object priority mode - R/W
    pub const SVBK: usize = 0xFF70; // WRAM bank - R/W
    pub const PCM12: usize = 0xFF76; // Audio digital outputs 1 & 2 - R
    pub const PCM34: usize = 0xFF77; // Audio digital outputs 3 & 4 - R
    pub const IE: usize = 0xFFFF; // Interrupt enable - R/W
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryBus {
    pub ram: [u8; MemoryBus::RAM_SIZE],
    pub gpu: Gpu,
}

impl MemoryBus {
    const RAM_SIZE: usize = u16::MAX as usize;

    pub fn read8(&self, addr: u16) -> u8 {
        if MM::VRAM.contains(&(addr as usize)) {
            self.gpu.read(addr)
        } else {
            self.ram[addr as usize]
        }
    }

    pub fn write8(&mut self, addr: u16, value: u8) {
        if MM::VRAM.contains(&(addr as usize)) {
            self.gpu.write(addr, value);
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
