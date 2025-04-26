#![allow(unused)]

use std::ops::RangeBounds;

use crate::common::*;
use crate::memory::*;

#[derive(Default, Clone, Copy, Debug)]
pub enum TilePixel {
    #[default]
    Black, // 0b00
    LGray, // 0b01
    DGray, // 0b10
    White, // 0b11
}

impl From<[bool; 2]> for TilePixel {
    fn from(value: [bool; 2]) -> Self {
        match value {
            [false, false] => TilePixel::Black,
            [true, false] => TilePixel::LGray,
            [false, true] => TilePixel::DGray,
            [true, true] => TilePixel::White,
        }
    }
}

pub type Tile = [[TilePixel; 8]; 8];

#[derive(Clone, Copy, Debug)]
pub struct Gpu {
    pub tile_set: [Tile; span(VRAM_TILES)],
    pub vram: [u8; span(MM_VRAM)],
    pub canvas: [u8; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
}

impl Default for Gpu {
    fn default() -> Self {
        Gpu {
            tile_set: [[[Default::default(); 8]; 8]; span(VRAM_TILES)],
            vram: [Default::default(); span(MM_VRAM)],
            canvas: [Default::default(); SCREEN_WIDTH * SCREEN_HEIGHT * 4],
        }
    }
}

impl Gpu {
    pub fn read8(&self, index: usize) -> u8 {
        self.vram[index]
    }

    pub fn write8(&mut self, index: usize, value: u8) {
        self.vram[index] = value;

        if !VRAM_TILES.contains(&index) {
            return;
        }

        let norm_index = index & 0xFFFE;
        let lo_by = self.vram[norm_index];
        let hi_by = self.vram[norm_index + 1];

        let tile = index / 16;
        let row = (index % 16) / 2;

        for col in 0..8 {
            let lo = lo_by & (1 << (7 - col)) != 0;
            let hi = hi_by & (1 << (7 - col)) != 0;
            self.tile_set[tile][row][col] = TilePixel::from([lo, hi]);
        }
    }
}
