use minifb::{Key, Window, WindowOptions};
use std::path::Path;

use crate::cartridge::Cartridge;
use crate::common::*;
use crate::cpu::Cpu;
use crate::interrupt::{InterruptFlags, Interrupts};
use crate::joypad::{Joypad, JoypadKey};
use crate::mmu::*;
use crate::ppu::*;
use crate::timer::Timer;

pub const MASTER_CLOCK: usize = 4_194_304;
pub const MASTER_SYSTEM_CLOCK_RATIO: usize = 4;
pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;
const TARGET_FPS: usize = 60;
const FRAME_CYCLES: u32 = (MASTER_CLOCK as f64 / TARGET_FPS as f64).ceil() as u32;

#[derive(Default, Clone, Copy)]
struct DmaTransfer {
    addr: u16,
    pos: usize,
}

impl DmaTransfer {
    fn new(start: u8) -> Self {
        DmaTransfer {
            addr: (start as u16) << 8,
            pos: 0,
        }
    }
}

impl Iterator for DmaTransfer {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= std::mem::size_of::<Oam>() {
            return None;
        }
        let val = *self;
        self.addr = self.addr.wrapping_add(1);
        self.pos += 1;
        Some(val)
    }
}

pub struct Gameboy {
    pub cartridge: Cartridge,
    pub ppu: Ppu,
    pub inter: Interrupts,
    pub joypad: Joypad,
    pub timer: Timer,
    pub wram: Box<[u8; wram_range!().span()]>,
    pub hram: Box<[u8; hram_range!().span()]>,
    dma_transfer: Option<DmaTransfer>,

    window: Window,
    screen: Box<[u32; SCREEN_W * SCREEN_H]>,
}

impl Gameboy {
    pub fn new(cartridge_path: impl AsRef<Path>) -> Box<Self> {
        let cartridge = Cartridge::new(cartridge_path.as_ref());

        let mut window = Window::new(
            &cartridge.title,
            SCREEN_W,
            SCREEN_H,
            WindowOptions::default(),
        )
        .unwrap();
        window.set_target_fps(TARGET_FPS);

        Box::new(Gameboy {
            cartridge,
            ppu: Ppu::new(),
            inter: Interrupts::new(),
            joypad: Joypad::new(),
            timer: Timer::new(),
            wram: boxed_1d(),
            hram: boxed_1d(),
            window,
            screen: boxed_1d(),
            dma_transfer: None,
        })
    }

    fn draw(&mut self) {
        for (i, rgb) in self.ppu.buf.iter().enumerate() {
            let [r, g, b] = *rgb;
            self.screen[i] = u32::from_le_bytes([r, g, b, 0xFF]);
        }
        self.window
            .update_with_buffer(self.screen.as_ref(), SCREEN_W, SCREEN_H)
            .unwrap();
    }

    fn handle_input(&mut self) -> bool {
        let joypad_keys = [
            (Key::Right, JoypadKey::Right),
            (Key::Up, JoypadKey::Up),
            (Key::Left, JoypadKey::Left),
            (Key::Down, JoypadKey::Down),
            (Key::Z, JoypadKey::A),
            (Key::X, JoypadKey::B),
            (Key::Space, JoypadKey::Select),
            (Key::Enter, JoypadKey::Start),
        ];

        for (key, joypad_key) in joypad_keys {
            if self.window.is_key_down(key) {
                self.joypad.press(&mut self.inter, joypad_key);
                self.inter.raise(InterruptFlags::JOYPAD);
            } else {
                self.joypad.release(joypad_key);
            }
        }

        if self.window.is_key_down(Key::Escape) {
            return false;
        }

        self.window.is_open()
    }

    pub fn in_dma_transfer(&self) -> bool {
        self.dma_transfer.is_some()
    }

    pub fn start_dma_transfer(&mut self, start: u8) {
        self.dma_transfer = Some(DmaTransfer::new(start));
    }

    fn tick_dma_transfer(&mut self, cycles: u32) {
        if let Some(dma_transfer) = self.dma_transfer {
            let mut it = dma_transfer.take(cycles as usize);
            for dma_transfer in &mut it {
                let val = self.read8(dma_transfer.addr);
                self.ppu.write8_oam_transfer(dma_transfer.pos, val); // ignore r/w blocks
            }
            self.dma_transfer = it.next();
        }
    }

    fn step(&mut self, cpu: &mut Cpu) -> u32 {
        let system_cycles = cpu.step(self);
        let master_cycles = system_cycles as u32 * MASTER_SYSTEM_CLOCK_RATIO as u32;
        self.timer.tick(&mut self.inter, master_cycles);
        self.tick_dma_transfer(master_cycles);
        self.ppu.render(&mut self.inter, master_cycles);

        master_cycles
    }

    pub fn main_loop(&mut self) {
        let mut cpu = Cpu::new();

        while self.handle_input() {
            let mut cycles = 0;
            while cycles < FRAME_CYCLES {
                cycles += self.step(&mut cpu);
            }
            self.draw();
        }
    }
}
