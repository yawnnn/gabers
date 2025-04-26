#![allow(unused)]

use std::io;

use num_traits::ops::overflowing::OverflowingAdd;
use num_traits::PrimInt;
use num_traits::WrappingAdd;
use num_traits::WrappingSub;

use crate::cpu::*;
use crate::registers::*;

#[derive(Debug)]
pub struct Opcode {
    pub byte: u8,
    pub extended: bool,
}

pub enum ConditionCode {
    CF,
    NoCF,
    ZF,
    NoZF,
}

#[allow(clippy::upper_case_acronyms)]
pub enum Instruction {
    // ADD(), // add to RA               -
    // ADDHL(), // add to HL               - just like ADD except that the target is added to the HL register
    // ADC(), // add with carry          - just like ADD except that the value of the carry flag is also added to the number
    // SUB(), // subtract                - subtract the value stored in a specific register with the value in the A register
    // SBC(), // subtract with carry     - just like ADD except that the value of the carry flag is also subtracted from the number
    // AND(), // logical and             - do a bitwise and on the value in a specific register and the value in the A register
    // OR(), // logical or              - do a bitwise or on the value in a specific register and the value in the A register
    // XOR(), // logical xor             - do a bitwise xor on the value in a specific register and the value in the A register
    // CP(), // compare                 - just like SUB except the result of the subtraction is not stored back into A
    // INC(), // increment               - increment the value in a specific register by 1
    // DEC(), // decrement               - decrement the value in a specific register by 1
    // CCF(), // complement carry flag   - toggle the value of the carry flag
    // SCF(), // set carry flag          - set the carry flag to true
    // RRA(), // rotate right A register - bit rotate A register right through the carry flag
    // RLA(), // rotate left A register  - bit rotate A register left through the carry flag
    // RRCA(), // rotate right A register - bit rotate A register right (not through the carry flag)
    // RRLA(), // rotate left A register  - bit rotate A register left (not through the carry flag)
    // CPL(), // complement              - toggle every bit of the A register
    // BIT(), // bit test                - test to see if a specific bit of a specific register is set
    // RESET(), // bit reset               - set a specific bit of a specific register to 0
    // SET(), // bit set                 - set a specific bit of a specific register to 1
    // SRL(), // shift right logical     - bit shift a specific register right by 1
    // RR(), // rotate right            - bit rotate a specific register right by 1 through the carry flag
    // RL(), // rotate left             - bit rotate a specific register left by 1 through the carry flag
    // RRC(), // rorate right            - bit rotate a specific register right by 1 (not through the carry flag)
    // RLC(), // rorate left             - bit rotate a specific register left by 1 (not through the carry flag)
    // SRA(), // shift right arithmetic  - arithmetic shift a specific register right by 1
    // SLA(), // shift left arithmetic   - arithmetic shift a specific register left by 1
    // SWAP(), // swap nibbles            - switch upper and lower nibble of a specific register
    // JP(JumpKind), // jump based on flags     -
    // JR(),  // jump base on pc         -
    // JPI(), // jump to [HI]            -
}

trait In8<T: Copy> {
    fn read8(&mut self, what: T) -> u8;
}

trait Out8<T: Copy> {
    fn write8(&mut self, what: T, value: u8);
}

trait In16<T: Copy> {
    fn read16(&mut self, what: T) -> u16;
}

