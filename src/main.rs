#![feature(const_trait_impl)]

use std::env;

use gabers::Gameboy;

fn main() {
    env_logger::init();
    let mut args = env::args().skip(1);
    let rom = args.next().expect("Missing path to ROM file");
    let bootrom = args.next();
    let bootrom = bootrom.as_deref();

    let mut gb = Gameboy::new_with_screen(rom, bootrom);
    gb.game_loop();
}
