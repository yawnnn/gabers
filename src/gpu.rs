#![allow(unused)]

use std::ops::RangeBounds;

use crate::common::*;
use crate::memory::*;

#[derive(Default, Clone, Copy, Debug)]
pub enum TilePixel {
    #[default]
    Black,
    LGray,
    DGray,
    White,
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

impl From<TilePixel> for u8 {
    fn from(value: TilePixel) -> Self {
        match value {
            TilePixel::Black => 0b00,
            TilePixel::LGray => 0b01,
            TilePixel::DGray => 0b10,
            TilePixel::White => 0b11,
        }
    }
}

type Tile = [[TilePixel; 8]; 8];

#[derive(Clone, Copy, Debug)]
pub struct Gpu {
    pub tile_data: [Tile; Gpu::TILES_SIZE],
    pub vram: [u8; Gpu::VRAM_SIZE],
    pub canvas: [u8; Gpu::CANVAS_SIZE],
}

impl Gpu {
    const TILES_SIZE: usize = span(VRAM_TILES);
    const VRAM_SIZE: usize = span(MM_VRAM);
    const CANVAS_SIZE: usize = 256 * 256 * 4;

    fn vram_addr(addr: u16) -> usize {
        addr as usize - MM_VRAM.start
    }

    // TODO: address mode
    fn tile_addr(index: u8) -> usize {
        index as usize
    }

    pub fn read(&self, addr: u16) -> u8 {
        let vram_addr = Self::vram_addr(addr);
        self.vram[vram_addr]
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        let vram_addr = Self::vram_addr(addr);
        self.vram[vram_addr] = value;

        if !VRAM_TILES.contains(&vram_addr) {
            return;
        }

        let norm_index = vram_addr & 0xFFFE;
        let lo_by = self.vram[norm_index];
        let hi_by = self.vram[norm_index + 1];

        let tile = vram_addr / 16;
        let row = (vram_addr % 16) / 2;

        for col in 0..8 {
            let lo = lo_by & (1 << (7 - col)) != 0;
            let hi = hi_by & (1 << (7 - col)) != 0;
            self.tile_data[tile][row][col] = TilePixel::from([lo, hi]);
        }
    }

    fn draw_tile(&mut self, x: usize, y: usize, index: u8) {
        for i in 0..8 {
            for j in 0..8 {
                let cx = x + i;
                let cy = y + j;
                self.canvas[cx + (cy * 32)] = self.tile_data[Self::tile_addr(index)][x][y].into();
            }
        }
    }

    fn draw(&mut self) {
        for vram_addr in 0..span(VRAM_MAP0) {
            let x = (vram_addr / 32) * 8;
            let y = (vram_addr % 32) * 8;
            let index = self.vram[vram_addr + VRAM_MAP0.start];
            self.draw_tile(x, y, index);
        }
    }
}

impl Default for Gpu {
    fn default() -> Self {
        Gpu {
            tile_data: [Default::default(); Gpu::TILES_SIZE],
            vram: [0; Gpu::VRAM_SIZE],
            canvas: [0; Gpu::CANVAS_SIZE],
        }
    }
}
