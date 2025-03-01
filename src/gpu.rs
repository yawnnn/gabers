#![allow(unused)]

use std::ops::RangeBounds;

use crate::common::*;
use crate::memory::*;

struct Tile {
    pixels: [[u8; 8]; 8],
}

pub struct Gpu {
    tile_set: [Tile; span(VRAM_TILES)],
    video_ram: [u8; span(MM_VRAM)],
    canvas_buffer: [u8; 1],
}

impl Gpu {
    pub fn read8(&self, addr: u16) -> u8 {
        self.video_ram[addr as usize]
    }

    pub fn write8(&mut self, addr: u16, byte: u8) {
        self.video_ram[addr as usize] = byte;
    }
}
