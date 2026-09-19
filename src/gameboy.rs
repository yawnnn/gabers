use minifb::{Key, Window, WindowOptions};
use std::ops::DerefMut;
use std::{path::Path, pin::Pin};

use crate::cartridge::Cartridge;
use crate::common::*;
use crate::cpu::Cpu;
use crate::interrupt::Interrupt;
use crate::joypad::{Joypad, JoypadKey};
use crate::mmu::*;
use crate::ppu::Ppu;
use crate::timer::Timer;

pub const MASTER_CLOCK: usize = 4_194_304;
pub const MASTER_SYSTEM_CLOCK_RATIO: usize = 4;
pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;
const TARGET_FPS: usize = 60;
const FRAME_CYCLES: u32 = (MASTER_CLOCK as f64 / TARGET_FPS as f64).ceil() as u32;

pub struct Gameboy {
    pub cartridge: Cartridge,
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub int_enable: u8,
    pub int_flag: Interrupt,
    pub joypad: Joypad,
    pub timer: Timer,
    pub wram: Box<[u8; wram_range!().span()]>,
    pub hram: Box<[u8; hram_range!().span()]>,

    window: Window,
    screen: Box<[u32; SCREEN_W * SCREEN_H]>,
}

impl Gameboy {
    pub fn new(cartridge_path: impl AsRef<Path>) -> Pin<Box<Self>> {
        let cartridge = Cartridge::new(cartridge_path.as_ref());

        let mut window = Window::new(
            &cartridge.title,
            SCREEN_W,
            SCREEN_H,
            WindowOptions::default(),
        )
        .unwrap();
        window.set_target_fps(TARGET_FPS);

        let mut gb = Box::into_pin(Box::new(Gameboy {
            cartridge,
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            int_enable: 0,
            int_flag: Interrupt::new(),
            joypad: Joypad::new(),
            timer: Timer::new(),
            wram: boxed_1d(),
            hram: boxed_1d(),
            window,
            screen: boxed_1d(),
        }));
        gb.cpu.gb = gb.deref_mut() as *mut Gameboy;
        gb.ppu.gb = gb.deref_mut() as *mut Gameboy;
        gb.timer.gb = gb.deref_mut() as *mut Gameboy;
        gb.joypad.gb = gb.deref_mut() as *mut Gameboy;

        gb
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
                self.joypad.press(joypad_key);
                self.int_flag.raise(Interrupt::JOYPAD);
            } else {
                self.joypad.release(joypad_key);
            }
        }

        if self.window.is_key_down(Key::Escape) {
            return false;
        }

        self.window.is_open()
    }

    fn step(&mut self) -> u32 {
        let system_cycles = self.cpu.step();
        let master_cycles = system_cycles as u32 * MASTER_SYSTEM_CLOCK_RATIO as u32;
        self.timer.tick(master_cycles);
        self.ppu.render(master_cycles);

        master_cycles
    }

    pub fn main_loop(&mut self) {
        while self.handle_input() {
            let mut cycles = 0;
            while cycles < FRAME_CYCLES {
                cycles += self.step();
            }
            self.draw();
        }
    }
}
