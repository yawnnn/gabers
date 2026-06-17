use crate::common::*;
use crate::constants::*;
use crate::gameboy::Gameboy;
use crate::interrupt::Interrupt;
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

struct ObjectFlags(u8);

#[rustfmt::skip]
impl ObjectFlags {
    const PALETTE_2: u8  = 1 << 4;
    const FLIP_X: u8     = 1 << 5;
    const FLIP_Y: u8     = 1 << 6;
    const PRIORITY: u8   = 1 << 7;

    fn get(&self, bit: u8) -> bool {
        (self.0 & bit) != 0
    }
}

struct Stat(u8);

impl Stat {
    //const GPU_MODE: u8      = 0b11;
    const LYC_EQ_LC: u8 = 1 << 1;
    const INT_HBLANK: u8 = 1 << 2;
    const INT_VBLANK: u8 = 1 << 3;
    const INT_OAM_SCAN: u8 = 1 << 4;
    const INT_LYC_EQ_LC: u8 = 1 << 5;
    const WRITABLE_MASK: u8 = Self::INT_HBLANK | Self::INT_VBLANK | Self::INT_OAM_SCAN | Self::INT_LYC_EQ_LC;

    fn get(&self, bit: u8) -> bool {
        (self.0 & bit) != 0
    }

    fn set(&mut self, bit: u8) {
        self.0 |= bit;
    }

    fn read8(&self, lyc_eq_lc: bool, mode: GpuMode) -> u8 {
        let lyc_bit = if lyc_eq_lc { Self::LYC_EQ_LC } else  { 0 };
        0x80 | self.0 | lyc_bit | mode as u8
    }

    fn write8(&mut self, val: u8) {
        self.0 = val & Stat::WRITABLE_MASK
    }
}

#[derive(Clone, Copy)]
enum GpuMode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Draw = 3,
}

impl GpuMode {
    const OAM_SCAN_END: u32 = 80;
    const DRAW_END: u32 = Self::OAM_SCAN_END + 172;
    const HBLANK_END: u32 = Self::DRAW_END + 204;
    const FRAME_END: u32 = 456;
    const VBLANK_LEN: usize = 10;
}

