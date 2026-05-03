#![feature(const_trait_impl)]

use minifb::{Key, Window, WindowOptions};

mod constants;
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
const MACHINE_TO_CPU_RATIO: u32 = 4;
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
    window.set_target_fps(TARGET_FPS);
    let mut window_buf = vec![0; WIN_SIZE.0 * WIN_SIZE.1];
    let mut cpu = Box::<Cpu>::default();
    const TARGET_CYCLES: u32 = (CPU_HZ as f64 / TARGET_FPS as f64).ceil() as u32;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut ncycles = 0;
        while ncycles < TARGET_CYCLES {
            ncycles += cpu.step() as u32 * MACHINE_TO_CPU_RATIO;
        }
        update_window(&mut window, &mut window_buf, &cpu.bus.gpu.canvas);
    }
}
