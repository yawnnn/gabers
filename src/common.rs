#![allow(unused)]

pub const TILE_COUNT: usize = 384;
pub const VRAM_BEG: u16 = 1;
pub const VRAM_SIZE: usize = 1;
pub const VRAM_END: u16 = VRAM_BEG + VRAM_SIZE as u16;
pub const PIXEL_COUNT: usize = 1;
pub const ROM_BEG: usize = 0x0;
pub const ROM_END: usize = 0x100;
pub const GAME_ROM_BEG: usize = 0x100;
pub const GAME_ROM_END: usize = 0x3FFF;