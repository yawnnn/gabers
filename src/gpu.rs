#![allow(unused)]

use crate::common::*;

struct Tile {
    pixels: [[u8; 8]; 8],
}

pub struct GPU {
    tile_set: [Tile; TILE_COUNT],
    video_ram: [u8; VRAM_SIZE],
    canvas_buffer: [u8; PIXEL_COUNT],
}

impl GPU {
    pub fn read_byte(&self, addr: u16) -> u8 {
        self.video_ram[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) {
        self.video_ram[addr as usize] = byte;
    }
}
