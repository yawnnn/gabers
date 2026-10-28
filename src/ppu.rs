use std::ops::{Deref, DerefMut};

use crate::common::*;
use crate::gameboy::*;
use crate::interrupt::*;
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

#[derive(Clone, Copy, Default)]
struct Palette(u8);

impl Palette {
    fn grayscale(self, color_id: u8) -> GrayScale {
        let color_shade = self.0 >> (color_id * 2) & 0b11;
        GrayScale::from(color_shade)
    }
}

impl Deref for Palette {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Palette {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default, Clone, Copy)]
struct LcdControl(u8);

#[rustfmt::skip]
impl LcdControl {
    const BG_ENABLE: u8        = 1 << 0; // LCDC.0
    const OBJ_ENABLE: u8       = 1 << 1; // LCDC.1
    const OBJ_SIZE_16: u8      = 1 << 2; // LCDC.2
    const BG_TILEMAP_2: u8     = 1 << 3; // LCDC.3
    const TILE_BLOCK_2: u8     = 1 << 4; // LCDC.4
    const WINDOW_ENABLE: u8    = 1 << 5; // LCDC.5
    const WINDOW_TILEMAP_2: u8 = 1 << 6; // LCDC.6
    const LCD_ENABLE: u8       = 1 << 7; // LCDC.7

    fn get(&self, bit: u8) -> bool {
        (self.0 & bit) != 0
    }
}

impl Deref for LcdControl {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LcdControl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy, Default)]
struct LcdStatus {
    pub val: u8,         // current value
    pub line_high: bool, // current status
}

#[rustfmt::skip]
impl LcdStatus {
    const LYC_EQ_LC: u8     = 1 << 1;
    const INT_HBLANK: u8    = 1 << 2;
    const INT_VBLANK: u8    = 1 << 3;
    const INT_OAM_SCAN: u8  = 1 << 4;
    const INT_LYC_EQ_LC: u8 = 1 << 5;
    const WRITABLE_MASK: u8 = Self::INT_HBLANK | Self::INT_VBLANK | Self::INT_OAM_SCAN | Self::INT_LYC_EQ_LC;

    fn read8(&self, lyc_eq_lc: bool, mode: Mode) -> u8 {
        let lyc_bit = if lyc_eq_lc { Self::LYC_EQ_LC } else { 0 };
        0x80 | self.val | lyc_bit | mode as u8
    }

    fn write8(&mut self, val: u8) {
        self.val = val & LcdStatus::WRITABLE_MASK
    }
}

struct ObjFlags(u8);

#[rustfmt::skip]
impl ObjFlags {
    const PALETTE_2: u8  = 1 << 4;
    const FLIP_X: u8     = 1 << 5;
    const FLIP_Y: u8     = 1 << 6;
    const PRIORITY: u8   = 1 << 7;

    fn get(&self, bit: u8) -> bool {
        (self.0 & bit) != 0
    }
}

struct Object {
    x: u8,
    y: u8,
    size: u8,
    tile_id: u16,
    flags: ObjFlags,
}

