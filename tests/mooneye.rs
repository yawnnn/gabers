use std::{cell::RefCell, env, io, path::PathBuf, rc::Rc};

use gabers::Gameboy;

const MAX_CYCLES: u64 = 100_000_000; // ~24 seconds of emulation

#[derive(Default)]
struct TestIo {
    res: Vec<u8>,
    curr_byte: u8,
    bits_recvd: u8,
}

impl TestIo {
    fn success(&self) -> Option<bool> {
        match self.res.as_slice() {
            [3, 5, 8, 13, 21, 34] => Some(true),
            [42, 42, 42, 42, 42, 42] => Some(false),
            _ => None,
        }
    }
}

impl io::Read for TestIo {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match buf.get_mut(0) {
            Some(b) => {
                *b = 0xFF; // value when cable is disconnected 
                Ok(1)
            }
            _ => Err(io::Error::other("Empty buffer")),
        }
    }
}

impl io::Write for TestIo {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let Some(byte) = buf.first() else {
            return Err(io::Error::other("Expected 1 bit"));
        };

        self.curr_byte = (self.curr_byte << 1) | (byte & 1);
        self.bits_recvd += 1;
        if self.bits_recvd == 8 {
            self.res.push(self.curr_byte);
            self.curr_byte = 0;
            self.bits_recvd = 0;
        }