trait Out16<T: Copy> {
    fn write16(&mut self, what: T, value: u16);
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Addr {
    Reg16(Reg16),
    Imm16,
    HiC,
    HiImm8,
    HLI,
    HLD,
}

impl In8<Reg8> for Cpu {
    fn read8(&mut self, in_: Reg8) -> u8 {
        self.regs.read8(in_)
    }
}

impl Out8<Reg8> for Cpu {
    fn write8(&mut self, out_: Reg8, value: u8) {
        self.regs.write8(out_, value);
    }
}

impl In16<Reg16> for Cpu {
    fn read16(&mut self, in_: Reg16) -> u16 {
        self.regs.read16(in_)
    }
}

impl Out16<Reg16> for Cpu {
    fn write16(&mut self, out_: Reg16, value: u16) {
        self.regs.write16(out_, value);
    }
}

impl In8<Addr> for Cpu {
    fn read8(&mut self, in_: Addr) -> u8 {
        let addr = match in_ {
            Addr::HiC => {
                let value = self.read8(Reg8::C);
                0xFF00 | value as u16
            }
            Addr::HiImm8 => {
                let value = self.fetch8();
                0xFF00 | value as u16
            }
            Addr::Reg16(reg16) => self.read16(reg16),
            Addr::Imm16 => self.fetch16(),
            Addr::HLI => {
                let addr = self.read16(Reg16::HL);
                self.write16(Reg16::HL, addr.checked_add(1).unwrap()); // TODO: checked or wrapping?
                addr
            }
            Addr::HLD => {
                let addr = self.read16(Reg16::HL);
                self.write16(Reg16::HL, addr.checked_sub(1).unwrap()); // TODO: checked or wrapping?
                addr
            }
        };

        self.ram.read8(addr)
    }
}

impl Out8<Addr> for Cpu {
    fn write8(&mut self, out_: Addr, value: u8) {
        let addr = match out_ {
            Addr::HiC => {
                let value = self.read8(Reg8::C);
                0xFF00 | value as u16
            }
            Addr::HiImm8 => {
                let value = self.fetch8();
                0xFF00 | value as u16
            }
            Addr::Reg16(reg16) => self.read16(reg16),
            Addr::Imm16 => self.fetch16(),
            Addr::HLI => {
                let addr = self.read16(Reg16::HL);
                self.write16(Reg16::HL, addr.checked_add(1).unwrap()); // TODO: checked or wrapping?
                addr
            }
            Addr::HLD => {
                let addr = self.read16(Reg16::HL);
                self.write16(Reg16::HL, addr.checked_sub(1).unwrap()); // TODO: checked or wrapping?
                addr
            }
        };

        self.ram.write8(addr, value)
    }
}

impl In16<SP> for Cpu {
    fn read16(&mut self, _: SP) -> u16 {
        self.regs.sp()
    }
}

impl Out16<SP> for Cpu {
    fn write16(&mut self, _: SP, value: u16) {
        self.regs.set_sp(value);
    }
}

pub trait NumTraitsExt
where
    Self: std::marker::Sized + PrimInt,
{
    /// return a `Self` such that all bits up to the `bit`th are 1 and the rest are 0
    fn bit_mask(bit: usize) -> Self;

    /// add `nums` with wrapping but check if `bit` has carry
    /// analogous to overflowing_add but over N elements and bit-specific
    fn bit_overflowing_add(nums: &[Self], bit: usize) -> (Self, bool);

    /// sub `nums` with wrapping but check if `bit` has borrow
    /// analogous to overflowing_sub but over N elements and bit-specific
    fn bit_overflowing_sub(nums: &[Self], bit: usize) -> (Self, bool);
}

impl<T> NumTraitsExt for T
where
    T: PrimInt + OverflowingAdd + WrappingSub,
{
    fn bit_mask(bit: usize) -> Self {
        let max_bits = 8 * std::mem::size_of::<Self>();
        assert!(bit < max_bits);

        if bit == max_bits - 1 {
            Self::max_value()
        } else {
            (Self::one() << (bit + 1)) - Self::one()
        }
    }

    fn bit_overflowing_add(nums: &[Self], bit: usize) -> (Self, bool) {
        assert!(!nums.is_empty());

        let mask = Self::bit_mask(bit);
        let mut acc = nums[0] & mask;
        let mut carry = false;

        for &x in &nums[1..] {
            let x_masked = x & mask;
            let (sum, overflow) = acc.overflowing_add(&x_masked);

            if (sum & !mask) != T::zero() || overflow {
                carry = true;
            }

            acc = sum;
        }

        (acc, carry)
    }

    fn bit_overflowing_sub(nums: &[Self], bit: usize) -> (Self, bool) {
        assert!(!nums.is_empty());

        let mask = Self::bit_mask(bit - 1);
        let mut acc = nums[0] & mask;
        let mut borrow = false;

        for &x in &nums[1..] {
            let x_masked = x & mask;

            if acc < x_masked {
                borrow = true;
            }
            
            acc = acc.wrapping_sub(&x_masked);
        }

        (acc, borrow)
    }
}

// TODO: checked or wrapping?
// TODO: in JSON a8 means Addr::HiImm8 or Addr::Imm16
// TODO: which specials (HiImm8, HLI, ..) should i encode in the enums, and which should be special functions
impl Cpu {
    /*
     * LOAD INSTRUCTIONS
     */

