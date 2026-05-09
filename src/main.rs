#![feature(const_trait_impl)]

use std::path;

use minifb::{Key, Window, WindowOptions};

mod cartridge;
mod common;
mod constants;
mod cpu;
mod decode;
mod gpu;
mod instructions;
mod mmu;
mod registers;

use crate::constants::*;
use crate::cpu::Cpu;
use crate::mmu::Mmu;

const TARGET_FPS: usize = 60;
const WINDOW_SCALE: usize = 1;
const WINDOW_RES: (usize, usize) = (RESOLUTION.0 * WINDOW_SCALE, RESOLUTION.1 * WINDOW_SCALE);
const TARGET_CYCLES: u32 = (MASTER_CLOCK as f64 / TARGET_FPS as f64).ceil() as u32;

struct Gameboy {
    window: Window,
    window_buf: Vec<u32>,
    cpu: Cpu,
}

impl Gameboy {
    fn new(path: &path::Path) -> Self {
        let mmu = Mmu::new(path);
        let mut window = Window::new(
            &mmu.cartridge.title,
            WINDOW_RES.0,
            WINDOW_RES.1,
            WindowOptions::default(),
        )
        .unwrap();
        window.set_target_fps(TARGET_FPS);
        let window_buf = vec![0; WINDOW_RES.0 * WINDOW_RES.1];
        let cpu = Cpu::new(mmu);

        Gameboy {
            window,
            window_buf,
            cpu,
        }
    }

    fn update_window(&mut self) {
        let window = &mut self.window;
        let window_buf = &mut self.window_buf;
        let gpu_buf = self.cpu.mmu.gpu.canvas;

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
}

fn main() {
    env_logger::init();

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let cartridge = path::PathBuf::from(&args[0]);
    let mut gb: Gameboy = Gameboy::new(&cartridge);

    while gb.window.is_open() && !gb.window.is_key_down(Key::Escape) {
        let mut ncycles = 0;
        while ncycles < TARGET_CYCLES {
            ncycles += gb.cpu.step() as u32 * MASTER_SYSTEM_CLOCK_RATIO as u32;
        }
        gb.update_window();
    }
}
