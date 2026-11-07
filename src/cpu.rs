use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fs;
use std::path::Path;

use crate::common::*;
use crate::registers::*;
use crate::*;

pub enum Condition {
    CF,
    NoCF,
    ZF,
    NoZF,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Addr {
    BC,
    DE,
    HL,
    HLI,
    HLD,
    Imm16,
}

#[derive(Clone, Copy)]
pub struct Imm8;

#[derive(Clone, Copy)]
pub struct Imm16;

pub struct Cpu {
    pub regs: Registers,
    pub ime: bool,
    pub low_power_mode: bool,
    pub halt_bug: bool,
    pub pending_enable_ime: bool,
}

impl Cpu {
    pub fn new(has_bootrom: bool, cartridge_checksum: u8) -> Self {
        Cpu {
            regs: Registers::new(has_bootrom, cartridge_checksum),
            ime: false,
            low_power_mode: false,
            halt_bug: false,
            pending_enable_ime: false,
        }
    }

    pub fn fetch8(&mut self, pe: &mut Peripherals) -> u8 {
        let pc = self.regs.pc;
        let byte = pe.read8(pc);
        self.regs.pc = pc.wrapping_add(1);

        byte
    }

    pub fn fetch16(&mut self, pe: &mut Peripherals) -> u16 {
        let pc = self.regs.pc;
        let word = pe.read16(pc);
        self.regs.pc = pc.wrapping_add(2);

        word
    }

    pub fn get_reg8(&self, r8: Reg8) -> u8 {
        self.regs.read8(r8)
    }

    pub fn get_reg16(&self, r16: Reg16) -> u16 {
        self.regs.read16(r16)
    }

    pub fn set_reg8(&mut self, r8: Reg8, value: u8) {
        self.regs.write8(r8, value);
    }

    pub fn set_reg16(&mut self, r16: Reg16, value: u16) {
        self.regs.write16(r16, value);
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        self.regs.get_flag(flag)
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        self.regs.set_flag(flag, value);
    }

    pub fn read_addr(&mut self, pe: &mut Peripherals, addr: Addr) -> u16 {
        match addr {
            Addr::BC => self.read16(pe, Reg16::BC),
            Addr::DE => self.read16(pe, Reg16::DE),
            Addr::HL => self.read16(pe, Reg16::HL),
            Addr::HLI => {
                let addr = self.read16(pe, Reg16::HL);
                self.write16(pe, Reg16::HL, addr.wrapping_add(1));
                addr
            }
            Addr::HLD => {
                let addr = self.read16(pe, Reg16::HL);
                self.write16(pe, Reg16::HL, addr.wrapping_sub(1));
                addr
            }
            Addr::Imm16 => self.fetch16(pe),
        }
    }

    fn handle_interrupts(&mut self, pe: &mut Peripherals) -> Option<MCycle> {
        let bits = pe.inter.enabled();
        let inter = (1 << bits.trailing_zeros()) as u8; // lowest bit has priority
        if inter == 0 {
            return None;
        }
        self.low_power_mode = false; // interrupts wake from HALT regardless of IME
        if !self.ime {
            return None;
        }
        self.ime = false;
        pe.inter.lower(inter);
        let addr = match inter {
            0x01 => 0x40, // V-Blank
            0x02 => 0x48, // LCD
            0x04 => 0x50, // Timer
            0x08 => 0x58, // Serial
            0x10 => 0x60, // Joypad
            _ => unreachable!(),
        };
        self.call(pe, addr);

        Some(5) // 2 cycles of NOP + 3 cycles for call
    }

    pub fn step(&mut self, pe: &mut Peripherals) -> MCycle {
        if let Some(cycles) = self.handle_interrupts(pe) {
            return cycles;
        }

        if self.low_power_mode {
            return 1; // NOP
        }

        update_stack_trace(self, pe);
        let opcode = self.fetch8(pe);
        let cycles = self.decode_exec(pe, opcode);

        if self.pending_enable_ime {
            self.pending_enable_ime = false;
            self.ime = true;
        }

        if self.halt_bug {
            self.halt_bug = false;
            self.regs.pc = self.regs.pc.wrapping_sub(1);
        }

        cycles
    }

