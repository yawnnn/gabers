use crate::common::*;
use crate::constants::*;
use crate::mmu::{oam_range, tilemaps_range, tiles_range};

const TILES_START: u16 = *tiles_range!().start() as u16;
const TILE_SIDE: usize = 8; // 8x8 square
const TILE_SIZE: usize = TILE_SIDE * TILE_SIDE / 8 * 2; // 8x8 pixels, 2 bits per pixel
const TILE_COUNT: usize = 128;
const TILEMAPS_START: u16 = *tilemaps_range!().start() as u16;
const TILEMAP_SIDE: usize = 32; // 32x32 square
const OBJ_START: u16 = *oam_range!().start() as u16;
const OBJ_SIZE: usize = 4;
const OBJ_COUNT: usize = 40;
const MAX_OBJS_X_LINE: usize = 10;

fn color_id(lo: u8, hi: u8, bit: usize) -> u8 {
    let lo_bit = (lo >> bit) & 1;
    let hi_bit = (hi >> bit) & 1;
    hi_bit << 1 | lo_bit
}

#[derive(Clone, Copy)]
struct Palette(u8);

impl Palette {
    fn grayscale(self, color_id: u8) -> GrayScale {
        let color_shade = self.0 >> (color_id * 2) & 0b11;
        GrayScale::from(color_shade)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum GrayScale {
    #[default]
    Black = 0x00,
    LGray = 0x60,
    DGray = 0xC0,
    White = 0xFF,
}

impl From<u8> for GrayScale {
    fn from(shade: u8) -> Self {
        match shade & 0b11 {
            0b00 => GrayScale::Black,
            0b01 => GrayScale::LGray,
            0b10 => GrayScale::DGray,
            0b11 => GrayScale::White,
            _ => unreachable!(),
        }
    }
}

impl GrayScale {
    fn rgb(self) -> [u8; 3] {
        let c = self as u8;
        [c, c, c]
    }
}

#[derive(Clone, Copy)]
struct LcdControl(u8);

#[rustfmt::skip]
impl LcdControl {
    const LCD_ENABLE: u8       = 1 << 7; // LCDC.7
    const WINDOW_TILEMAP_2: u8 = 1 << 6; // LCDC.6
    const WINDOW_ENABLE: u8    = 1 << 5; // LCDC.5
    const TILE_BLOCK_2: u8     = 1 << 4; // LCDC.4
    const BG_TILEMAP_2: u8     = 1 << 3; // LCDC.3
    const OBJ_SIZE_16: u8      = 1 << 2; // LCDC.2
    const OBJ_ENABLE: u8       = 1 << 1; // LCDC.1
    const BG_ENABLE: u8        = 1 << 0; // LCDC.0

    fn get(&self, bit: u8) -> bool {
        (self.0 & bit) != 0
    }
}

struct ObjectFlags(u8);

#[rustfmt::skip]
impl ObjectFlags {
    const PRIORITY: u8   = 1 << 7;
    const FLIP_Y: u8     = 1 << 6;
    const FLIP_X: u8     = 1 << 5;
    const PALETTE_2: u8  = 1 << 4;

    fn get(&self, bit: u8) -> bool {
        (self.0 & bit) != 0
    }
}

pub struct Gpu {
    pub buf: [[u8; 3]; SCREEN_W * SCREEN_H],
    tiles: [[[u8; TILE_SIZE]; TILE_COUNT]; 3],
    tilemaps: [[[u8; TILEMAP_SIDE]; TILEMAP_SIDE]; 2],
    oam: [[u8; OBJ_SIZE]; OBJ_COUNT],
    lcdc: LcdControl,           // LCDC
    current_y: u8,              // LY
    scroll_x: u8,               // SCX
    scroll_y: u8,               // SCY
    window_x: u8,               // WX
    window_y: u8,               // WY
    bg_palette: Palette,        // BGP
    obj_palettes: [Palette; 2], // OBP0, OBP1
    priority: [bool; SCREEN_W],
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            buf: [[0; 3]; SCREEN_W * SCREEN_H],
            tiles: [[[0; TILE_SIZE]; TILE_COUNT]; 3],
            tilemaps: [[[0; TILEMAP_SIDE]; TILEMAP_SIDE]; 2],
            oam: [[0; OBJ_SIZE]; OBJ_COUNT],
            lcdc: LcdControl(0),
            current_y: 0,
            scroll_x: 0,
            scroll_y: 0,
            window_x: 0,
            window_y: 0,
            bg_palette: Palette(0),
            obj_palettes: [Palette(0); 2],
            priority: [false; SCREEN_W],
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            tiles_range!() => *self.tiles.get_3d(addr - TILES_START),
            tilemaps_range!() => *self.tilemaps.get_3d(addr - TILEMAPS_START),
            oam_range!() => *self.oam.get_2d(addr - OBJ_START),
            0xFF40 => self.lcdc.0,
            0xFF44 => self.current_y,
            0xFF47 => self.bg_palette.0,
            0xFF48 => self.obj_palettes[0].0,
            0xFF49 => self.obj_palettes[1].0,
            0xFF4A => self.window_x,
            0xFF4B => self.window_y,
            _ => todo!(),
        }
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            tiles_range!() => self.tiles.set_3d(addr - TILES_START, val),
            tilemaps_range!() => self.tilemaps.set_3d(addr - TILEMAPS_START, val),
            oam_range!() => self.oam.set_2d(addr - OBJ_START, val),
            0xFF40 => self.lcdc.0 = val,
            0xFF44 => (),
            0xFF47 => self.bg_palette.0 = val,
            0xFF48 => self.obj_palettes[0].0 = val,
            0xFF49 => self.obj_palettes[1].0 = val,
            0xFF4A => self.window_x = val,
            0xFF4B => self.window_y = val,
            _ => todo!(),
        }
    }

    fn set_pixel(&mut self, x: usize, grayscale: GrayScale) {
        let idx = x + self.current_y as usize * SCREEN_H;
        self.buf[idx] = grayscale.rgb();
    }

    fn draw_background_line_with(
        &mut self,
        shift_x: u8,
        scroll_x: u8,
        scroll_y: u8,
        lcdc_tilemap_2: u8,
    ) {
        let tilemap = if self.lcdc.get(lcdc_tilemap_2) { 0 } else { 1 };
        let tilemap_y = self.current_y.wrapping_add(scroll_y) as usize / TILEMAP_SIDE;

        for i in (0..SCREEN_W as u8).step_by(TILE_SIDE) {
            let start_x = i.wrapping_add(shift_x) as usize;
            if start_x >= SCREEN_W {
                continue;
            }
            let tilemap_x = i.wrapping_add(scroll_x) as usize / TILEMAP_SIDE;
            let tile_id = self.tilemaps[tilemap][tilemap_x][tilemap_y];
            let tile_id = if self.lcdc.get(LcdControl::TILE_BLOCK_2) {
                ((TILE_COUNT * 2) as i16 + (tile_id as i8 as i16)) as u16
            } else {
                tile_id as u16
            };
            let tile = self.tiles.get_2d(tile_id);
            let tile_byte = (self.current_y as usize % TILE_SIDE) * 2;
            let lo = tile[tile_byte];
            let hi = tile[tile_byte + 1];
            for j in 0..TILE_SIDE {
                let color_id = color_id(lo, hi, TILE_SIDE - 1 - j);
                let grayscale = self.bg_palette.grayscale(color_id);
                self.priority[start_x + j] = color_id != 0;
                self.set_pixel(start_x + j, grayscale);
            }
        }
    }

    fn draw_background_line(&mut self) {
        // the background can shifts the tiles (scroll), but draws the whole screen
        // the window can shifts the output area (shift), but uses the same tiles
        if self.lcdc.get(LcdControl::WINDOW_ENABLE) && self.window_y <= self.current_y {
            let shift_x = self.window_x.wrapping_sub(7);
            self.draw_background_line_with(shift_x, 0, 0, LcdControl::WINDOW_TILEMAP_2);
        } else if self.lcdc.get(LcdControl::BG_ENABLE) {
            self.draw_background_line_with(
                0,
                self.scroll_x,
                self.scroll_y,
                LcdControl::BG_TILEMAP_2,
            );
        }
    }

    fn draw_objs_line(&mut self) {
        struct Object {
            x: u8,
            y: u8,
            tile_id: u8,
            flags: ObjectFlags,
        }

        if !self.lcdc.get(LcdControl::OBJ_ENABLE) {
            return;
        }

        let size = if self.lcdc.get(LcdControl::OBJ_SIZE_16) {
            TILE_SIDE * 2
        } else {
            TILE_SIDE
        };

        let mut objs = self
            .oam
            .iter()
            .filter_map(|&[y, x, tile_id, flags]| {
                let y = y.wrapping_sub(16);
                let x = x.wrapping_sub(8);
                let tile_id = if size > TILE_SIDE {
                    tile_id & 0xFE
                } else {
                    tile_id
                };
                let flags = ObjectFlags(flags);
                let y_range = y..(y + size as u8);
                y_range.contains(&self.current_y).then_some(Object {
                    y,
                    x,
                    tile_id,
                    flags,
                })
            })
            .take(MAX_OBJS_X_LINE)
            .enumerate()
            .collect::<Vec<_>>();

        objs.sort_by_key(|(i, obj)| (obj.x, *i));

        // start from the last one so i overdraw overlapping ones with the highest priority one
        for (_, obj) in objs.into_iter().rev() {
            let palette = if obj.flags.get(ObjectFlags::PALETTE_2) {
                1
            } else {
                0
            };

            let mut tile_y = obj.y % size as u8;
            if obj.flags.get(ObjectFlags::FLIP_Y) {
                tile_y = size as u8 - 1 - tile_y;
            }
            let tile_byte = tile_y as usize * 2;
            let tile = self.tiles.get_2d(obj.tile_id);
            let lo = tile[tile_byte];
            let hi = tile[tile_byte + 1];

            for j in 0..TILE_SIDE {
                let bit = if obj.flags.get(ObjectFlags::FLIP_X) {
                    TILE_SIDE - 1 - j
                } else {
                    j
                };
                let current_x = (obj.x as usize).wrapping_add(j);
                let color_id = color_id(lo, hi, bit);
                if current_x >= SCREEN_W
                    || color_id == 0
                    || (obj.flags.get(ObjectFlags::PRIORITY) && self.priority[current_x])
                {
                    continue;
                }
                let grayscale = self.obj_palettes[palette].grayscale(color_id);
                self.set_pixel(current_x, grayscale);
            }
        }
    }

    fn draw_line(&mut self) {
        self.draw_background_line();
        self.draw_objs_line();
    }

    pub fn draw(&mut self) {
        if !self.lcdc.get(LcdControl::LCD_ENABLE) {
            return;
        }

        for y in 0..SCREEN_H {
            self.current_y = y as u8;
            self.draw_line();
        }
    }
}
