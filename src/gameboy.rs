use std::path::Path;

use crate::cartridge::Cartridge;
use crate::constants::*;
use crate::cpu::Cpu;
use crate::gpu::Gpu;
use crate::interrupt::Interrupt;
use crate::joypad::{Joypad, JoypadKey};
use crate::timer::Timer;

const TARGET_FPS: usize = 60;
const TARGET_CYCLES: u32 = (MASTER_CLOCK as f64 / TARGET_FPS as f64).ceil() as u32;

pub struct Gameboy {
    pub cartridge: Cartridge,
    pub cpu: Cpu,
    pub gpu: Gpu,
    pub inter_enable: Interrupt,
    pub inter_flag: Interrupt,
    pub joypad: Joypad,
    pub timer: Timer,

    window: minifb::Window,
    window_buf: Vec<u32>,
}

impl Gameboy {
    pub fn new(cartridge_path: impl AsRef<Path>) -> Box<Self> {
        let cartridge = Cartridge::new(cartridge_path.as_ref());

        let mut window = minifb::Window::new(
            &cartridge.title,
            SCREEN_W,
            SCREEN_H,
            minifb::WindowOptions::default(),
        )
        .unwrap();
        window.set_target_fps(TARGET_FPS);

        let mut gb = Box::new(Gameboy {
            cartridge,
            cpu: Cpu::new(),
            gpu: Gpu::new(),
            inter_enable: Interrupt::new(),
            inter_flag: Interrupt::new(),
            joypad: Joypad::new(),
            timer: Timer::new(),
            window,
            window_buf: vec![0; SCREEN_W * SCREEN_H],
        });
        let ptr = gb.as_mut() as *mut Gameboy;
        gb.cpu.set_gb(ptr);

        gb
    }

    fn update_window(&mut self) {
        self.gpu.draw();

        for (i, rgb) in self.gpu.buf.iter().enumerate() {
            let [r, g, b] = *rgb;
            self.window_buf[i] = u32::from_le_bytes([r, g, b, 0xFF]);
        }
        self.window
            .update_with_buffer(&self.window_buf, SCREEN_W, SCREEN_H)
            .unwrap();
    }

    fn handle_input(&mut self) -> bool {
        let joypad_keys = [
            (minifb::Key::Right, JoypadKey::Right),
            (minifb::Key::Up, JoypadKey::Up),
            (minifb::Key::Left, JoypadKey::Left),
            (minifb::Key::Down, JoypadKey::Down),
            (minifb::Key::Z, JoypadKey::A),
            (minifb::Key::X, JoypadKey::B),
            (minifb::Key::Space, JoypadKey::Select),
            (minifb::Key::Enter, JoypadKey::Start),
        ];
        for (key, joypad_key) in joypad_keys {
            if self.window.is_key_down(key) {
                self.joypad.press(joypad_key);
            } else {
                self.joypad.release(joypad_key);
            }
        }

        if self.window.is_key_down(minifb::Key::Escape) {
            return false;
        }

        self.window.is_open()
    }

    pub fn main_loop(&mut self) {
        while self.handle_input() {
            let mut ncycles = 0;
            while ncycles < TARGET_CYCLES {
                ncycles += self.cpu.step() as u32 * MASTER_SYSTEM_CLOCK_RATIO as u32;
            }
            self.update_window();
        }
    }
}
