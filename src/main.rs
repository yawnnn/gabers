mod codegen;
mod common;
mod cpu;
mod gpu;
mod instructions;
mod memory;
mod registers;

use cpu::Cpu;
use memory::*;

use minifb::{Key, Window, WindowOptions};

use std::thread::sleep;
use std::time::{Duration, Instant};

const SCALING_FACTOR: usize = 1;
const WINDOW_WIDTH: usize = SCREEN_WIDTH * SCALING_FACTOR;
const WINDOW_HEIGHT: usize = SCREEN_HEIGHT * SCALING_FACTOR;

fn main() {
    let window = Window::new(
        "DMG-01",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    run(window)
}

const ONE_SECOND_IN_MICROS: usize = 1_000_000_000;
const ONE_SECOND_IN_CYCLES: usize = 4_190_000;
const ONE_FRAME_IN_CYCLES: usize = 70_224;
const NUMBER_OF_PIXELS: usize = 23_040;

fn run(mut window: Window) {
    let mut cpu = Box::<Cpu>::default();
    let mut buffer = vec![0; NUMBER_OF_PIXELS];
    let mut cycles_elapsed_in_frame = 0;
    let mut now = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let time_delta = now.elapsed().subsec_nanos();
        now = Instant::now();
        let delta = time_delta as f64 / ONE_SECOND_IN_MICROS as f64;
        let cycles_to_run = delta * ONE_SECOND_IN_CYCLES as f64;

        let mut cycles_elapsed = 0;
        while cycles_elapsed <= cycles_to_run as usize {
            cycles_elapsed += cpu.step();
        }
        cycles_elapsed_in_frame += cycles_elapsed;

        if cycles_elapsed_in_frame >= ONE_FRAME_IN_CYCLES {
            for (i, pixel) in cpu.ram.gpu.canvas.chunks(4).enumerate() {
                buffer[i] = ((pixel[3] as u32) << 24)
                    | ((pixel[2] as u32) << 16)
                    | ((pixel[1] as u32) << 8)
                    | (pixel[0] as u32)
            }
            window
                .update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
                .unwrap();
            cycles_elapsed_in_frame = 0;
        } else {
            sleep(Duration::from_nanos(2))
        }
    }
}
