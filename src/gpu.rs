use crate::common::*;
use crate::constants::*;
use crate::mmu::vram_range;

const VRAM_START: u16 = *vram_range!().start() as u16;
const VRAM_SIZE: usize = vram_range!().span();
const TILE_SIDE: usize = 8; // 8x8 square
const TILE_BYTES: usize = TILE_SIDE * TILE_SIDE / 8 * 2; // 8x8 pixels, 2 bits per pixel
const TILEMAP_SIDE: usize = 32; // 32x32 square

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

pub struct Gpu {
    pub buf: [[u8; 3]; SCREEN_W * SCREEN_H],
    vram: [u8; VRAM_SIZE],
    lcdc: LcdControl,
    current_y: u8,  // LY
    scroll_x: u8,   // SCX
    scroll_y: u8,   // SCY
    window_x: u8,   // WX
    window_y: u8,   // WY
    bg_palette: u8, // BGP
    priority: [bool; SCREEN_W],
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            buf: [[0; 3]; SCREEN_W * SCREEN_H],
            vram: [0; VRAM_SIZE],
            lcdc: LcdControl::default(),
            current_y: 0,
            scroll_x: 0,
            scroll_y: 0,
            window_x: 0,
            window_y: 0,
            bg_palette: 0,
            priority: [false; SCREEN_W],
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            vram_range!() => self.vram[(addr - VRAM_START) as usize],
            0xFF40 => self.lcdc.0,
            0xFF44 => self.current_y,
            0xFF47 => self.bg_palette,
            0xFF4A => self.window_x,
            0xFF4B => self.window_y,
            _ => todo!(),
        }
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            vram_range!() => self.vram[(addr - VRAM_START) as usize] = val,
            0xFF40 => self.lcdc.0 = val,
            0xFF44 => (),
            0xFF47 => self.bg_palette = val,
            0xFF4A => self.window_x = val,
            0xFF4B => self.window_y = val,
            _ => todo!(),
        }
    }

    fn get_grayscale(&self, color_id: u8) -> GrayScale {
        let color_shade = self.bg_palette >> (color_id * 2) & 0b11;
        GrayScale::from(color_shade)
    }

    fn set_pixel(&mut self, x: usize, grayscale: GrayScale) {
        let idx = x + self.current_y as usize * SCREEN_H;
        self.buf[idx] = grayscale.rgb();
    }

    fn draw_tile(&mut self, tile_id: u8, current_x: usize) {
        let (tile_base, tile_offset): (u16, i16) = if self.lcdc.get(LcdControl::TILE_BLOCK_2) {
            (0x9000, tile_id as i8 as i16)
        } else {
            (0x8000, tile_id as i16)
        };
        let tile_addr = tile_base.wrapping_add_signed(tile_offset * TILE_BYTES as i16);
        let tile_byte = ((self.current_y as usize % TILE_SIDE) * 2) as u16;
        let lo_by = self.read8(tile_addr + tile_byte);
        let hi_by = self.read8(tile_addr + tile_byte + 1);
        for j in 0..TILE_SIDE {
            let bitmask = 1 << (TILE_SIDE - 1 - j);
            let lo = (lo_by & bitmask) != 0;
            let hi = (hi_by & bitmask) != 0;
            let color_id = (hi as u8) << 1 | lo as u8;
            let grayscale = self.get_grayscale(color_id);
            self.priority[current_x + j] = color_id != 0;
            self.set_pixel(current_x + j, grayscale);
        }
    }

    fn draw_line_background(&mut self) {
        fn inner(gpu: &mut Gpu, start_x: u8, scroll_x: u8, scroll_y: u8, lcdc_tilemap_2: u8) {
            let tilemap_base = if gpu.lcdc.get(lcdc_tilemap_2) { 0x9C00 } else { 0x9800 };
            let tilemap_y = gpu.current_y.wrapping_add(scroll_y) as u16 / TILEMAP_SIDE as u16;

            for x in (0..SCREEN_W as u8).step_by(TILE_SIDE) {
                let current_x = x.wrapping_add(start_x) as usize;
                if current_x >= SCREEN_W {
                    continue;
                }
                let tilemap_x = x.wrapping_add(scroll_x) as u16 / TILEMAP_SIDE as u16;
                let tilemap_offset = tilemap_x + tilemap_y * TILEMAP_SIDE as u16;
                let tile_id = gpu.read8(tilemap_base + tilemap_offset);
                gpu.draw_tile(tile_id, current_x);
            }
        }

        // the window shifts where it draws the pixels, but keeps the same tilemap
        // the background shifts the tilemap, but always draws the whole screen
        if self.lcdc.get(LcdControl::WINDOW_ENABLE) && self.window_y <= self.current_y {
            let start_x = self.window_x.wrapping_sub(7);
            inner(self, start_x, 0, 0, LcdControl::WINDOW_TILEMAP_2);
        } else if self.lcdc.get(LcdControl::BG_ENABLE) {
            inner(self, 0, self.scroll_x, self.scroll_y, LcdControl::BG_TILEMAP_2);
        }
    }

    fn draw_line(&mut self) {
        self.draw_line_background();
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
