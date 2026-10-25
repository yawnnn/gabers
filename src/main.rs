#![feature(const_trait_impl)]

mod cartridge;
mod common;
mod cpu;
mod decode;
mod gameboy;
mod instructions;
mod interrupt;
mod joypad;
mod mmu;
mod ppu;
mod registers;
mod timer;

use std::env;

use crate::gameboy::Gameboy;

fn main() {
    env_logger::init();
    let mut args = env::args().skip(1);
    let rom = args.next().expect("Missing path to ROM file");
    let bootrom = args.next();
    let bootrom = bootrom.as_deref();
    let mut gb = Gameboy::new(rom, bootrom);
    gb.main_loop();
}
