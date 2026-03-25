#![feature(const_trait_impl)]

use minifb::{Key, Window, WindowOptions};
use std::thread;
use std::time::{Duration, Instant};

mod codegen;
mod common;
mod cpu;
mod decode;
mod gpu;
mod instructions;
mod memory;
mod registers;

use cpu::Cpu;

pub const SCREEN_SIZE: (usize, usize) = (160, 140);
const CPU_HZ: usize = 4_194_304;
const TARGET_FPS: usize = 60;
const WIN_SCALE: usize = 1;
const WIN_SIZE: (usize, usize) = (SCREEN_SIZE.0 * WIN_SCALE, SCREEN_SIZE.1 * WIN_SCALE);

fn update_window(window: &mut Window, window_buf: &mut [u32], gpu_buf: &[u8]) {
    let (chunks, _) = gpu_buf.as_chunks::<4>();
    for (i, pixel) in chunks.iter().enumerate() {
        window_buf[i] = u32::from_le_bytes(*pixel);
    }
    window
        .update_with_buffer(window_buf, WIN_SIZE.0, WIN_SIZE.1)
        .unwrap();
}

fn main() {
    let mut window =
        Window::new("Gameboy", WIN_SIZE.0, WIN_SIZE.1, WindowOptions::default()).unwrap();
    let mut window_buf = vec![0; WIN_SIZE.0 * WIN_SIZE.1];
    let mut cpu = Box::<Cpu>::default();
    let target_duration = Duration::from_secs_f64(1.0 / TARGET_FPS as f64);
    let target_cycles = (CPU_HZ as f64 / TARGET_FPS as f64).ceil() as usize;

    let mut last_frame = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut ncycles = 0;
        while ncycles < target_cycles {
            ncycles += cpu.step();
        }

        let elapsed = last_frame.elapsed();
        if elapsed < target_duration {
            thread::sleep(target_duration - elapsed);
        }

        update_window(&mut window, &mut window_buf, &cpu.bus.gpu.canvas);
        last_frame = Instant::now();
    }
}