        Ok(1)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn run_mooneye_test(rom: &str) {
    env_logger::init();
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let rom_path = root.join("mooneye-test-suite").join(rom);
    let serial_io = Rc::new(RefCell::new(TestIo::default()));
    let mut gb = Gameboy::new_headless(&rom_path, None, serial_io.clone());

    let mut cycles: u64 = 0;
    while serial_io.borrow().success().is_none() {
        cycles += gb.step() as u64;
        if cycles > MAX_CYCLES {
            panic!("Test timed out after {MAX_CYCLES} cycles");
        }
    }

    assert!(serial_io.borrow().success().unwrap());
}

macro_rules! mooneye_test {
    ($name:ident, $path:literal) => {
        #[test]
        #[allow(non_snake_case)]
        fn $name() {
            run_mooneye_test($path);
        }
    };
}

// ============================================================================
// acceptance/ - Main DMG acceptance tests
// ============================================================================

mooneye_test!(acceptance_add_sp_e_timing, "acceptance/add_sp_e_timing.gb");
mooneye_test!(
    acceptance_boot_div_dmgABCmgb,
    "acceptance/boot_div-dmgABCmgb.gb"
);
mooneye_test!(
    acceptance_boot_hwio_dmgABCmgb,
    "acceptance/boot_hwio-dmgABCmgb.gb"
);
mooneye_test!(
    acceptance_boot_regs_dmgABC,
    "acceptance/boot_regs-dmgABC.gb"
);
mooneye_test!(acceptance_call_cc_timing, "acceptance/call_cc_timing.gb");
mooneye_test!(acceptance_call_cc_timing2, "acceptance/call_cc_timing2.gb");
mooneye_test!(acceptance_call_timing, "acceptance/call_timing.gb");
mooneye_test!(acceptance_call_timing2, "acceptance/call_timing2.gb");
mooneye_test!(acceptance_div_timing, "acceptance/div_timing.gb");
mooneye_test!(acceptance_ei_sequence, "acceptance/ei_sequence.gb");
mooneye_test!(acceptance_ei_timing, "acceptance/ei_timing.gb");
mooneye_test!(acceptance_halt_ime0_ei, "acceptance/halt_ime0_ei.gb");
mooneye_test!(
    acceptance_halt_ime0_nointr_timing,
    "acceptance/halt_ime0_nointr_timing.gb"
);
mooneye_test!(
    acceptance_halt_ime1_timing,
    "acceptance/halt_ime1_timing.gb"
);
mooneye_test!(acceptance_if_ie_registers, "acceptance/if_ie_registers.gb");
mooneye_test!(acceptance_intr_timing, "acceptance/intr_timing.gb");
mooneye_test!(acceptance_jp_cc_timing, "acceptance/jp_cc_timing.gb");
mooneye_test!(acceptance_jp_timing, "acceptance/jp_timing.gb");
mooneye_test!(
    acceptance_ld_hl_sp_e_timing,
    "acceptance/ld_hl_sp_e_timing.gb"
);
mooneye_test!(acceptance_oam_dma_restart, "acceptance/oam_dma_restart.gb");
mooneye_test!(acceptance_oam_dma_start, "acceptance/oam_dma_start.gb");
mooneye_test!(acceptance_oam_dma_timing, "acceptance/oam_dma_timing.gb");
mooneye_test!(acceptance_pop_timing, "acceptance/pop_timing.gb");
mooneye_test!(acceptance_push_timing, "acceptance/push_timing.gb");
mooneye_test!(acceptance_rapid_di_ei, "acceptance/rapid_di_ei.gb");
mooneye_test!(acceptance_ret_cc_timing, "acceptance/ret_cc_timing.gb");
mooneye_test!(acceptance_ret_timing, "acceptance/ret_timing.gb");
mooneye_test!(
    acceptance_reti_intr_timing,
    "acceptance/reti_intr_timing.gb"
);
mooneye_test!(acceptance_reti_timing, "acceptance/reti_timing.gb");
mooneye_test!(acceptance_rst_timing, "acceptance/rst_timing.gb");

// acceptance/bits/
mooneye_test!(acceptance_bits_mem_oam, "acceptance/bits/mem_oam.gb");
mooneye_test!(acceptance_bits_reg_f, "acceptance/bits/reg_f.gb");

// acceptance/instr/
mooneye_test!(acceptance_instr_daa, "acceptance/instr/daa.gb");

// acceptance/interrupts/
mooneye_test!(
    acceptance_interrupts_ie_push,
    "acceptance/interrupts/ie_push.gb"
);

// acceptance/oam_dma/
mooneye_test!(acceptance_oam_dma_basic, "acceptance/oam_dma/basic.gb");
mooneye_test!(
    acceptance_oam_dma_reg_read,
    "acceptance/oam_dma/reg_read.gb"
);

// acceptance/ppu/ (DMG-compatible only, excluding -GS suffix)
mooneye_test!(
    acceptance_ppu_intr_2_0_timing,
    "acceptance/ppu/intr_2_0_timing.gb"
);
mooneye_test!(
    acceptance_ppu_intr_2_mode0_timing,
    "acceptance/ppu/intr_2_mode0_timing.gb"
);
mooneye_test!(
    acceptance_ppu_intr_2_mode0_timing_sprites,
    "acceptance/ppu/intr_2_mode0_timing_sprites.gb"
);
mooneye_test!(
    acceptance_ppu_intr_2_mode3_timing,
    "acceptance/ppu/intr_2_mode3_timing.gb"
);
mooneye_test!(
    acceptance_ppu_intr_2_oam_ok_timing,
    "acceptance/ppu/intr_2_oam_ok_timing.gb"
);
mooneye_test!(
    acceptance_ppu_stat_irq_blocking,
    "acceptance/ppu/stat_irq_blocking.gb"
);
mooneye_test!(
    acceptance_ppu_stat_lyc_onoff,
    "acceptance/ppu/stat_lyc_onoff.gb"
);

// acceptance/timer/
mooneye_test!(acceptance_timer_div_write, "acceptance/timer/div_write.gb");
mooneye_test!(
    acceptance_timer_rapid_toggle,
    "acceptance/timer/rapid_toggle.gb"
);
mooneye_test!(acceptance_timer_tim00, "acceptance/timer/tim00.gb");
mooneye_test!(
    acceptance_timer_tim00_div_trigger,
    "acceptance/timer/tim00_div_trigger.gb"
);
mooneye_test!(acceptance_timer_tim01, "acceptance/timer/tim01.gb");
mooneye_test!(
    acceptance_timer_tim01_div_trigger,
    "acceptance/timer/tim01_div_trigger.gb"
);
mooneye_test!(acceptance_timer_tim10, "acceptance/timer/tim10.gb");
mooneye_test!(
    acceptance_timer_tim10_div_trigger,
    "acceptance/timer/tim10_div_trigger.gb"
);
mooneye_test!(acceptance_timer_tim11, "acceptance/timer/tim11.gb");
mooneye_test!(
    acceptance_timer_tim11_div_trigger,
    "acceptance/timer/tim11_div_trigger.gb"
);
mooneye_test!(
    acceptance_timer_tima_reload,
    "acceptance/timer/tima_reload.gb"
);
mooneye_test!(
    acceptance_timer_tima_write_reloading,
    "acceptance/timer/tima_write_reloading.gb"
);
mooneye_test!(
    acceptance_timer_tma_write_reloading,
    "acceptance/timer/tma_write_reloading.gb"
);

// // ============================================================================
// // emulator-only/mbc1/ - MBC1 cartridge tests
// // ============================================================================

// mooneye_test!(mbc1_bits_bank1, "emulator-only/mbc1/bits_bank1.gb");
// mooneye_test!(mbc1_bits_bank2, "emulator-only/mbc1/bits_bank2.gb");
// mooneye_test!(mbc1_bits_mode, "emulator-only/mbc1/bits_mode.gb");
// mooneye_test!(mbc1_bits_ramg, "emulator-only/mbc1/bits_ramg.gb");
// mooneye_test!(mbc1_multicart_rom_8Mb, "emulator-only/mbc1/multicart_rom_8Mb.gb");
// mooneye_test!(mbc1_ram_256kb, "emulator-only/mbc1/ram_256kb.gb");
// mooneye_test!(mbc1_ram_64kb, "emulator-only/mbc1/ram_64kb.gb");
// mooneye_test!(mbc1_rom_16Mb, "emulator-only/mbc1/rom_16Mb.gb");
// mooneye_test!(mbc1_rom_1Mb, "emulator-only/mbc1/rom_1Mb.gb");
// mooneye_test!(mbc1_rom_2Mb, "emulator-only/mbc1/rom_2Mb.gb");
// mooneye_test!(mbc1_rom_4Mb, "emulator-only/mbc1/rom_4Mb.gb");
// mooneye_test!(mbc1_rom_512kb, "emulator-only/mbc1/rom_512kb.gb");
// mooneye_test!(mbc1_rom_8Mb, "emulator-only/mbc1/rom_8Mb.gb");

// // ============================================================================
// // emulator-only/mbc2/ - MBC2 cartridge tests
// // ============================================================================

// mooneye_test!(mbc2_bits_ramg, "emulator-only/mbc2/bits_ramg.gb");
// mooneye_test!(mbc2_bits_romb, "emulator-only/mbc2/bits_romb.gb");
// mooneye_test!(mbc2_bits_unused, "emulator-only/mbc2/bits_unused.gb");
// mooneye_test!(mbc2_ram, "emulator-only/mbc2/ram.gb");
// mooneye_test!(mbc2_rom_1Mb, "emulator-only/mbc2/rom_1Mb.gb");
// mooneye_test!(mbc2_rom_2Mb, "emulator-only/mbc2/rom_2Mb.gb");
// mooneye_test!(mbc2_rom_512kb, "emulator-only/mbc2/rom_512kb.gb");

// // ============================================================================
// // emulator-only/mbc5/ - MBC5 cartridge tests
// // ============================================================================

// mooneye_test!(mbc5_rom_16Mb, "emulator-only/mbc5/rom_16Mb.gb");
// mooneye_test!(mbc5_rom_1Mb, "emulator-only/mbc5/rom_1Mb.gb");
// mooneye_test!(mbc5_rom_2Mb, "emulator-only/mbc5/rom_2Mb.gb");
// mooneye_test!(mbc5_rom_32Mb, "emulator-only/mbc5/rom_32Mb.gb");
// mooneye_test!(mbc5_rom_4Mb, "emulator-only/mbc5/rom_4Mb.gb");
// mooneye_test!(mbc5_rom_512kb, "emulator-only/mbc5/rom_512kb.gb");
// mooneye_test!(mbc5_rom_64Mb, "emulator-only/mbc5/rom_64Mb.gb");
// mooneye_test!(mbc5_rom_8Mb, "emulator-only/mbc5/rom_8Mb.gb");
