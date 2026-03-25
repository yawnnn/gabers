#![feature(const_trait_impl)]

mod codegen;
mod common;
mod cpu;
mod gpu;
mod instructions;
mod memory;
mod registers;
mod decode;

use cpu::Cpu;
use memory::*;

use minifb::{Key, Window, WindowOptions};

use std::thread::sleep;
use std::time::{Duration, Instant};

const SCALING_FACTOR: usize = 1;
const WINDOW: (usize, usize) = (SCREEN.0 * SCALING_FACTOR, SCREEN.1 * SCALING_FACTOR);
const MICROS_PER_SECOND: usize = 1_000_000_000;
const CYCLES_PER_SECOND: usize = 4_190_000;
const CYCLES_PER_FRAME: usize = 70_224;

fn main() {
    let mut window = Window::new("Gameboy", WINDOW.0, WINDOW.1, WindowOptions::default()).unwrap();
    let mut cpu = Box::<Cpu>::default();
    let mut buffer = vec![0; WINDOW.0 * WINDOW.1];
    let mut elapsed_cycles_in_frame = 0;
    let mut now = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let time_delta = now.elapsed().subsec_nanos();
        now = Instant::now();
        let delta = time_delta as f32 / MICROS_PER_SECOND as f32;
        let cycles_to_run = delta * CYCLES_PER_SECOND as f32;

        let mut elapsed_cycles = 0;
        while elapsed_cycles <= cycles_to_run.ceil() as usize {
            elapsed_cycles += cpu.step();
        }
        elapsed_cycles_in_frame += elapsed_cycles;

        if elapsed_cycles_in_frame >= CYCLES_PER_FRAME {
            for (i, pixel) in cpu.bus.gpu.canvas.chunks(4).enumerate() {
                buffer[i] = ((pixel[3] as u32) << 24)
                    | ((pixel[2] as u32) << 16)
                    | ((pixel[1] as u32) << 8)
                    | (pixel[0] as u32)
            }
            window
                .update_with_buffer(&buffer, WINDOW.0, WINDOW.1)
                .unwrap();
            elapsed_cycles_in_frame = 0;
        } else {
            sleep(Duration::from_nanos(2))
        }
    }
}