pub struct Gpu {
    pub buf: [[u8; 3]; SCREEN_W * SCREEN_H],
    tiles: [[[u8; TILE_SIZE]; TILE_COUNT]; 3],
    tilemaps: [[[u8; TILEMAP_SIDE]; TILEMAP_SIDE]; 2],
    oam: [[u8; OBJ_SIZE]; OBJ_COUNT],
    lcdc: LcdControl, // LCDC
    stat: Stat,       // STAT: LCD status
    mode: GpuMode,
    dots: u32,
    current_y: u8,              // LY
    y_cmp: u8,                  // LYC
    scroll_x: u8,               // SCX
    scroll_y: u8,               // SCY
    window_x: u8,               // WX
    window_y: u8,               // WY
    bg_palette: Palette,        // BGP
    obj_palettes: [Palette; 2], // OBP0, OBP1
    priority: [bool; SCREEN_W],
    pub gb: *mut Gameboy,
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            buf: [[0; 3]; SCREEN_W * SCREEN_H],
            tiles: [[[0; TILE_SIZE]; TILE_COUNT]; 3],
            tilemaps: [[[0; TILEMAP_SIDE]; TILEMAP_SIDE]; 2],
            oam: [[0; OBJ_SIZE]; OBJ_COUNT],
            lcdc: LcdControl(0),
            stat: Stat(0),
            mode: GpuMode::OamScan,
            dots: 0,
            current_y: 0,
            y_cmp: 0,
            scroll_x: 0,
            scroll_y: 0,
            window_x: 0,
            window_y: 0,
            bg_palette: Palette(0),
            obj_palettes: [Palette(0); 2],
            priority: [false; SCREEN_W],
            gb: std::ptr::null_mut(),
        }
    }

    pub fn gb(&mut self) -> &mut Gameboy {
        assert!(!self.gb.is_null(), "Expected valid pointer");
        // SAFETY: this is used to access data inside gb that's not already "in scope" (eg. cpu.gb().timer), so aliasing *shouldn't* be an issue
        // TODO: this is still pretty unsafe and should be removed
        unsafe { self.gb.as_mut().unwrap() }
    }

    fn oam_access(&self) -> bool {
        !matches!(self.mode, GpuMode::OamScan | GpuMode::Draw)
    }

    fn vram_access(&self) -> bool {
        !matches!(self.mode, GpuMode::Draw)
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            tiles_range!() => {
                if self.vram_access() {
                    *self.tiles.get_3d(addr - TILES_START)
                } else {
                    0xFF
                }
            }
            tilemaps_range!() => {
                if self.vram_access() {
                    *self.tilemaps.get_3d(addr - TILEMAPS_START)
                } else {
                    0xFF
                }
            }
            oam_range!() => {
                if self.oam_access() {
                    *self.oam.get_2d(addr - OBJ_START)
                } else {
                    0xFF
                }
            }
            0xFF40 => self.lcdc.0,
            0xFF41 => self.stat.read8(self.current_y == self.y_cmp, self.mode),
            0xFF44 => self.current_y,
            0xFF45 => self.y_cmp,
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
            tiles_range!() => {
                if self.vram_access() {
                    self.tiles.set_3d(addr - TILES_START, val)
                }
            }
            tilemaps_range!() => {
                if self.vram_access() {
                    self.tilemaps.set_3d(addr - TILEMAPS_START, val)
                }
            }
            oam_range!() => {
                if self.oam_access() {
                    self.oam.set_2d(addr - OBJ_START, val)
                }
            }
            0xFF40 => self.lcdc.0 = val,
            0xFF41 => self.stat.write8(val),
            0xFF44 => (),
            0xFF45 => self.y_cmp = val,
            0xFF47 => self.bg_palette.0 = val,
            0xFF48 => self.obj_palettes[0].0 = val,
            0xFF49 => self.obj_palettes[1].0 = val,
            0xFF4A => self.window_x = val,
            0xFF4B => self.window_y = val,
            _ => todo!(),
        }
    }

    pub fn dma_transfer(gb: &mut Gameboy, addr_hi: u8) {
        let src_base = (addr_hi as u16) << 8;
        let nbytes = std::mem::size_of_val(&gb.gpu.oam);
        for i in 0..nbytes {
            let by = gb.read8(src_base + i as u16);
            gb.gpu.oam.set_2d(i, by); // this ignores GpuMode r/w blocks
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

                let current_x = obj.x as usize + j;
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

    fn inc_current_y(&mut self) {
        self.current_y += 1;
        if self.stat.get(Stat::INT_LYC_EQ_LC) && self.current_y == self.y_cmp {
            self.gb().inter_flag.raise(Interrupt::LCD);
        }
    }

    fn change_mode(&mut self, new_mode: GpuMode) {
        self.mode = new_mode;
        let bit = match new_mode {
            GpuMode::HBlank =>  Stat::INT_HBLANK,
            GpuMode::VBlank =>  Stat::INT_VBLANK,
            GpuMode::OamScan => Stat::INT_OAM_SCAN,
            _ => 0,
        };
        if self.stat.get(bit) {
            self.gb().inter_flag.raise(Interrupt::LCD);
        }
    }

    pub fn draw(&mut self, cycles: u32) {
        if !self.lcdc.get(LcdControl::LCD_ENABLE) {
            return;
        }

        let frame_dots = (self.dots % GpuMode::FRAME_END) + cycles;
        self.dots += cycles;

        match self.mode {
            GpuMode::OamScan => {
                if frame_dots >= GpuMode::OAM_SCAN_END {
                    self.change_mode(GpuMode::Draw);
                }
            }
            GpuMode::Draw => {
                if frame_dots >= GpuMode::DRAW_END {
                    self.draw_line();
                    self.change_mode(GpuMode::HBlank);
                }
            }
            GpuMode::HBlank => {
                if frame_dots >= GpuMode::HBLANK_END {
                    self.inc_current_y();
                    let new_mode = if self.current_y == SCREEN_H as u8 {
                        self.gb().inter_flag.raise(Interrupt::VBLANK);
                        GpuMode::VBlank
                    } else {
                        GpuMode::OamScan
                    };
                    self.change_mode(new_mode);
                }
            }
            GpuMode::VBlank => {
                if frame_dots >= GpuMode::FRAME_END {
                    self.inc_current_y();
                    if self.current_y == (SCREEN_H + GpuMode::VBLANK_LEN) as u8 {
                        self.current_y = 0;
                        self.dots = 0;
                        self.change_mode(GpuMode::OamScan);
                    }
                }
            }
        }
    }
}