    // LD r8, r8
    // LD r8, n8
    // LD [HL], r8
    // LD [HL], n8
    // LD r8, [HL]
    // LD [r16], A
    // LD [n16], A
    // LD A, [r16]
    // LD A, [n16]
    // LD [HLI], A
    // LD [HLD], A
    // LD A, [HLI]
    // LD A, [HLD]
    // Z N H C
    // - - - -
    fn load8<I: Copy, O: Copy>(&mut self, out_: O, in_: I)
    where
        Self: In8<I> + Out8<O>,
    {
        let value = self.read8(in_);
        self.write8(out_, value);
    }

    // LD r16, n16
    // LD SP, n16
    // LD SP, HL
    // Z N H C
    // - - - -
    fn load16<I: Copy, O: Copy>(&mut self, out_: O, in_: I)
    where
        Self: In16<I> + Out16<O>,
    {
        let value = self.read16(in_);
        self.write16(out_, value);
    }

    // LD B, B
    // Z N H C
    // - - - -
    fn load_b_b(&mut self) {
        todo!() // TODO: breakpoint
    }

    // LD D, D
    // Z N H C
    // - - - -
    fn load_d_d(&mut self) {
        todo!() // TODO: debug
    }

    // LD [n16], SP
    // Z N H C
    // - - - -
    fn load_imm16_sp(&mut self) {
        let addr = self.fetch16();
        let value = self.regs.sp();
        let [lo, hi] = u16::to_le_bytes(value);
        self.ram.write8(addr, lo);
        self.ram.write8(addr.wrapping_add(1), hi);
    }

    // LD HL, SP+e8
    // Z N H C
    // - - - -
    fn load_hl_sp_e8(&mut self) {
        let sp = self.regs.sp();
        let offset = self.fetch8() as i8;
        let (res, carry) = u16::bit_overflowing_add(&[sp, offset as i16 as u16], 7);
        let (_, half_carry) = u16::bit_overflowing_add(&[sp, offset as i16 as u16], 3);

        self.regs.set_flag(Flag::ZF, false);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, half_carry);
        self.regs.set_flag(Flag::CF, carry);

        self.write16(Reg16::HL, res);
    }

    // LDH A, [n16]
    // LDH A, [C]
    // Z N H C
    // - - - -
    fn loadh_in(&mut self) {
        let addr = self.fetch16();
        if (0xFF00..0xFFFF).contains(&addr) {
            let value = self.ram.read8(addr);
            self.write8(Reg8::A, value);
        }
    }

    // LDH [n16],A
    // LDH [C],A
    // Z N H C
    // - - - -
    fn loadh_out(&mut self) {
        let addr = self.fetch16();
        if (0xFF00..0xFFFF).contains(&addr) {
            let reg_a = self.read8(Reg8::A);
            self.ram.write8(addr, reg_a);
        }
    }

    /*
     * ARITHMETIC INSTRUCTIONS
     */

    // ADC A, r8
    // ADC A, [HL]
    // ADC A, n8
    // Z N H C
    // * 0 * *
    fn adc8<I: Copy, O: Copy>(&mut self, out_: O, in_: I)
    where
        Self: In8<I> + Out8<O>,
    {
        let value = self.read8(in_);
        let cf = self.regs.flag(Flag::CF);
        let res = self.alu_add(value, cf as u8);
        self.regs.write8(Reg8::A, res);
    }

    // ADD A, r8
    // ADD A, [HL]
    // ADD A, n8
    // Z N H C
    // * 0 * *
    fn add8<I: Copy>(&mut self, in_: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(in_);
        let res = self.alu_add(value, 0);
        self.regs.write8(Reg8::A, res);
    }