    // Z N H C
    // * 0 * *
    pub fn alu_add(&mut self, value: u8, cf: bool) -> u8 {
        let a = self.get_reg8(Reg8::A);
        let (res, carry) = a.carrying_add(value, cf);
        let (_, half_carry) = a.half_carrying_add(value, cf);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, half_carry);
        self.set_flag(Flag::C, carry);

        res
    }

    // Z N H C
    // * 1 * *
    pub fn alu_sub(&mut self, value: u8, cf: bool) -> u8 {
        let a = self.get_reg8(Reg8::A);
        let (res, carry) = a.borrowing_sub(value, cf);
        let (_, half_carry) = a.half_borrowing_sub(value, cf);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::H, half_carry);
        self.set_flag(Flag::C, carry);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rl(&mut self, value: u8) -> u8 {
        let cf = self.get_flag(Flag::C);
        let res = (value << 1) | cf as u8;
        let new_cf = value & 0x80;

        self.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rlc(&mut self, value: u8) -> u8 {
        let res = value.rotate_left(1);
        let new_cf = value & 0x80;

        self.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rr(&mut self, value: u8) -> u8 {
        let cf = self.get_flag(Flag::C);
        let res = (value >> 1) | ((cf as u8) << 7);
        let new_cf = value & 0x01;

        self.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_rrc(&mut self, value: u8) -> u8 {
        let res = value.rotate_right(1);
        let new_cf = value & 0x01;

        self.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_sla(&mut self, value: u8) -> u8 {
        let res = value << 1;
        let new_cf = value & 0x80;

        self.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_sra(&mut self, value: u8) -> u8 {
        let res = (value >> 1) | (value & 0x80);
        let new_cf = value & 0x01;

        self.set_flag(Flag::C, new_cf != 0);

        res
    }

    // Z N H C
    // - - - *
    pub fn alu_srl(&mut self, value: u8) -> u8 {
        let res = value >> 1;
        let new_cf = value & 0x01;

        self.set_flag(Flag::C, new_cf != 0);

        res
    }

    pub fn stack_push(&mut self, pe: &mut Peripherals, value: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        pe.write16(self.regs.sp, value);
    }

    pub fn stack_pop(&mut self, pe: &mut Peripherals) -> u16 {
        let res = pe.read16(self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(2);
        res
    }

    pub fn check_condition(&self, cond: Condition) -> bool {
        match cond {
            Condition::CF => self.get_flag(Flag::C),
            Condition::NoCF => !self.get_flag(Flag::C),
            Condition::ZF => self.get_flag(Flag::Z),
            Condition::NoZF => !self.get_flag(Flag::Z),
        }
    }

    pub fn jump_rel(&mut self, offset: i8) {
        self.regs.pc = self.regs.pc.wrapping_add_signed(offset as i16);
    }

    pub fn jump_abs(&mut self, addr: u16) {
        self.regs.pc = addr;
    }

    pub fn call(&mut self, pe: &mut Peripherals, addr: u16) {
        self.stack_push(pe, self.regs.pc);
        self.jump_abs(addr);
    }
}

pub trait In8<T> {
    fn read8(&mut self, pe: &mut Peripherals, src: T) -> u8;
}

pub trait Out8<T> {
    fn write8(&mut self, pe: &mut Peripherals, dst: T, value: u8);
}

pub trait In16<T> {
    fn read16(&mut self, pe: &mut Peripherals, src: T) -> u16;
}

pub trait Out16<T> {
    fn write16(&mut self, pe: &mut Peripherals, dst: T, value: u16);
}

impl In8<Reg8> for Cpu {
    fn read8(&mut self, _: &mut Peripherals, src: Reg8) -> u8 {
        self.get_reg8(src)
    }
}

impl Out8<Reg8> for Cpu {
    fn write8(&mut self, _: &mut Peripherals, dst: Reg8, value: u8) {
        self.set_reg8(dst, value);
    }
}

impl In16<Reg16> for Cpu {
    fn read16(&mut self, _: &mut Peripherals, src: Reg16) -> u16 {
        self.get_reg16(src)
    }
}

impl Out16<Reg16> for Cpu {
    fn write16(&mut self, _: &mut Peripherals, dst: Reg16, value: u16) {
        self.set_reg16(dst, value);
    }
}

impl In8<Addr> for Cpu {
    fn read8(&mut self, pe: &mut Peripherals, src: Addr) -> u8 {
        let addr = self.read_addr(pe, src);
        pe.read8(addr)
    }
}

impl Out8<Addr> for Cpu {
    fn write8(&mut self, pe: &mut Peripherals, dst: Addr, value: u8) {
        let addr = self.read_addr(pe, dst);
        pe.write8(addr, value)
    }
}

impl In16<SP> for Cpu {
    fn read16(&mut self, _: &mut Peripherals, _: SP) -> u16 {
        self.regs.sp
    }
}

impl Out16<SP> for Cpu {
    fn write16(&mut self, _: &mut Peripherals, _: SP, value: u16) {
        self.regs.sp = value;
    }
}

impl In8<Imm8> for Cpu {
    fn read8(&mut self, pe: &mut Peripherals, _: Imm8) -> u8 {
        self.fetch8(pe)
    }
}

impl In16<Imm16> for Cpu {
    fn read16(&mut self, pe: &mut Peripherals, _: Imm16) -> u16 {
        self.fetch16(pe)
    }
}

pub trait Io8<T>: In8<T> + Out8<T> {}

impl<T> Io8<T> for Cpu where Cpu: In8<T> + Out8<T> {}

pub trait Io16<T>: In16<T> + Out16<T> {}

impl<T> Io16<T> for Cpu where Cpu: In16<T> + Out16<T> {}

#[allow(unused)]
#[derive(Default, Clone, Copy)]
pub struct StackState {
    pc: u16,
    sp: u16,
    fetch0: u8,
    fetch1: u8,
    fetch2: u8,
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    ime: bool,
    ie: u8,
    iff: u8,
}

impl StackState {
    fn new(cpu: &Cpu, pe: &Peripherals) -> Self {
        StackState {
            pc: cpu.regs.pc,
            sp: cpu.regs.sp,
            fetch0: pe.read8(cpu.regs.pc),
            fetch1: pe.read8(cpu.regs.pc.wrapping_add(1)),
            fetch2: pe.read8(cpu.regs.pc.wrapping_add(2)),
            af: cpu.get_reg16(Reg16::AF),
            bc: cpu.get_reg16(Reg16::BC),
            de: cpu.get_reg16(Reg16::DE),
            hl: cpu.get_reg16(Reg16::HL),
            ime: cpu.ime,
            ie: pe.inter.enable,
            iff: *pe.inter.flags,
        }
    }

    /// return the instruction description + the first 1 byte following it
    fn get_instruction_str(&self) -> String {
        // yeah, i'm doing this...
        const DECODE_SRC: &str = include_str!("decode.rs");
        const FN_MARKER: &str = " fn ";

        let inner = |code: u8, is_cb: bool| {
            let (_, rest) = DECODE_SRC.split_once(FN_MARKER)?;
            let (src_nocb, src_cb) = rest.split_once(FN_MARKER)?;
            let src = if is_cb { src_cb } else { src_nocb };
            let (_, rest) = src.split_once(&format!("{code:02X} => self."))?;
            let (desc, _) = rest.split_once('\n')?;
            let desc = desc.trim().strip_suffix(",")?;
            Some(desc)
        };

        let is_cb = self.fetch0 == 0xCB;
        let idx = is_cb as usize;
        let fetches = [self.fetch0, self.fetch1, self.fetch2];
        let desc = inner(fetches[idx], is_cb)
            .map(String::from)
            .unwrap_or(format!("{:02X}", fetches[idx]));
        format!("{desc} [{:02X}]", fetches[idx + 1])
    }
}

impl Debug for StackState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let instruction = self.get_instruction_str();
        write!(
            f,
            "PC: {:04X}, SP: {:04X}, I: {instruction}, AF: {:04X}, BC: {:04X}, DE: {:04X}, HL: {:04X}, ime: {}, IE: {:02X}, IF: {:02X}",
            self.pc, self.sp, self.af, self.bc, self.de, self.hl, self.ime as u8, self.ie, self.iff
        )
    }
}

impl PartialEq for StackState {
    fn eq(&self, other: &Self) -> bool {
        self.pc == other.pc
            && self.fetch0 == other.fetch0
            && self.fetch1 == other.fetch1
            && self.fetch2 == other.fetch2
            && self.af == other.af
            && self.bc == other.bc
            && self.de == other.de
            && self.hl == other.hl
            && self.ime == other.ime
            && self.ie == other.ie
            && self.iff == other.iff
    }
}

#[derive(Default)]
pub struct StackTrace {
    labels: BTreeMap<u16, String>,
    trace: [Option<(StackState, u64)>; 8],
    cursor: usize,
}

impl StackTrace {
    /// parse labels found in `.sym` corresponding file
    fn load_rom_labels(cartridge_path: &Path) -> BTreeMap<u16, String> {
        let sym_path = cartridge_path.with_extension("sym");
        let Ok(content) = fs::read_to_string(&sym_path) else {
            return BTreeMap::new();
        };

        let mut in_labels = false;
        content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.starts_with('[') {
                    in_labels = line == "[labels]";
                    return None;
                }

                if !in_labels {
                    return None;
                }

                // "BANK:ADDR LABEL"
                let (bank_and_addr, label) = line.split_once(' ')?;
                let (_, saddr) = bank_and_addr.split_once(':')?;
                let addr = u16::from_str_radix(saddr, 16).ok()?;
                Some((addr, label.into()))
            })
            .collect()
    }

    /// closest label to `pc`
    #[allow(unused)]
    fn lookup_label(&self, pc: u16) -> Option<(u16, &str)> {
        self.labels
            .range(..=pc)
            .next_back()
            .map(|(&addr, label)| (addr, label.as_str()))
    }

    fn format_trace(&self, stack: &StackState, count: u64) -> String {
        //let label_info = self
        //    .lookup_label(stack.pc)
        //    .map(|(addr, label)| format!(" ({label}+{:#X})", stack.pc - addr))
        //    .unwrap_or_default();
        let label_info = String::new();
        let count = if count > 1 {
            format!(" [{count}]")
        } else {
            String::new()
        };
        format!("{stack:?}{label_info}{count}")
    }

    fn push(&mut self, stack: StackState) {
        self.trace[self.cursor] = Some((stack, 1));
        self.cursor = (self.cursor + 1) % self.trace.len();
    }

    fn last(&mut self) -> &mut Option<(StackState, u64)> {
        let idx = self.cursor.checked_sub(1).unwrap_or(self.trace.len() - 1);
        &mut self.trace[idx]
    }

    fn traces(&self) -> impl Iterator<Item = &(StackState, u64)> {
        let len = self.trace.len();
        (0..len).filter_map(move |i| self.trace[(self.cursor + i) % len].as_ref())
    }
}

thread_local! {
    pub static STACK_TRACE: RefCell<StackTrace> = RefCell::new(StackTrace::default());
}

pub fn init_stack_tracing(cartridge_path: &Path) {
    STACK_TRACE.with_borrow_mut(|st| {
        st.labels = StackTrace::load_rom_labels(cartridge_path);
    });

    // set panic hook to print gameboy stack trace too
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("{panic_info}");
        STACK_TRACE.with_borrow(|st| {
            eprintln!("--- stack trace ---");
            for (stack, count) in st.traces() {
                eprintln!("{}", st.format_trace(stack, *count));
            }
        });
    }));
}

fn update_stack_trace(cpu: &Cpu, pe: &Peripherals) {
    STACK_TRACE.with_borrow_mut(|st| {
        let stack = StackState::new(cpu, pe);
        if let Some(last) = st.last()
            && last.0 == stack
        {
            last.1 += 1;
        } else {
            st.push(stack);
        }
    })
}