impl Object {
    fn new(x: u8, y: u8, mut tile_id: u8, flags: u8, obj16: bool) -> Self {
        let x = x.wrapping_sub(8);
        let y = y.wrapping_sub(16);
        let size = if obj16 {
            tile_id &= 0xFE;
            TILE_SIDE * 2
        } else {
            TILE_SIDE
        };

        Object {
            x,
            y,
            size: size as u8,
            tile_id: tile_id as u16,
            flags: ObjFlags(flags),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    #[default]
    OamScan = 2,
    Draw = 3,
}

impl Mode {
    const OAM_SCAN_END: u32 = 80;
    const DRAW_END: u32 = Self::OAM_SCAN_END + 172;
    const HBLANK_END: u32 = Self::DRAW_END + 204;
    const SCANLINE_END: u32 = 456;
    const VBLANK_LEN: usize = 10;
}

type Pixel = [u8; 3];
type Tile = [u8; TILE_SIZE];
type Tilemap = [[u8; TILEMAP_SIDE]; TILEMAP_SIDE];
pub type Oam = [[u8; OBJ_SIZE]; OBJ_COUNT];

pub struct Ppu {
    pub buf: Box<[Pixel; SCREEN_W * SCREEN_H]>,
    tiles: Box<[[Tile; TILE_COUNT]; 3]>,
    tilemaps: Box<[Tilemap; 2]>,
    oam: Box<Oam>,
    mode: Mode,
    dots_count: u32,                   // total dots elapsed this frame
    bg_is_used: Box<[bool; SCREEN_W]>, // if the bg/window pixel is currently used
    lcdc: LcdControl,                  // LCDC
    stat: LcdStatus,                   // STAT
    current_y: u8,                     // LY
    y_cmp: u8,                         // LYC
    scroll_x: u8,                      // SCX
    scroll_y: u8,                      // SCY
    window_x: u8,                      // WX
    window_y: u8,                      // WY
    bg_palette: Palette,               // BGP
    obj_palettes: [Palette; 2],        // OBP0, OBP1
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            buf: boxed_2d(),
            tiles: boxed_3d(),
            tilemaps: boxed_3d(),
            oam: boxed_2d(),
            mode: Mode::default(),
            dots_count: 0,
            bg_is_used: boxed_1d(),
            lcdc: LcdControl::default(),
            stat: LcdStatus::default(),
            current_y: 0,
            y_cmp: 0,
            scroll_x: 0,
            scroll_y: 0,
            window_x: 0,
            window_y: 0,
            bg_palette: Palette::default(),
            obj_palettes: Default::default(),
        }
    }

    fn oam_access(&self) -> bool {
        !matches!(self.mode, Mode::OamScan | Mode::Draw)
    }

    fn vram_access(&self) -> bool {
        !matches!(self.mode, Mode::Draw)
    }

    fn lyc_is_ly(&self) -> bool {
        self.current_y == self.y_cmp
    }

    fn check_stat(&mut self, inter: &mut Interrupts) {
        let mut bits = match self.mode {
            Mode::HBlank => LcdStatus::INT_HBLANK,
            Mode::VBlank => LcdStatus::INT_VBLANK,
            Mode::OamScan => LcdStatus::INT_OAM_SCAN,
            _ => 0,
        };
        if self.lyc_is_ly() {
            bits |= LcdStatus::INT_LYC_EQ_LC;
        }
        let line_high = self.stat.val & bits != 0;
        if !self.stat.line_high && line_high {
            inter.raise(InterruptFlags::LCD);
        }
        self.stat.line_high = line_high;
    }

    fn set_pixel(&mut self, x: usize, grayscale: GrayScale) {
        let idx = x + self.current_y as usize * SCREEN_W;
        self.buf[idx] = grayscale.rgb();
    }

    fn get_tile_dword(&self, tile_id: u16, y: u8) -> [u8; 2] {
        let tile = self.tiles.get_1to2(tile_id);
        let tile_byte = y as usize * 2;
        let lo = tile[tile_byte];
        let hi = tile[tile_byte + 1];
        [lo, hi]
    }

    fn render_bg_tile(&mut self, tile_id: u16, start_x: u8, y: u8, window_x: i16)  {
        let start_tile_x = start_x as usize % TILE_SIDE;
        let tile_y = y % TILE_SIDE as u8;
        let [lo, hi] = self.get_tile_dword(tile_id, tile_y);

        for tile_x in start_tile_x..TILE_SIDE {
            let x = tile_x - start_tile_x;
            let screen_x = x as i16 + window_x;
            if screen_x < 0 || screen_x >= SCREEN_W as i16 {
                continue;
            }
            let color_id = color_id(lo, hi, TILE_SIDE - 1 - tile_x);
            let grayscale = self.bg_palette.grayscale(color_id);
            self.bg_is_used[screen_x as usize] = color_id != 0;
            self.set_pixel(screen_x as usize, grayscale);
        }
    }

