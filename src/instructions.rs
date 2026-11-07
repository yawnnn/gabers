use crate::common::*;
use crate::cpu::*;
use crate::registers::*;
use crate::*;

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
    pub fn load8<I, O>(&mut self, pe: &mut Peripherals, dst: O, src: I)
    where
        Self: In8<I> + Out8<O>,
    {
        let value = self.read8(pe, src);
        self.write8(pe, dst, value);
    }

    // LD r16, n16
    // LD SP, n16
    // LD SP, HL
    // Z N H C
    // - - - -
    pub fn load16<I, O>(&mut self, pe: &mut Peripherals, dst: O, src: I)
    where
        Self: In16<I> + Out16<O>,
    {
        let value = self.read16(pe, src);
        self.write16(pe, dst, value);
    }

    // LD B, B
    // Z N H C
    // - - - -
    pub fn load8_b_b(&mut self, _: &mut Peripherals) {
        // TODO: breakpoint
    }

    // LD D, D
    // Z N H C
    // - - - -
    pub fn load8_d_d(&mut self, _: &mut Peripherals) {
        // TODO: debug
    }

    // LD [n16], SP
    // Z N H C
    // - - - -
    pub fn load16_imm_sp(&mut self, pe: &mut Peripherals) {
        let addr = self.fetch16(pe);
        let value = self.read16(pe, SP);
        let [lo, hi] = u16::to_le_bytes(value);
        pe.write8(addr, lo);
        pe.write8(addr.wrapping_add(1), hi);
    }

    // LD HL, SP+e8
    // Z N H C
    // 0 0 * *
    pub fn load16_hl_sp_e8(&mut self, pe: &mut Peripherals) {
        let sp = self.read16(pe, SP);
        let offset = self.fetch8(pe);
        let res = sp.wrapping_add(offset as i8 as i16 as u16);
        let sp8 = (sp & 0xFF) as u8;
        let (_, carry) = sp8.overflowing_add(offset);
        let (_, half_carry) = sp8.half_carrying_add(offset, false);
        self.write16(pe, Reg16::HL, res);

        self.set_flag(Flag::Z, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, half_carry);
        self.set_flag(Flag::C, carry);
    }

    // LDH A, [n16]
    // LDH A, [C]
    // Z N H C
    // - - - -
    pub fn ldh8_addr_a<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In8<I>,
    {
        let addr = 0xFF00 | (self.read8(pe, src) as u16);
        let value = pe.read8(addr);
        self.write8(pe, Reg8::A, value);
    }

    // LDH [n16], A
    // LDH [C], A
    // Z N H C
    // - - - -
    pub fn ldh8_a_addr<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In8<I> + In8<Reg8>,
    {
        let addr = 0xFF00 | (self.read8(pe, src) as u16);
        let value = self.read8(pe, Reg8::A);
        pe.write8(addr, value);
    }

    /*
     * ARITHMETIC INSTRUCTIONS
     */

    // ADC A, r8
    // ADC A, [HL]
    // ADC A, n8
    // Z N H C
    // * 0 * *
    pub fn adc8<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(pe, src);
        let cf = self.get_flag(Flag::C);
        let res = self.alu_add(value, cf);
        self.set_reg8(Reg8::A, res);
    }

    // ADD A, r8
    // ADD A, [HL]
    // ADD A, n8
    // Z N H C
    // * 0 * *
    pub fn add8<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(pe, src);
        let res = self.alu_add(value, false);
        self.set_reg8(Reg8::A, res);
    }

    // ADD HL, r16
    // ADD HL, SP
    // Z N H C
    // - 0 * *
    pub fn add16_hl<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In16<I>,
    {
        let hl = self.get_reg16(Reg16::HL);
        let value = self.read16(pe, src);
        let (res, carry) = hl.overflowing_add(value);
        let (_, half_carry) = hl.half_carrying_add(value, false);
        self.set_reg16(Reg16::HL, res);

        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, half_carry);
        self.set_flag(Flag::C, carry);
    }

    // ADD SP, e8
    // Z N H C
    // 0 0 * *
    pub fn add16_sp_e(&mut self, pe: &mut Peripherals) {
        let sp = self.read16(pe, SP);
        let offset = self.fetch8(pe);
        let res = sp.wrapping_add(offset as i8 as i16 as u16);
        let sp8 = (sp & 0xFF) as u8;
        let (_, carry) = sp8.overflowing_add(offset);
        let (_, half_carry) = sp8.half_carrying_add(offset, false);
        self.write16(pe, SP, res);

        self.set_flag(Flag::Z, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, half_carry);
        self.set_flag(Flag::C, carry);
    }

    // CP A, r8
    // CP A, [HL]
    // CP A, n8
    // Z N H C
    // * 1 * *
    pub fn cmp8<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(pe, src);
        self.alu_sub(value, false);
    }

    // DEC r8
    // DEC [HL]
    // Z N H C
    // * 1 * -
    pub fn dec8<Io: Copy>(&mut self, pe: &mut Peripherals, io: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, io);
        let (res, half_carry) = value.half_borrowing_sub(1, false);
        self.write8(pe, io, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::H, half_carry);
    }

    // DEC r16
    // DEC SP
    // Z N H C
    // - - - -
    pub fn dec16<Io: Copy>(&mut self, pe: &mut Peripherals, dst: Io)
    where
        Self: Io16<Io>,
    {
        let value = self.read16(pe, dst);
        let res = value.wrapping_sub(1);
        self.write16(pe, dst, res);
    }

    // INC r8
    // INC [HL]
    // Z N H C
    // * 0 * -
    pub fn inc8<Io: Copy>(&mut self, pe: &mut Peripherals, dst: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, dst);
        let (res, half_carry) = value.half_carrying_add(1, false);
        self.write8(pe, dst, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, half_carry);
    }

    // INC r16
    // INC SP
    // Z N H C
    // - - - -
    pub fn inc16<Io: Copy>(&mut self, pe: &mut Peripherals, dst: Io)
    where
        Self: Io16<Io>,
    {
        let value = self.read16(pe, dst);
        let res = value.wrapping_add(1);
        self.write16(pe, dst, res);
    }

    // SBC A, r8
    // SBC A, [HL]
    // SBC A, n8
    pub fn sbc8<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(pe, src);
        let cf = self.get_flag(Flag::C);
        let res = self.alu_sub(value, cf);
        self.write8(pe, Reg8::A, res);
    }

    // SUB A, r8
    // SUB A, [HL]
    // SUB A, n8
    // Z N H C
    // * 1 * *
    pub fn sub8<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(pe, src);
        let res = self.alu_sub(value, false);
        self.write8(pe, Reg8::A, res);
    }

    /*
     * LOGIC INSTRUCTIONS
     */

    // AND A, r8
    // AND A, [HL]
    // AND A, n8
    // Z N H C
    // * 0 1 0
    pub fn and8<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In8<I>,
    {
        let reg_a = self.get_reg8(Reg8::A);
        let value = self.read8(pe, src);
        let res = reg_a & value;
        self.set_reg8(Reg8::A, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, true);
        self.set_flag(Flag::C, false);
    }

    // CPL
    // Z N H C
    // - 1 1 -
    pub fn cpl(&mut self, _: &mut Peripherals) {
        let value = self.get_reg8(Reg8::A);
        let res = !value;
        self.set_reg8(Reg8::A, res);

        self.set_flag(Flag::N, true);
        self.set_flag(Flag::H, true);
    }

    // OR A, r8
    // OR A, [HL]
    // OR A, n8
    // Z N H C
    // * 0 0 0
    pub fn or8<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In8<I>,
    {
        let reg_a = self.get_reg8(Reg8::A);
        let value = self.read8(pe, src);
        let res = reg_a | value;
        self.set_reg8(Reg8::A, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, false);
    }

    // XOR A, r8
    // XOR A, [HL]
    // XOR A, n8
    // Z N H C
    // * 0 0 0
    pub fn xor8<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In8<I>,
    {
        let reg_a = self.get_reg8(Reg8::A);
        let value = self.read8(pe, src);
        let res = reg_a ^ value;
        self.set_reg8(Reg8::A, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, false);
    }

    /*
     * BIT FLAGS INSTRUCTIONS
     */

    // BIT u3, r8
    // BIT u3, [HL]
    // Z N H C
    // * 0 1 -
    pub fn bit8<I>(&mut self, pe: &mut Peripherals, bit: u8, src: I)
    where
        Self: In8<I>,
    {
        let value = self.read8(pe, src);
        let res = value & (1 << bit);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, true);
    }

    // RES u3, r8
    // RES u3, [HL]
    // Z N H C
    // - - - -
    pub fn res8<Io: Copy>(&mut self, pe: &mut Peripherals, bit: u8, io: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, io);
        let res = value & !(1 << bit);
        self.write8(pe, io, res);
    }

    // SET u3,r8
    // SET u3,[HL]
    // Z N H C
    // - - - -
    pub fn set8<Io: Copy>(&mut self, pe: &mut Peripherals, bit: u8, io: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, io);
        let res = value | (1 << bit);
        self.write8(pe, io, res);
    }

    /*
     * BIT SHIFT INSTRUCTIONS
     */

    // RL r8
    // RL [HL]
    // Z N H C
    // * 0 0 *
    pub fn rl8<Io: Copy>(&mut self, pe: &mut Peripherals, io: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, io);
        let res = self.alu_rl(value);
        self.write8(pe, io, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }

    // RLA
    // Z N H C
    // 0 0 0 *
    pub fn rla(&mut self, _: &mut Peripherals) {
        let value = self.get_reg8(Reg8::A);
        let res = self.alu_rl(value);
        self.set_reg8(Reg8::A, res);

        self.set_flag(Flag::Z, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }

    // RLC r8
    // RLC [HL]
    // Z N H C
    // * 0 0 *
    pub fn rlc8<Io: Copy>(&mut self, pe: &mut Peripherals, io: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, io);
        let res = self.alu_rlc(value);
        self.write8(pe, io, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }

    // RLCA
    // Z N H C
    // 0 0 0 *
    pub fn rlca(&mut self, _: &mut Peripherals) {
        let value = self.get_reg8(Reg8::A);
        let res = self.alu_rlc(value);
        self.set_reg8(Reg8::A, res);

        self.set_flag(Flag::Z, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }

    // RR r8
    // RR [HL]
    // Z N H C
    // * 0 0 *
    pub fn rr8<Io: Copy>(&mut self, pe: &mut Peripherals, io: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, io);
        let res = self.alu_rr(value);
        self.write8(pe, io, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }

    // RRA
    // Z N H C
    // 0 0 0 *
    pub fn rra(&mut self, _: &mut Peripherals) {
        let value = self.get_reg8(Reg8::A);
        let res = self.alu_rr(value);
        self.set_reg8(Reg8::A, res);

        self.set_flag(Flag::Z, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }

    // RRC r8
    // RRC [HL]
    // Z N H C
    // * 0 0 *
    pub fn rrc8<Io: Copy>(&mut self, pe: &mut Peripherals, io: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, io);
        let res = self.alu_rrc(value);
        self.write8(pe, io, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }

    // RRCA
    // Z N H C
    // 0 0 0 *
    pub fn rrca(&mut self, _: &mut Peripherals) {
        let value = self.get_reg8(Reg8::A);
        let res = self.alu_rrc(value);
        self.set_reg8(Reg8::A, res);

        self.set_flag(Flag::Z, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }

    // SLA r8
    // SLA [HL]
    // Z N H C
    // * 0 0 *
    pub fn sla8<Io: Copy>(&mut self, pe: &mut Peripherals, io: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, io);
        let res = self.alu_sla(value);
        self.write8(pe, io, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }

    // SRA r8
    // SRA [HL]
    // Z N H C
    // * 0 0 *
    pub fn sra8<Io: Copy>(&mut self, pe: &mut Peripherals, io: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, io);
        let res = self.alu_sra(value);
        self.write8(pe, io, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }

    // SRL r8
    // SRL [HL]
    // Z N H C
    // * 0 0 *
    pub fn srl8<Io: Copy>(&mut self, pe: &mut Peripherals, io: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, io);
        let res = self.alu_srl(value);
        self.write8(pe, io, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
    }

    // SWAP r8
    // SWAP [HL]
    // Z N H C
    // * 0 0 0
    pub fn swap8<Io: Copy>(&mut self, pe: &mut Peripherals, io: Io)
    where
        Self: Io8<Io>,
    {
        let value = self.read8(pe, io);
        let lo = value << 4;
        let hi = value >> 4;
        let res = lo | hi;
        self.write8(pe, io, res);

        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, false);
    }

    /*
     * PROGRAM FLOW INSTRUCTIONS
     */

    // CALL n16
    // Z N H C
    // - - - -
    pub fn call16(&mut self, pe: &mut Peripherals) {
        let addr = self.fetch16(pe);
        self.call(pe, addr);
    }

    // CALL cond, n16
    // Z N H C
    // - - - -
    pub fn call16_cc(&mut self, pe: &mut Peripherals, cond: Condition) {
        let addr = self.fetch16(pe);
        if self.check_condition(cond) {
            self.call(pe, addr);
        }
    }

    // JP HL
    // JP n16
    // Z N H C
    // - - - -
    pub fn jump16<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In16<I>,
    {
        let addr = self.read16(pe, src);
        self.jump_abs(addr);
    }

    // JP cond, n16
    // Z N H C
    // - - - -
    pub fn jump16_cc(&mut self, pe: &mut Peripherals, cond: Condition) {
        let addr = self.fetch16(pe);
        if self.check_condition(cond) {
            self.jump_abs(addr);
        }
    }

    // JR n16
    // Z N H C
    // - - - -
    pub fn jumpr8(&mut self, pe: &mut Peripherals) {
        let offset = self.fetch8(pe) as i8;
        self.jump_rel(offset);
    }

    // JR cond, n16
    // Z N H C
    // - - - -
    pub fn jumpr8_cc(&mut self, pe: &mut Peripherals, cond: Condition) {
        let offset = self.fetch8(pe) as i8;
        if self.check_condition(cond) {
            self.jump_rel(offset);
        }
    }

    // RET cond
    // Z N H C
    // - - - -
    pub fn ret_cc(&mut self, pe: &mut Peripherals, cond: Condition) {
        if self.check_condition(cond) {
            self.ret(pe);
        }
    }

    // RET
    // Z N H C
    // - - - -
    pub fn ret(&mut self, pe: &mut Peripherals) {
        let addr = self.stack_pop(pe);
        self.jump_abs(addr);
    }

    // RETI
    // Z N H C
    // - - - -
    pub fn reti(&mut self, pe: &mut Peripherals) {
        self.ret(pe);
        self.ime = true;
    }

    // RST vec
    // Z N H C
    // - - - -
    pub fn rst(&mut self, pe: &mut Peripherals, addr: u8) {
        self.call(pe, addr as u16);
    }

    /*
     * CARRY FLAG INSTRUCTIONS
     */

    // CCF
    // Z N H C
    // - 0 0 *
    pub fn ccf(&mut self, _: &mut Peripherals) {
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, !self.get_flag(Flag::C));
    }

    // SCF
    // Z N H C
    // - 0 0 1
    pub fn scf(&mut self, _: &mut Peripherals) {
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, true);
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
    pub fn pop16<O>(&mut self, pe: &mut Peripherals, dst: O)
    where
        Self: Out16<O>,
    {
        let value = self.stack_pop(pe);
        self.write16(pe, dst, value);
    }

    // PUSH AF
    // PUSH r16
    // Z N H C
    // - - - -
    pub fn push16<I>(&mut self, pe: &mut Peripherals, src: I)
    where
        Self: In16<I>,
    {
        let value = self.read16(pe, src);
        self.stack_push(pe, value);
    }

    /*
     * INTERRUPT INSTRUCTIONS
     */

    // DI
    // Z N H C
    // - - - -
    pub fn di(&mut self, _: &mut Peripherals) {
        self.ime = false;
        self.pending_enable_ime = false;
    }

    // EI
    // Z N H C
    // - - - -
    pub fn ei(&mut self, _: &mut Peripherals) {
        self.pending_enable_ime = true;
    }

    // HALT
    pub fn halt(&mut self, pe: &mut Peripherals) {
        if !self.ime && pe.inter.enabled() != 0 {
            self.halt_bug = true;
        } else {
            self.low_power_mode = true;
        }
    }

    /*
     * MISC INSTRUCTIONS
     */

    // DAA
    // Z N H C
    // * - 0 *
    pub fn daa(&mut self, pe: &mut Peripherals) {
        let mut adjust: u8 = 0;
        let half_carry = self.get_flag(Flag::H);
        let mut carry = self.get_flag(Flag::C);
        let a = self.read8(pe, Reg8::A);
        let res = if self.get_flag(Flag::N) {
            if half_carry {
                adjust += 0x06;
            }

            if carry {
                adjust += 0x60;
            }
            a.wrapping_sub(adjust)
        } else {
            if half_carry || a & 0x0F > 0x09 {
                adjust += 0x06;
            }

            if carry || a > 0x99 {
                adjust += 0x60;
                carry = true;
            }
            a.wrapping_add(adjust)
        };
        self.write8(pe, Reg8::A, res);
        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, carry);
    }

    // NOP
    // Z N H C
    // - - - -
    pub fn nop(&self) {}

    // STOP
    // Z N H C
    // - - - -
    pub fn stop(&mut self, _: &mut Peripherals) {}
}