    // ADD HL, r16
    // ADD HL, SP
    // Z N H C
    // - 0 * *
    fn add16<I: Copy>(&mut self, in_: I)
    where
        Self: In16<I>,
    {
        let reg_hl = self.regs.read16(Reg16::HL);
        let value = self.read16(in_);
        let (res, carry) = u16::bit_overflowing_add(&[reg_hl, value], 15);
        let (_, half_carry) = u16::bit_overflowing_add(&[reg_hl, value], 11);

        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, half_carry);
        self.regs.set_flag(Flag::CF, carry);

        self.regs.write16(Reg16::HL, res);
    }

    // ADD SP, e8
    // Z N H C
    // 0 0 * *
    fn add_sp_e<I: Copy>(&mut self, in_: I) {
        let reg_sp = self.regs.sp();
        let offset = self.fetch8() as i8;
        let (res, carry) = u16::bit_overflowing_add(&[reg_sp, offset as i16 as u16], 7);
        let (_, half_carry) = u16::bit_overflowing_add(&[reg_sp, offset as i16 as u16], 3);

        self.regs.set_flag(Flag::ZF, false);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, half_carry);
        self.regs.set_flag(Flag::CF, carry);

        self.write16(Reg16::HL, res);
    }

    // CP A, r8
    // CP A, [HL]
    // CP A, n8
    // Z N H C
    // * 1 * *
    fn cmp8<I: Copy>(&mut self, in_: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(in_);
        self.alu_sub(value, 0);
    }

    // DEC r8
    // DEC [HL]
    // Z N H C
    // * 1 * -
    fn dec8<IO: Copy>(&mut self, io_: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io_);
        let res = value.wrapping_sub(1);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, true);
        self.regs.set_flag(Flag::HF, res & 0xF == 0);

        self.write8(io_, res);
    }

    // DEC r16
    // DEC SP
    // Z N H C
    // - - - -
    fn dec16<IO: Copy>(&mut self, out_: IO)
    where
        Self: In16<IO> + Out16<IO>,
    {
        let value = self.read16(out_);
        let res = value.wrapping_sub(1);
        self.write16(out_, res);
    }

    // INC r8
    // INC [HL]
    // Z N H C
    // * 0 * -
    fn inc8<IO: Copy>(&mut self, out_: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(out_);
        let (res, half_carry) = u8::bit_overflowing_add(&[value, 1], 3);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, half_carry);

        self.write8(out_, res);
    }

    // INC r16
    // INC SP
    // Z N H C
    // - - - -
    fn inc16<IO: Copy>(&mut self, out_: IO)
    where
        Self: In16<IO> + Out16<IO>,
    {
        let value = self.read16(out_);
        let res = value.wrapping_add(1);
        self.write16(out_, res);
    }

    // SBC A, r8
    // SBC A, [HL]
    // SBC A, n8
    fn sbc8<I: Copy, O: Copy>(&mut self, in_: I)
    where
        Self: In8<I> + Out8<O>,
    {
        let value = self.read8(in_);
        let cf = self.regs.flag(Flag::CF);
        self.alu_sub(value, cf as u8);
    }

    // SUB A, r8
    // SUB A, [HL]
    // SUB A, n8
    // Z N H C
    // * 1 * *
    fn sub8<I: Copy, O: Copy>(&mut self, in_: I)
    where
        Self: In8<I> + Out8<O>,
    {
        let value = self.read8(in_);
        self.alu_sub(value, 0);
    }

    /*
     * LOGIC INSTRUCTIONS
     */

    // AND A, r8
    // AND A, [HL]
    // AND A, n8
    // Z N H C
    // * 0 1 0
    fn and8<I: Copy, O: Copy>(&mut self, out_: O, in_: I)
    where
        Self: In8<I> + Out8<O>,
    {
        let reg_a = self.regs.read8(Reg8::A);
        let value = self.read8(in_);
        let res = reg_a & value;

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, true);
        self.regs.set_flag(Flag::CF, false);

        self.regs.write8(Reg8::A, res);
    }

    // CPL
    // Z N H C
    // - 1 1 -
    fn cpl(&mut self) {
        let value = self.regs.read8(Reg8::A);
        let res = !value;

        self.regs.set_flag(Flag::NF, true);
        self.regs.set_flag(Flag::HF, true);

        self.regs.write8(Reg8::A, res);
    }

    // OR A, r8
    // OR A, [HL]
    // OR A, n8
    // Z N H C
    // * 0 0 0
    fn or8<I: Copy, O: Copy>(&mut self, out_: O, in_: I)
    where
        Self: In8<I> + Out8<O>,
    {
        let reg_a = self.regs.read8(Reg8::A);
        let value = self.read8(in_);
        let res = reg_a | value;

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);
        self.regs.set_flag(Flag::CF, false);

        self.regs.write8(Reg8::A, res);
    }

    // XOR A, r8
    // XOR A, [HL]
    // XOR A, n8
    // Z N H C
    // * 0 0 0
    fn xor8<I: Copy, O: Copy>(&mut self, out_: O, in_: I)
    where
        Self: In8<I> + Out8<O>,
    {
        let reg_a = self.regs.read8(Reg8::A);
        let value = self.read8(in_);
        let res = reg_a ^ value;

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);
        self.regs.set_flag(Flag::CF, false);

        self.regs.write8(Reg8::A, res);
    }

    /*
     * BIT FLAGS INSTRUCTIONS
     */

    // BIT u3, r8
    // BIT u3, [HL]
    // Z N H C
    // * 0 1 -
    fn bit8<I: Copy>(&mut self, in_: I, bit: u8)
    where
        Self: In8<I>,
    {
        let value = self.read8(in_);
        let res = value & (1 << bit);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, true);
    }

    // RES u3, r8
    // RES u3, [HL]
    // Z N H C
    // - - - -
    fn res8<IO: Copy>(&mut self, io_: IO, bit: u8)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io_);
        let res = value & !(1 << bit);
        self.write8(io_, res);
    }

    // SET u3,r8
    // SET u3,[HL]
    // Z N H C
    // - - - -
    fn set8<IO: Copy>(&mut self, io_: IO, bit: u8)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io_);
        let res = value | (1 << bit);
        self.write8(io_, res);
    }

    /*
     * BIT SHIFT INSTRUCTIONS
     */

    // RL r8
    // RL [HL]
    // Z N H C
    // * 0 0 *
    fn rl8<IO: Copy>(&mut self, io_: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io_);
        let res = self.alu_rl(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io_, res);
    }

    // RLA
    // Z N H C
    // 0 0 0 *
    fn rla(&mut self) {
        let value = self.regs.read8(Reg8::A);
        let res = self.alu_rl(value);

        self.regs.set_flag(Flag::ZF, false);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.regs.write8(Reg8::A, res);
    }

    // RLC r8
    // RLC [HL]
    // Z N H C
    // * 0 0 *
    fn rlc8<IO: Copy>(&mut self, io_: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io_);
        let res = self.alu_rlc(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io_, res);
    }

    // RLCA
    // Z N H C
    // 0 0 0 *
    fn rlca(&mut self) {
        let value = self.regs.read8(Reg8::A);
        let res = self.alu_rlc(value);

        self.regs.set_flag(Flag::ZF, false);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.regs.write8(Reg8::A, res);
    }

    // RR r8
    // RR [HL]
    // Z N H C
    // * 0 0 *
    fn rr8<IO: Copy>(&mut self, io_: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io_);
        let res = self.alu_rr(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io_, res);
    }

    // RRA
    // Z N H C
    // 0 0 0 *
    fn rra(&mut self) {
        let value = self.regs.read8(Reg8::A);
        let res = self.alu_rr(value);

        self.regs.set_flag(Flag::ZF, false);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.regs.write8(Reg8::A, res);
    }

    // RRC r8
    // RRC [HL]
    // Z N H C
    // * 0 0 *
    fn rrc8<IO: Copy>(&mut self, io_: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io_);
        let res = self.alu_rrc(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io_, res);
    }

    // RRCA
    // Z N H C
    // 0 0 0 *
    fn rrca(&mut self) {
        let value = self.regs.read8(Reg8::A);
        let res = self.alu_rrc(value);

        self.regs.set_flag(Flag::ZF, false);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.regs.write8(Reg8::A, res);
    }

    // SLA r8
    // SLA [HL]
    // Z N H C
    // * 0 0 *
    fn sla8<IO: Copy>(&mut self, io_: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io_);
        let res = self.alu_sla(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io_, res);
    }

    // SRA r8
    // SRA [HL]
    // Z N H C
    // * 0 0 *
    fn sra8<IO: Copy>(&mut self, io_: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io_);
        let res = self.alu_sra(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io_, res);
    }

    // SRL r8
    // SRL [HL]
    // Z N H C
    // * 0 0 *
    fn srl8<IO: Copy>(&mut self, io_: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io_);
        let res = self.alu_srl(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io_, res);
    }

    // SWAP r8
    // SWAP [HL]
    // Z N H C
    // * 0 0 0
    fn swap8<IO: Copy>(&mut self, io_: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io_);
        let lo = value << 4;
        let hi = value >> 4;
        let res = lo | hi;

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);
        self.regs.set_flag(Flag::CF, false);

        self.write8(io_, res);
    }

    /*
     * PROGRAM FLOW INSTRUCTIONS
     */

    // CALL n16
    // Z N H C
    // - - - -
    fn call16(&mut self) {
        let addr = self.fetch16();
        self.call(addr);
    }

    // CALL cc, n16
    // Z N H C
    // - - - -
    fn call_cc(&mut self, cc: ConditionCode) {
        let addr = self.fetch16();
        if self.check_condition(cc) {
            self.call(addr);
        }
    }

    // JP HL
    // JP n16
    // Z N H C
    // - - - -
    fn jump16<I: Copy>(&mut self, in_: I)
    where
        Self: In16<I>,
    {
        let addr = self.read16(in_);
        self.regs.set_pc(addr);
    }

    // JP cc, n16
    // Z N H C
    // - - - -
    fn jump_cc(&mut self, cc: ConditionCode) {
        let addr = self.fetch16();
        if self.check_condition(cc) {
            self.regs.set_pc(addr);
        }
    }

    // JR n16
    // Z N H C
    // - - - -
    fn jumpr16(&mut self) {
        let offset = self.fetch8() as i8;
        self.jump_rel(offset);
    }

    // JR cc, n16
    // Z N H C
    // - - - -
    fn jumpr_cc(&mut self, cc: ConditionCode) {
        let offset = self.fetch8() as i8;
        if self.check_condition(cc) {
            self.jump_rel(offset);
        }
    }

    // RET cc
    // Z N H C
    // - - - -
    fn ret_cc(&mut self, cc: ConditionCode) {
        if self.check_condition(cc) {
            self.ret();
        }
    }

    // RET
    // Z N H C
    // - - - -
    fn ret(&mut self) {
        let value = self.stack_pop();
        self.regs.set_pc(value);
    }

    // TODO: RETI
    // TODO: RST vec

    /*
     * CARRY FLAG INSTRUCTIONS
     */

    // CCF
    // Z N H C
    // - 0 0 *
    fn ccf(&mut self) {
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);
        self.regs.set_flag(Flag::CF, !self.regs.flag(Flag::CF));
    }

    // SCF
    // Z N H C
    // - 0 0 1
    fn scf(&mut self) {
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);
        self.regs.set_flag(Flag::CF, true);
    }

    /*
     * STACK INSTRUCTIONS
     */

    // POP AF
    // POP r16
    // Z N H C
    // * * * *
    fn pop16<O: Copy>(&mut self, out_: O)
    where
        Self: Out16<O>,
    {
        let value = self.stack_pop();
        self.write16(out_, value);
    }

    // PUSH AF
    // PUSH r16
    // Z N H C
    // * * * *
    fn push16<I: Copy>(&mut self, in_: I)
    where
        Self: In16<I>,
    {
        let value = self.read16(in_);
        self.stack_push(value);
    }

    /*
     * INTERRUPT INSTRUCTIONS
     */

    // TODO: DI
    // TODO: EI
    // TODO: HALT

    /*
     * MISC INSTRUCTIONS
     */

    // TODO: DAA

    // NOP
    // Z N H C
    // - - - -
    fn noop(&self) {}

    // STOP
    // Z N H C
    // - - - -
    fn stop(&mut self) {
        panic!("STOP")
    }
}