    fn render_bg_line_with(
        &mut self,
        start_window_x: i16,
        scroll_x: u8,
        scroll_y: u8,
        lcdc_tilemap_2: u8,
    ) {
        let tilemap = self.lcdc.get(lcdc_tilemap_2) as usize;
        let y = self.current_y.wrapping_add(scroll_y);
        let tilemap_y = y as usize / TILE_SIDE;

        // iterate SCREEN_W+1 pixels to handle partial tiles due to scroll
        for current_x in (0..=SCREEN_W as u8).step_by(TILE_SIDE) {
            let window_x = current_x as i16 + start_window_x;
            let start_x = current_x.wrapping_add(scroll_x);
            let tilemap_x = start_x as usize / TILE_SIDE;
            let tile_id = self.tilemaps[tilemap][tilemap_y][tilemap_x];
            let tile_id = if self.lcdc.get(LcdControl::TILE_BLOCK_2) {
                tile_id as u16
            } else {
                ((TILE_COUNT * 2) as i16 + (tile_id as i8 as i16)) as u16
            };
            self.render_bg_tile(tile_id, start_x, y, window_x);
        }
    }

    fn render_bg_line(&mut self) {
        // the scroll shifts the tilemap, so the pixel values
        if self.lcdc.get(LcdControl::BG_ENABLE) {
            self.render_bg_line_with(0, self.scroll_x, self.scroll_y, LcdControl::BG_TILEMAP_2);
        }

        // the window shifts the output position
        if self.lcdc.get(LcdControl::WINDOW_ENABLE) && self.window_y <= self.current_y {
            let start_window_x = self.window_x as i16 - 7;
            self.render_bg_line_with(start_window_x, 0, 0, LcdControl::WINDOW_TILEMAP_2);
        }
    }

    fn render_obj(&mut self, obj: Object) {
        let palette = obj.flags.get(ObjFlags::PALETTE_2) as usize;
        let mut tile_y = self.current_y.wrapping_sub(obj.y);
        if obj.flags.get(ObjFlags::FLIP_Y) {
            tile_y = obj.size - 1 - tile_y;
        }
        let tile_id = obj.tile_id + (tile_y as u16 / TILE_SIDE as u16);
        let [lo, hi] = self.get_tile_dword(tile_id, tile_y);

        for j in 0..TILE_SIDE {
            let bit = if obj.flags.get(ObjFlags::FLIP_X) {
                j
            } else {
                TILE_SIDE - 1 - j
            };

            let current_x = obj.x as usize + j;
            let color_id = color_id(lo, hi, bit);
            if current_x >= SCREEN_W
                || color_id == 0
                || (obj.flags.get(ObjFlags::PRIORITY) && self.bg_is_used[current_x])
            {
                continue;
            }

            let grayscale = self.obj_palettes[palette].grayscale(color_id);
            self.set_pixel(current_x, grayscale);
        }
    }

    fn render_obj_line(&mut self) {
        if !self.lcdc.get(LcdControl::OBJ_ENABLE) {
            return;
        }

        let obj16 = self.lcdc.get(LcdControl::OBJ_SIZE_16);

        let mut objs: Vec<_> = self
            .oam
            .iter()
            .map(|&[y, x, tile_id, flags]| Object::new(x, y, tile_id, flags, obj16))
            .filter(|obj| self.current_y.wrapping_sub(obj.y) < obj.size)
            .take(MAX_OBJS_X_LINE)
            .enumerate()
            .collect();

        objs.sort_by_key(|(i, obj)| (obj.x, *i));

        // start from the last one so i overdraw overlapping ones correctly
        for (_, obj) in objs.into_iter().rev() {
            self.render_obj(obj);
        }
    }

    fn render_line(&mut self) {
        self.bg_is_used.fill(false);
        self.render_bg_line();
        self.render_obj_line();
    }

