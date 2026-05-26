#![feature(const_trait_impl)]

mod cartridge;
mod common;
mod constants;
mod cpu;
mod decode;
mod gameboy;
mod gpu;
mod instructions;
mod interrupt;
mod joypad;
mod mmu;
mod registers;
mod serial;
mod timer;

use std::env;

use crate::gameboy::Gameboy;

fn main() {
    env_logger::init();
    let args = env::args().skip(1).collect::<Vec<_>>();
    let mut gb = Gameboy::new(args.first().expect("Missing path to ROM file"));
    gb.main_loop();
}
