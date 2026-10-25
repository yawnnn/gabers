use anyhow::{Context, anyhow};
use log::debug;
use std::ops::Range;
use std::{fs, path::Path};

use crate::common::*;
use crate::mmu::{eram_range, rom_range};

const HEADER: Range<usize> = 0x0100..0x014F + 1;
const NINTENDO_LOGO_RANGE: Range<usize> = 0x0104..0x0134;
const ROM_TITLE_DMG: Range<usize> = 0x0134..0x0143 + 1;
const CARTRIDGE_TYPE: usize = 0x0147;
const ROM_SIZE: usize = 0x0148;
const RAM_SIZE: usize = 0x0149;
const HEADER_CHECKSUM_POS: usize = 0x014D;
const HEADER_CHECKSUM_RANGE: Range<usize> = 0x0134..0x014C + 1;

const ROM_START: usize = *rom_range!().start();
const RAM_START: usize = *eram_range!().start();

const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

const CARTRIDGE_TYPES: ConstMap<u8, &str, 31> = ConstMap::new([
    (0x00, "ROM ONLY"),
    (0x01, "MBC1"),
    (0x02, "MBC1+RAM"),
    (0x03, "MBC1+RAM+BATTERY"),
    (0x05, "MBC2"),
    (0x06, "MBC2+BATTERY"),
    (0x08, "ROM+RAM"),
    (0x09, "ROM+RAM+BATTERY"),
    (0x0b, "MMM01"),
    (0x0c, "MMM01+RAM"),
    (0x0d, "MMM01+RAM+BATTERY"),
    (0x0f, "MBC3+TIMER+BATTERY"),
    (0x10, "MBC3+TIMER+RAM+BATTERY"),
    (0x11, "MBC3"),
    (0x12, "MBC3+RAM"),
    (0x13, "MBC3+RAM+BATTERY"),
    (0x15, "MBC4"),
    (0x16, "MBC4+RAM"),
    (0x17, "MBC4+RAM+BATTERY"),
    (0x19, "MBC5"),
    (0x1a, "MBC5+RAM"),
    (0x1b, "MBC5+RAM+BATTERY"),
    (0x1c, "MBC5+RUMBLE"),
    (0x1d, "MBC5+RUMBLE+RAM"),
    (0x1e, "MBC5+RUMBLE+RAM+BATTERY"),
    (0x20, "MBC6"),
    (0x22, "MBC7+SENSOR+RUMBLE+RAM+BATTERY"),
    (0xfc, "POCKET CAMERA"),
    (0xfd, "BANDAI TAMA5"),
    (0xfe, "HuC3"),
    (0xff, "HuC1+RAM+BATTERY"),
]);

const ROM_SIZES: ConstMap<u8, usize, 12> = ConstMap::new([
    (0x00, 2),
    (0x01, 4),
    (0x02, 8),
    (0x03, 16),
    (0x04, 32),
    (0x05, 64),
    (0x06, 128),
    (0x07, 256),
    (0x08, 512),
    (0x52, 72),
    (0x53, 80),
    (0x54, 96),
]);

const ROM_BANK_SIZE: usize = 0x4000;

const RAM_SIZES: ConstMap<u8, usize, 6> = ConstMap::new([
    (0x00, 0),
    (0x01, 0),
    (0x02, 8),
    (0x03, 32),
    (0x04, 128),
    (0x05, 64),
]);

const BOOTROMS: ConstMap<&'static str, &'static [u8], 3> = ConstMap::new([
    ("dmg0", include_bytes!("../bootroms/bin/dmg0.bin")),
    ("dmg", include_bytes!("../bootroms/bin/dmg.bin")),
    ("mgb", include_bytes!("../bootroms/bin/mgb.bin")),
]);

pub struct Cartridge {
    pub title: String,
    rom: Box<[u8]>,
    ram: Box<[u8]>,
    bootrom: Option<&'static [u8]>,
}

impl Cartridge {
    pub fn new(path: &Path, bootrom: Option<&str>) -> anyhow::Result<Self> {
        let rom = fs::read(path)?.into_boxed_slice();
        if rom.len() < HEADER.end {
            return Err(anyhow!("Invalid rom length"));
        }
        if rom[NINTENDO_LOGO_RANGE] != NINTENDO_LOGO {
            return Err(anyhow!("Invalid Nintendo logo"));
        }
        let title = String::from_utf8(
            rom[ROM_TITLE_DMG]
                .iter()
                .take_while(|&b| *b != 0)
                .copied()
                .collect(),
        )?;
        debug!("Cartridge title {title}");
        let cartridge_type = CARTRIDGE_TYPES
            .get(rom[CARTRIDGE_TYPE])
            .context("Invalid cartridge type")?;
        debug!("Cartridge type {cartridge_type}");
        let _ = ROM_SIZES
            .get(rom[ROM_SIZE])
            .map(|v| v * ROM_BANK_SIZE)
            .filter(|v| *v == rom.len())
            .context("Invalid rom size")?;
        let ram_size = if cartridge_type.contains("RAM") {
            RAM_SIZES.get(rom[RAM_SIZE]).context("Invalid RAM size")?
        } else {
            0
        };
        debug!("Ram size: {ram_size}");

        let checksum: u8 = rom[HEADER_CHECKSUM_RANGE]
            .iter()
            .fold(0, |acc, b| acc.wrapping_sub(*b).wrapping_sub(1));
        if checksum != rom[HEADER_CHECKSUM_POS] {
            return Err(anyhow!("Invalid checksum"));
        }

        let bootrom = bootrom
            .and_then(|name| BOOTROMS.get(name))
            .unwrap_or(BOOTROMS.index(0));

        Ok(Cartridge {
            title,
            rom,
            ram: Box::from(vec![0; ram_size]),
            bootrom: Some(bootrom),
        })
    }

    pub fn unmap_bootrom(&mut self) {
        self.bootrom = None;
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            rom_range!() => {
                let idx = addr as usize - ROM_START;
                self.bootrom
                    .as_ref()
                    .and_then(|bootrom| bootrom.get(idx).copied())
                    .unwrap_or(self.rom[idx])
            }
            eram_range!() => self.ram[addr as usize - RAM_START],
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            rom_range!() => (),
            eram_range!() => self.ram[addr as usize - RAM_START] = val,
            _ => (),
        }
    }
}
