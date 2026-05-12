use crate::common::*;
use crate::constants::*;
use crate::cpu::*;
use crate::registers::*;

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
    pub fn load8<I: Copy, O: Copy>(&mut self, dst: O, src: I)
    where
        Self: In8<I> + Out8<O>,
    {
        let value = self.read8(src);
        self.write8(dst, value);
    }

    // LD r16, n16
    // LD SP, n16
    // LD SP, HL
    // Z N H C
    // - - - -
    pub fn load16<I: Copy, O: Copy>(&mut self, dst: O, src: I)
    where
        Self: In16<I> + Out16<O>,
    {
        let value = self.read16(src);
        self.write16(dst, value);
    }

    // LD B, B
    // Z N H C
    // - - - -
    pub fn load8_b_b(&mut self) {
        todo!() // TODO: breakpoint
    }

    // LD D, D
    // Z N H C
    // - - - -
    pub fn load8_d_d(&mut self) {
        todo!() // TODO: debug
    }

    // LD [n16], SP
    // Z N H C
    // - - - -
    pub fn load16_imm_sp(&mut self) {
        let addr = self.fetch16();
        let value = self.read16(SP);
        let [lo, hi] = u16::to_le_bytes(value);
        self.mmu.write8(addr, lo);
        self.mmu.write8(addr.wrapping_add(1), hi);
    }

    // LD HL, SP+e8
    // Z N H C
    // - - - -
    pub fn load16_hl_sp_e8(&mut self) {
        let sp = self.read16(SP);
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
    pub fn ldh8_addr_a(&mut self, addr: Addr) {
        let addr = addr.read_addr(self);
        if LDH_RANGE.contains(&(addr as usize)) {
            let value = self.mmu.read8(addr);
            self.write8(Reg8::A, value);
        }
    }

    // LDH [n16], A
    // LDH [C], A
    // Z N H C
    // - - - -
    pub fn ldh8_a_addr(&mut self, addr: Addr) {
        let addr = addr.read_addr(self);
        if LDH_RANGE.contains(&(addr as usize)) {
            let value = self.read8(Reg8::A);
            self.mmu.write8(addr, value);
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
    pub fn adc8<I: Copy>(&mut self, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(src);
        let cf = self.regs.get_flag(Flag::CF);
        let res = self.alu_add(value, cf as u8);
        self.regs.write8(Reg8::A, res);
    }

    // ADD A, r8
    // ADD A, [HL]
    // ADD A, n8
    // Z N H C
    // * 0 * *
    pub fn add8<I: Copy>(&mut self, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(src);
        let res = self.alu_add(value, 0);
        self.regs.write8(Reg8::A, res);
    }

    // ADD HL, r16
    // ADD HL, SP
    // Z N H C
    // - 0 * *
    pub fn add16<I: Copy>(&mut self, src: I)
    where
        Self: In16<I>,
    {
        let reg_hl = self.regs.read16(Reg16::HL);
        let value = self.read16(src);
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
    pub fn add16_sp_e(&mut self) {
        let sp = self.read16(SP);
        let offset = self.fetch8() as i8;
        let (res, carry) = u16::bit_overflowing_add(&[sp, offset as i16 as u16], 7);
        let (_, half_carry) = u16::bit_overflowing_add(&[sp, offset as i16 as u16], 3);

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
    pub fn cmp8<I: Copy>(&mut self, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(src);
        self.alu_sub(value, 0);
    }

    // DEC r8
    // DEC [HL]
    // Z N H C
    // * 1 * -
    pub fn dec8<IO: Copy>(&mut self, io: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io);
        let res = value.wrapping_sub(1);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, true);
        self.regs.set_flag(Flag::HF, res & 0xF == 0);

        self.write8(io, res);
    }

    // DEC r16
    // DEC SP
    // Z N H C
    // - - - -
    pub fn dec16<IO: Copy>(&mut self, dst: IO)
    where
        Self: In16<IO> + Out16<IO>,
    {
        let value = self.read16(dst);
        let res = value.wrapping_sub(1);
        self.write16(dst, res);
    }

    // INC r8
    // INC [HL]
    // Z N H C
    // * 0 * -
    pub fn inc8<IO: Copy>(&mut self, dst: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(dst);
        let (res, half_carry) = u8::bit_overflowing_add(&[value, 1], 3);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, half_carry);

        self.write8(dst, res);
    }

    // INC r16
    // INC SP
    // Z N H C
    // - - - -
    pub fn inc16<IO: Copy>(&mut self, dst: IO)
    where
        Self: In16<IO> + Out16<IO>,
    {
        let value = self.read16(dst);
        let res = value.wrapping_add(1);
        self.write16(dst, res);
    }

    // SBC A, r8
    // SBC A, [HL]
    // SBC A, n8
    pub fn sbc8<I: Copy>(&mut self, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(src);
        let cf = self.regs.get_flag(Flag::CF);
        self.alu_sub(value, cf as u8);
    }

    // SUB A, r8
    // SUB A, [HL]
    // SUB A, n8
    // Z N H C
    // * 1 * *
    pub fn sub8<I: Copy>(&mut self, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(src);
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
    pub fn and8<I: Copy>(&mut self, src: I)
    where
        Self: In8<I>,
    {
        let reg_a = self.regs.read8(Reg8::A);
        let value = self.read8(src);
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
    pub fn cpl(&mut self) {
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
    pub fn or8<I: Copy>(&mut self, src: I)
    where
        Self: In8<I>,
    {
        let reg_a = self.regs.read8(Reg8::A);
        let value = self.read8(src);
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
    pub fn xor8<I: Copy>(&mut self, src: I)
    where
        Self: In8<I>,
    {
        let reg_a = self.regs.read8(Reg8::A);
        let value = self.read8(src);
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
    pub fn bit8<I: Copy>(&mut self, bit: u8, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(src);
        let res = value & (1 << bit);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, true);
    }

    // RES u3, r8
    // RES u3, [HL]
    // Z N H C
    // - - - -
    pub fn res8<IO: Copy>(&mut self, bit: u8, io: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io);
        let res = value & !(1 << bit);
        self.write8(io, res);
    }

    // SET u3,r8
    // SET u3,[HL]
    // Z N H C
    // - - - -
    pub fn set8<IO: Copy>(&mut self, bit: u8, io: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io);
        let res = value | (1 << bit);
        self.write8(io, res);
    }

    /*
     * BIT SHIFT INSTRUCTIONS
     */

    // RL r8
    // RL [HL]
    // Z N H C
    // * 0 0 *
    pub fn rl8<IO: Copy>(&mut self, io: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io);
        let res = self.alu_rl(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io, res);
    }

    // RLA
    // Z N H C
    // 0 0 0 *
    pub fn rla(&mut self) {
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
    pub fn rlc8<IO: Copy>(&mut self, io: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io);
        let res = self.alu_rlc(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io, res);
    }

    // RLCA
    // Z N H C
    // 0 0 0 *
    pub fn rlca(&mut self) {
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
    pub fn rr8<IO: Copy>(&mut self, io: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io);
        let res = self.alu_rr(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io, res);
    }

    // RRA
    // Z N H C
    // 0 0 0 *
    pub fn rra(&mut self) {
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
    pub fn rrc8<IO: Copy>(&mut self, io: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io);
        let res = self.alu_rrc(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io, res);
    }

    // RRCA
    // Z N H C
    // 0 0 0 *
    pub fn rrca(&mut self) {
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
    pub fn sla8<IO: Copy>(&mut self, io: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io);
        let res = self.alu_sla(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io, res);
    }

    // SRA r8
    // SRA [HL]
    // Z N H C
    // * 0 0 *
    pub fn sra8<IO: Copy>(&mut self, io: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io);
        let res = self.alu_sra(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io, res);
    }

    // SRL r8
    // SRL [HL]
    // Z N H C
    // * 0 0 *
    pub fn srl8<IO: Copy>(&mut self, io: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io);
        let res = self.alu_srl(value);

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);

        self.write8(io, res);
    }

    // SWAP r8
    // SWAP [HL]
    // Z N H C
    // * 0 0 0
    pub fn swap8<IO: Copy>(&mut self, io: IO)
    where
        Self: In8<IO> + Out8<IO>,
    {
        let value = self.read8(io);
        let lo = value << 4;
        let hi = value >> 4;
        let res = lo | hi;

        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);
        self.regs.set_flag(Flag::CF, false);

        self.write8(io, res);
    }

    /*
     * PROGRAM FLOW INSTRUCTIONS
     */

    // CALL n16
    // Z N H C
    // - - - -
    pub fn call16(&mut self) {
        let addr = self.fetch16();
        self.call(addr);
    }

    // CALL cond, n16
    // Z N H C
    // - - - -
    pub fn call16_cc(&mut self, cond: Condition) {
        let addr = self.fetch16();
        if self.check_condition(cond) {
            self.call(addr);
        }
    }

    // JP HL
    // JP n16
    // Z N H C
    // - - - -
    pub fn jump16<I: Copy>(&mut self, src: I)
    where
        Self: In16<I>,
    {
        let addr = self.read16(src);
        self.jump_abs(addr);
    }

    // JP cond, n16
    // Z N H C
    // - - - -
    pub fn jump16_cc(&mut self, cond: Condition) {
        let addr = self.fetch16();
        if self.check_condition(cond) {
            self.jump_abs(addr);
        }
    }

    // JR n16
    // Z N H C
    // - - - -
    pub fn jumpr8(&mut self) {
        let offset = self.fetch8() as i8;
        self.jump_rel(offset);
    }

    // JR cond, n16
    // Z N H C
    // - - - -
    pub fn jumpr8_cc(&mut self, cond: Condition) {
        let offset = self.fetch8() as i8;
        if self.check_condition(cond) {
            self.jump_rel(offset);
        }
    }

    // RET cond
    // Z N H C
    // - - - -
    pub fn ret_cc(&mut self, cond: Condition) {
        if self.check_condition(cond) {
            self.ret();
        }
    }

    // RET
    // Z N H C
    // - - - -
    pub fn ret(&mut self) {
        let addr = self.stack_pop();
        self.jump_abs(addr);
    }

    // RETI
    // Z N H C
    // - - - -
    pub fn reti(&mut self) {
        self.ret();
        self.ime = true;
    }

    // RST vec
    // Z N H C
    // - - - -
    pub fn rst(&mut self, addr: u8) {
        self.call(addr as u16);
    }

    /*
     * CARRY FLAG INSTRUCTIONS
     */

    // CCF
    // Z N H C
    // - 0 0 *
    pub fn ccf(&mut self) {
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);
        self.regs.set_flag(Flag::CF, !self.regs.get_flag(Flag::CF));
    }

    // SCF
    // Z N H C
    // - 0 0 1
    pub fn scf(&mut self) {
        self.regs.set_flag(Flag::NF, false);
        self.regs.set_flag(Flag::HF, false);
        self.regs.set_flag(Flag::CF, true);
    }

    /*
     * STACK INSTRUCTIONS
     */

    // POP r16
    // Z N H C
    // - - - -
    // POP AF
    // Z N H C
    // * * * *
    pub fn pop16<O: Copy>(&mut self, dst: O)
    where
        Self: Out16<O>,
    {
        let value = self.stack_pop();
        self.write16(dst, value);
    }

    // PUSH AF
    // PUSH r16
    // Z N H C
    // - - - -
    pub fn push16<I: Copy>(&mut self, src: I)
    where
        Self: In16<I>,
    {
        let value = self.read16(src);
        self.stack_push(value);
    }

    /*
     * INTERRUPT INSTRUCTIONS
     */

    // DI
    // Z N H C
    // - - - -
    pub fn di(&mut self) {
        self.ime = false;
        self.pending_enable_ime = false;
    }

    // EI
    // Z N H C
    // - - - -
    pub fn ei(&mut self) {
        self.pending_enable_ime = true;
    }

    // TODO: HALT

    /*
     * MISC INSTRUCTIONS
     */

    // DAA
    // Z N H C
    // * - 0 *
    pub fn daa(&mut self) {
        let mut adj: u8 = 0;
        let half_carry = self.regs.get_flag(Flag::HF);
        let mut carry =self.regs.get_flag(Flag::CF);
        let a = self.read8(Reg8::A);
        let res = if self.regs.get_flag(Flag::NF) {
            if half_carry {
                adj |= 0x06;
            }
            if carry {
                adj |= 0x60;
            }
            a.wrapping_sub(adj)
        }
        else {
            if half_carry || a & 0x0F > 0x09 {
                adj |= 0x06;
            }
            if carry || a > 0x99 {
                adj |= 0x60;
                carry = true;
            }
            a.wrapping_add(adj)
        };
        self.write8(Reg8::A, res);
        self.regs.set_flag(Flag::ZF, res == 0);
        self.regs.set_flag(Flag::HF, false);
        self.regs.set_flag(Flag::CF, carry);
    }

    // NOP
    // Z N H C
    // - - - -
    pub fn noop(&self) {}

    // STOP
    // Z N H C
    // - - - -
    pub fn stop(&mut self) {
        panic!("STOP")
    }
}