    fn inc_current_y(&mut self, inter: &mut Interrupts) {
        self.current_y += 1;
        self.check_stat(inter);
    }

    fn change_mode(&mut self, inter: &mut Interrupts, new_mode: Mode) {
        self.mode = new_mode;
        self.check_stat(inter);
    }

    pub fn render(&mut self, inter: &mut Interrupts, cycles: TCycle) {
        if !self.lcdc.get(LcdControl::LCD_ENABLE) {
            return;
        }

        let scanline_dots = (self.dots_count % Mode::SCANLINE_END) + cycles;
        self.dots_count += cycles;

        match self.mode {
            Mode::OamScan => {
                if scanline_dots >= Mode::OAM_SCAN_END {
                    self.change_mode(inter, Mode::Draw);
                }
            }
            Mode::Draw => {
                if scanline_dots >= Mode::DRAW_END {
                    self.render_line();
                    self.change_mode(inter, Mode::HBlank);
                }
            }
            Mode::HBlank => {
                if scanline_dots >= Mode::HBLANK_END {
                    self.inc_current_y(inter);
                    let new_mode = if self.current_y == SCREEN_H as u8 {
                        inter.raise(InterruptFlags::VBLANK);
                        Mode::VBlank
                    } else {
                        Mode::OamScan
                    };
                    self.change_mode(inter, new_mode);
                }
            }
            Mode::VBlank => {
                if scanline_dots >= Mode::SCANLINE_END {
                    self.inc_current_y(inter);
                    if self.current_y == (SCREEN_H + Mode::VBLANK_LEN) as u8 {
                        self.current_y = 0;
                        self.dots_count = 0;
                        self.change_mode(inter, Mode::OamScan);
                    }
                }
            }
        }
    }

    pub fn write8_oam_unchecked(&mut self, pos: usize, val: u8) {
        self.oam.set_1to2(pos, val);
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            tiles_range!() if self.vram_access() => *self.tiles.get_1to3(addr - TILES_START),
            tilemaps_range!() if self.vram_access() => {
                *self.tilemaps.get_1to3(addr - TILEMAPS_START)
            }
            oam_range!() if self.oam_access() => *self.oam.get_1to2(addr - OBJ_START),
            0xFF40 => *self.lcdc,
            0xFF41 => self.stat.read8(self.lyc_is_ly(), self.mode),
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.current_y,
            0xFF45 => self.y_cmp,
            0xFF47 => *self.bg_palette,
            0xFF48 => *self.obj_palettes[0],
            0xFF49 => *self.obj_palettes[1],
            0xFF4A => self.window_x,
            0xFF4B => self.window_y,
            _ => 0xFF,
        }
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            tiles_range!() if self.vram_access() => self.tiles.set_1to3(addr - TILES_START, val),
            tilemaps_range!() if self.vram_access() => {
                self.tilemaps.set_1to3(addr - TILEMAPS_START, val)
            }
            oam_range!() if self.oam_access() => self.oam.set_1to2(addr - OBJ_START, val),
            0xFF40 => {
                *self.lcdc = val;
                if !self.lcdc.get(LcdControl::LCD_ENABLE) {
                    self.dots_count = 0;
                    self.current_y = 0;
                    self.stat.write8(0);
                    self.mode = Mode::VBlank;
                    self.buf.fill([0xFF; 3]);
                }
            }
            0xFF41 => self.stat.write8(val),
            0xFF42 => self.scroll_y = val,
            0xFF43 => self.scroll_x = val,
            0xFF44 => (),
            0xFF45 => self.y_cmp = val,
            0xFF47 => *self.bg_palette = val,
            0xFF48 => *self.obj_palettes[0] = val,
            0xFF49 => *self.obj_palettes[1] = val,
            0xFF4A => self.window_x = val,
            0xFF4B => self.window_y = val,
            _ => (),
        }
    }
}
