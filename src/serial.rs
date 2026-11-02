use std::{
    io,
    ops::{Deref, DerefMut},
};

use crate::{
    gameboy::{SYSTEM_CLOCK, TCycle},
    interrupt::{InterruptFlags, Interrupts},
};

struct SerialControl(u8);

#[rustfmt::skip]
impl SerialControl {
    const IS_MASTER: u8   = 0b0000_0001;
    const UNUSED: u8      = 0b0111_1110;
    const IN_PROGRESS: u8 = 0b1000_0000;
}

impl Deref for SerialControl {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SerialControl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

const SERIAL_CLOCK: usize = 8192;
const SYSTEM_SERIAL_RATIO: usize = SYSTEM_CLOCK / SERIAL_CLOCK;

pub trait Rw: io::Read + io::Write {
    fn send(&mut self, bit: u8) {
        let _ = self.write(&[bit]);
    }

    fn recv(&mut self) -> u8 {
        let mut buf = [0; 1];
        match self.read(&mut buf) {
            Ok(1) => buf[0],
            _ => 0,
        }
    }
}

impl<T: io::Read + io::Write> Rw for T {}

pub struct NullIo;

impl io::Read for NullIo {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Ok(0xFF) // value when cable is disconnected
    }
}

impl io::Write for NullIo {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct Serial {
    data: u8,               // SB: Serial transfer data
    control: SerialControl, // SC: Serial transfer control
    progress: TCycle,
    io: Box<dyn Rw>,
}

impl Serial {
    pub fn new(io: Box<dyn Rw>) -> Self {
        Serial {
            data: 0,
            control: SerialControl(0),
            progress: 0,
            io,
        }
    }

    fn in_progress(&self) -> bool {
        *self.control & SerialControl::IN_PROGRESS != 0
    }

    #[allow(unused)]
    fn is_master(&self) -> bool {
        *self.control & SerialControl::IS_MASTER != 0
    }

    fn bits_sent(&self) -> TCycle {
        self.progress / SYSTEM_SERIAL_RATIO as TCycle
    }

    pub fn tick(&mut self, inter: &mut Interrupts, cycles: TCycle) {
        if !self.in_progress() {
            return;
        }

        let prev = self.bits_sent();
        self.progress += cycles;
        for _ in 0..(self.bits_sent() - prev) {
            self.io.send(self.data >> 7);
            self.data = (self.data << 1) | self.io.recv()
        }

        if self.bits_sent() >= 8 {
            self.progress = 0;
            *self.control &= !SerialControl::IN_PROGRESS;
            inter.raise(InterruptFlags::SERIAL);
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0xFF01 => self.data,
            0xFF02 => *self.control | SerialControl::UNUSED,
            _ => 0xFF,
        }
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF01 => self.data = val,
            0xFF02 => *self.control = val,
            _ => (),
        }
    }
}
