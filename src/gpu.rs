#![allow(unused)]

use std::ops::RangeBounds;

use crate::common::*;
use crate::mmu;
use crate::constants::*;

#[derive(Default, Clone, Copy, Debug)]
pub enum TileColor {
    #[default]
    Black,
    LGray,
    DGray,
    White,
}

impl From<[bool; 2]> for TileColor {
    fn from(value: [bool; 2]) -> Self {
        match value {
            [false, false] => TileColor::Black,
            [true, false] => TileColor::LGray,
            [false, true] => TileColor::DGray,
            [true, true] => TileColor::White,
        }
    }
}

impl From<TileColor> for u8 {
    fn from(value: TileColor) -> Self {
        match value {
            TileColor::Black => 0b00,
            TileColor::LGray => 0b01,
            TileColor::DGray => 0b10,
            TileColor::White => 0b11,
        }
    }
}

const TILE_SIDE: usize = 8;
type Tile = [[TileColor; TILE_SIDE]; TILE_SIDE];

#[derive(Clone, Copy, Debug)]
pub struct Gpu {
    pub tiles: [Tile; Self::TILES_COUNT],
    pub vram: [u8; Self::VRAM_SIZE],
    pub canvas: [u8; Self::CANVAS_SIZE],
}

impl Gpu {
    const TILES_COUNT: usize = VRAM::TILE_BLOCKS.span();
    const VRAM_SIZE: usize = mmu::vram_range!().span();
    const CANVAS_SIZE: usize = 256 * 256 * 4;

    // TODO: address mode
    fn tile_idx(idx: u8) -> usize {
        idx as usize
    }

    pub fn read(&self, addr: usize) -> u8 {
        self.vram[addr]
    }

    pub fn write(&mut self, addr: usize, value: u8) {
        self.vram[addr] = value;

        if !VRAM::TILE_BLOCKS.contains(&addr) {
            return;
        }

        let base_addr = addr & !1;
        let lo_by = self.vram[base_addr];
        let hi_by = self.vram[base_addr + 1];

        let tile_idx = addr / 16;
        let row = (addr % 16) / 2;

        for col in 0..TILE_SIDE {
            let bitmask = 1 << (TILE_SIDE - 1 - col);
            let lo = (lo_by & bitmask) != 0;
            let hi = (hi_by & bitmask) != 0;
            self.tiles[tile_idx][row][col] = TileColor::from([lo, hi]);
        }
    }

    fn draw_tile(&mut self, row: usize, col: usize, idx: u8) {
        for i in 0..8 {
            for j in 0..8 {
                let crow = row + i;
                let ccol = col + j;
                self.canvas[crow + (ccol * 32)] = self.tiles[Self::tile_idx(idx)][row][col].into();
            }
        }
    }

    fn draw(&mut self) {
        for addr in 0..VRAM::TMAP0.span() {
            let x = (addr / 32) * 8;
            let y = (addr % 32) * 8;
            let idx = self.vram[addr + VRAM::TMAP0.start];
            self.draw_tile(x, y, idx);
        }
    }
}

impl Default for Gpu {
    fn default() -> Self {
        Gpu {
            tiles: [Default::default(); Self::TILES_COUNT],
            vram: [0; Self::VRAM_SIZE],
            canvas: [0; Self::CANVAS_SIZE],
        }
    }
}
