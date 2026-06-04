use std::path::Path;

use crate::cartridge::Cartridge;
use crate::constants::*;
use crate::cpu::Cpu;
use crate::gpu::Gpu;
use crate::interrupt::Interrupt;
use crate::joypad::{Joypad, JoypadKey};
use crate::serial::Serial;
use crate::timer::Timer;

const TARGET_FPS: usize = 60;
const WINDOW_SCALE: usize = 1;
const WINDOW_RES: (usize, usize) = (RESOLUTION.0 * WINDOW_SCALE, RESOLUTION.1 * WINDOW_SCALE);
const TARGET_CYCLES: u32 = (MASTER_CLOCK as f64 / TARGET_FPS as f64).ceil() as u32;

pub struct Gameboy {
    pub cartridge: Cartridge,
    pub cpu: Cpu,
    pub gpu: Gpu,
    pub inter_enable: Interrupt,
    pub inter_flag: Interrupt,
    pub joypad: Joypad,
    pub _serial: Serial,
    pub timer: Timer,

    window: minifb::Window,
    window_buf: Vec<u32>,
}

impl Gameboy {
    pub fn new(cartridge_path: impl AsRef<Path>) -> Box<Self> {
        let cartridge = Cartridge::new(cartridge_path.as_ref());

        let mut window = minifb::Window::new(
            &cartridge.title,
            WINDOW_RES.0,
            WINDOW_RES.1,
            minifb::WindowOptions::default(),
        )
        .unwrap();
        window.set_target_fps(TARGET_FPS);
        let window_buf = vec![0; WINDOW_RES.0 * WINDOW_RES.1];

        let mut gb = Box::new(Gameboy {
            cartridge,
            cpu: Cpu::new(),
            gpu: Gpu::new(),
            inter_enable: Interrupt::new(),
            inter_flag: Interrupt::new(),
            joypad: Joypad::new(),
            _serial: Serial,
            timer: Timer::new(),
            window,
            window_buf,
        });
        let ptr = gb.as_mut() as *mut Gameboy;
        gb.cpu.set_gb(ptr);

        gb
    }

    fn update_window(&mut self) {
        let window = &mut self.window;
        let window_buf = &mut self.window_buf;
        let gpu_buf = self.gpu.canvas;

        let (chunks, _) = gpu_buf.as_chunks::<4>();
        for (i, pixel) in chunks.iter().enumerate() {
            if i >= window_buf.len() {
                break; // TODO: viewport
            }
            window_buf[i] = u32::from_le_bytes(*pixel);
        }
        window
            .update_with_buffer(window_buf, WINDOW_RES.0, WINDOW_RES.1)
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
