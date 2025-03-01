#![allow(unused)]

use crate::cpu::*;
use crate::registers::*;

#[derive(Debug)]
pub struct Opcode {
    pub byte: u8,
    pub extended: bool,
}

pub enum JumpKind {
    Unconditional,
    Carry,
    NotCarry,
    Zero,
    NotZero,
}

pub enum LoadKind {
    Byte(RegKind, RegKind),
    Word,             // just like the Byte type except with 16-bit values
    AFromIndirect, // load the A register with the contents from a value from a memory location whose address is stored in some location
    IndirectFromA, // load a memory location whose address is stored in some location with the contents of the A register
    AFromByteAddress, // Just like AFromIndirect except the memory address is some address in the very last byte of memory.
    ByteAddressFromA, // Just like IndirectFromA except the memory address is some address in the very last byte of memory.
}

pub enum Instruction {
    ADD(),        // add to RA               -
    ADDHL(), // add to HL               - just like ADD except that the target is added to the HL register
    ADC(), // add with carry          - just like ADD except that the value of the carry flag is also added to the number
    SUB(), // subtract                - subtract the value stored in a specific register with the value in the A register
    SBC(), // subtract with carry     - just like ADD except that the value of the carry flag is also subtracted from the number
    AND(), // logical and             - do a bitwise and on the value in a specific register and the value in the A register
    OR(), // logical or              - do a bitwise or on the value in a specific register and the value in the A register
    XOR(), // logical xor             - do a bitwise xor on the value in a specific register and the value in the A register
    CP(), // compare                 - just like SUB except the result of the subtraction is not stored back into A
    INC(), // increment               - increment the value in a specific register by 1
    DEC(), // decrement               - decrement the value in a specific register by 1
    CCF(), // complement carry flag   - toggle the value of the carry flag
    SCF(), // set carry flag          - set the carry flag to true
    RRA(), // rotate right A register - bit rotate A register right through the carry flag
    RLA(), // rotate left A register  - bit rotate A register left through the carry flag
    RRCA(), // rotate right A register - bit rotate A register right (not through the carry flag)
    RRLA(), // rotate left A register  - bit rotate A register left (not through the carry flag)
    CPL(), // complement              - toggle every bit of the A register
    BIT(), // bit test                - test to see if a specific bit of a specific register is set
    RESET(), // bit reset               - set a specific bit of a specific register to 0
    SET(), // bit set                 - set a specific bit of a specific register to 1
    SRL(), // shift right logical     - bit shift a specific register right by 1
    RR(), // rotate right            - bit rotate a specific register right by 1 through the carry flag
    RL(), // rotate left             - bit rotate a specific register left by 1 through the carry flag
    RRC(), // rorate right            - bit rotate a specific register right by 1 (not through the carry flag)
    RLC(), // rorate left             - bit rotate a specific register left by 1 (not through the carry flag)
    SRA(), // shift right arithmetic  - arithmetic shift a specific register right by 1
    SLA(), // shift left arithmetic   - arithmetic shift a specific register left by 1
    SWAP(), // swap nibbles            - switch upper and lower nibble of a specific register
    JP(JumpKind), // jump based on flags     -
    JR(),  // jump base on pc         -
    JPI(), // jump to [HI]            -
    LD(LoadKind), //
    PUSH(RegKind),
    POP(RegKind),
}

impl Instruction {
    pub fn from_opcode(opcode: &Opcode) -> Option<Instruction> {
        match opcode.extended {
            true => Self::from_byte_ext(opcode.byte),
            false => Self::from_byte(opcode.byte),
        }
    }

    fn from_byte(byte: u8) -> Option<Instruction> {
        /*
        match byte {
                0xB1 => Instruction::OR(A, C),
                0x47 => Instruction::LD(B, A),
                0x24 => Instruction::INC(H),
                0x09 => Instruction::ADD(HL, BC),
                0x6F => Instruction::LD(L, A),
                0x60 => Instruction::LD(H, B),
                0xB7 => Instruction::OR(A, A),
                0x39 => Instruction::ADD(HL, SP),
                0xE0 => Instruction::LDH(a8, A),
                0x01 => Instruction::LD(BC, n16),
                0xEA => Instruction::LD(a16, A),
                0x79 => Instruction::LD(A, C),
                0xDA => Instruction::JP(C, a16),
                0x7A => Instruction::LD(A, D),
                0x1F => Instruction::RRA(),
                0x4F => Instruction::LD(C, A),
                0xBC => Instruction::CP(A, H),
                0xFA => Instruction::LD(A, a16),
                0xC6 => Instruction::ADD(A, n8),
                0x6E => Instruction::LD(L, HL),
                0x73 => Instruction::LD(HL, E),
                0x58 => Instruction::LD(E, B),
                0x6B => Instruction::LD(L, E),
                0x43 => Instruction::LD(B, E),
                0xF1 => Instruction::POP(AF),
                0x5E => Instruction::LD(E, HL),
                0x36 => Instruction::LD(HL, n8),
                0x80 => Instruction::ADD(A, B),
                0xCC => Instruction::CALL(Z, a16),
                0x10 => Instruction::STOP(n8),
                0xE8 => Instruction::ADD(SP, e8),
                0x15 => Instruction::DEC(D),
                0x1A => Instruction::LD(A, DE),
                0xA0 => Instruction::AND(A, B),
                0xDF => Instruction::RST(18),
                0x5F => Instruction::LD(E, A),
                0xAF => Instruction::XOR(A, A),
                0xD0 => Instruction::RET(NC),
                0xA1 => Instruction::AND(A, C),
                0xB2 => Instruction::OR(A, D),
                0xEB => Instruction::ILLEGAL_EB(),
                0x85 => Instruction::ADD(A, L),
                0xD2 => Instruction::JP(NC, a16),
                0xA8 => Instruction::XOR(A, B),
                0x8C => Instruction::ADC(A, H),
                0x92 => Instruction::SUB(A, D),
                0xFE => Instruction::CP(A, n8),
                0x14 => Instruction::INC(D),
                0x35 => Instruction::DEC(HL),
                0x20 => Instruction::JR(NZ, e8),
                0x3E => Instruction::LD(A, n8),
                0xF3 => Instruction::DI(),
                0x1D => Instruction::DEC(E),
                0x4C => Instruction::LD(C, H),
                0x75 => Instruction::LD(HL, L),
                0x86 => Instruction::ADD(A, HL),
                0x0C => Instruction::INC(C),
                0xB5 => Instruction::OR(A, L),
                0x57 => Instruction::LD(D, A),
                0xED => Instruction::ILLEGAL_ED(),
                0xD6 => Instruction::SUB(A, n8),
                0xEF => Instruction::RST(28),
                0x08 => Instruction::LD(a16, SP),
                0x49 => Instruction::LD(C, C),
                0x1E => Instruction::LD(E, n8),
                0xC4 => Instruction::CALL(NZ, a16),
                0x81 => Instruction::ADD(A, C),
                0x31 => Instruction::LD(SP, n16),
                0xD7 => Instruction::RST(10),
                0x51 => Instruction::LD(D, C),
                0x54 => Instruction::LD(D, H),
                0x78 => Instruction::LD(A, B),
                0x93 => Instruction::SUB(A, E),
                0xBB => Instruction::CP(A, E),
                0x70 => Instruction::LD(HL, B),
                0x4A => Instruction::LD(C, D),
                0x04 => Instruction::INC(B),
                0xA7 => Instruction::AND(A, A),
                0xAE => Instruction::XOR(A, HL),
                0x45 => Instruction::LD(B, L),
                0xC5 => Instruction::PUSH(BC),
                0x13 => Instruction::INC(DE),
                0xAA => Instruction::XOR(A, D),
                0xA5 => Instruction::AND(A, L),
                0xC2 => Instruction::JP(NZ, a16),
                0x69 => Instruction::LD(L, C),
                0x74 => Instruction::LD(HL, H),
                0x95 => Instruction::SUB(A, L),
                0x3D => Instruction::DEC(A),
                0x12 => Instruction::LD(DE, A),
                0x77 => Instruction::LD(HL, A),
                0xC7 => Instruction::RST(00),
                0xCD => Instruction::CALL(a16),
                0x4D => Instruction::LD(C, L),
                0xE6 => Instruction::AND(A, n8),
                0x68 => Instruction::LD(L, B),
                0x8F => Instruction::ADC(A, A),
                0x83 => Instruction::ADD(A, E),
                0x30 => Instruction::JR(NC, e8),
                0x5B => Instruction::LD(E, E),
                0x3F => Instruction::CCF(),
                0x99 => Instruction::SBC(A, C),
                0x61 => Instruction::LD(H, C),
                0x00 => Instruction::NOP(),
                0x91 => Instruction::SUB(A, C),
                0x9A => Instruction::SBC(A, D),
                0xA4 => Instruction::AND(A, H),
                0x9B => Instruction::SBC(A, E),
                0x65 => Instruction::LD(H, L),
                0xAC => Instruction::XOR(A, H),
                0xDB => Instruction::ILLEGAL_DB(),
                0xB4 => Instruction::OR(A, H),
                0x26 => Instruction::LD(H, n8),
                0x6C => Instruction::LD(L, H),
                0x48 => Instruction::LD(C, B),
                0x2A => Instruction::LD(A, HL),
                0x23 => Instruction::INC(HL),
                0x56 => Instruction::LD(D, HL),
                0xF2 => Instruction::LDH(A, C),
                0xBD => Instruction::CP(A, L),
                0x9F => Instruction::SBC(A, A),
                0xB8 => Instruction::CP(A, B),
                0x33 => Instruction::INC(SP),
                0xCE => Instruction::ADC(A, n8),
                0x89 => Instruction::ADC(A, C),
                0x32 => Instruction::LD(HL, A),
                0x38 => Instruction::JR(C, e8),
                0x84 => Instruction::ADD(A, H),
                0x05 => Instruction::DEC(B),
                0x98 => Instruction::SBC(A, B),
                0xA6 => Instruction::AND(A, HL),
                0xC3 => Instruction::JP(a16),
                0xBE => Instruction::CP(A, HL),
                0x9C => Instruction::SBC(A, H),
                0x4E => Instruction::LD(C, HL),
                0x29 => Instruction::ADD(HL, HL),
                0xA9 => Instruction::XOR(A, C),
                0x2C => Instruction::INC(L),
                0x0E => Instruction::LD(C, n8),
                0x1B => Instruction::DEC(DE),
                0x76 => Instruction::HALT(),
                0x4B => Instruction::LD(C, E),
                0x8A => Instruction::ADC(A, D),
                0x19 => Instruction::ADD(HL, DE),
                0x9E => Instruction::SBC(A, HL),
                0xE3 => Instruction::ILLEGAL_E3(),
                0xFD => Instruction::ILLEGAL_FD(),
                0xE4 => Instruction::ILLEGAL_E4(),
                0xFC => Instruction::ILLEGAL_FC(),
                0xD4 => Instruction::CALL(NC, a16),
                0xA3 => Instruction::AND(A, E),
                0xB0 => Instruction::OR(A, B),
                0xAB => Instruction::XOR(A, E),
                0xFB => Instruction::EI(),
                0x21 => Instruction::LD(HL, n16),
                0xAD => Instruction::XOR(A, L),
                0x25 => Instruction::DEC(H),
                0x9D => Instruction::SBC(A, L),
                0xC1 => Instruction::POP(BC),
                0x8E => Instruction::ADC(A, HL),
                0x02 => Instruction::LD(BC, A),
                0xCA => Instruction::JP(Z, a16),
                0x63 => Instruction::LD(H, E),
                0xD8 => Instruction::RET(C),
                0xE2 => Instruction::LDH(C, A),
                0x50 => Instruction::LD(D, B),
                0xB6 => Instruction::OR(A, HL),
                0xDE => Instruction::SBC(A, n8),
                0xF0 => Instruction::LDH(A, a8),
                0xD5 => Instruction::PUSH(DE),
                0xC9 => Instruction::RET(),
                0x2B => Instruction::DEC(HL),
                0xF5 => Instruction::PUSH(AF),
                0x34 => Instruction::INC(HL),
                0xF9 => Instruction::LD(SP, HL),
                0x1C => Instruction::INC(E),
                0xA2 => Instruction::AND(A, D),
                0xBA => Instruction::CP(A, D),
                0x18 => Instruction::JR(e8),
                0x3A => Instruction::LD(A, HL),
                0x42 => Instruction::LD(B, D),
                0x2E => Instruction::LD(L, n8),
                0x7B => Instruction::LD(A, E),
                0xD1 => Instruction::POP(DE),
                0x16 => Instruction::LD(D, n8),
                0xFF => Instruction::RST(38),
                0xE7 => Instruction::RST(20),
                0xB3 => Instruction::OR(A, E),
                0x0D => Instruction::DEC(C),
                0x5C => Instruction::LD(E, H),
                0x6A => Instruction::LD(L, D),
                0xCB => Instruction::PREFIX(),
                0xF7 => Instruction::RST(30),
                0xD9 => Instruction::RETI(),
                0x52 => Instruction::LD(D, D),
                0x17 => Instruction::RLA(),
                0x87 => Instruction::ADD(A, A),
                0x97 => Instruction::SUB(A, A),
                0x6D => Instruction::LD(L, L),
                0x11 => Instruction::LD(DE, n16),
                0x5A => Instruction::LD(E, D),
                0x2F => Instruction::CPL(),
                0x8D => Instruction::ADC(A, L),
                0x94 => Instruction::SUB(A, H),
                0x06 => Instruction::LD(B, n8),
                0x37 => Instruction::SCF(),
                0x27 => Instruction::DAA(),
                0x55 => Instruction::LD(D, L),
                0x66 => Instruction::LD(H, HL),
                0x72 => Instruction::LD(HL, D),
                0x8B => Instruction::ADC(A, E),
                0x82 => Instruction::ADD(A, D),
                0xCF => Instruction::RST(08),
                0xDD => Instruction::ILLEGAL_DD(),
                0x67 => Instruction::LD(H, A),
                0x90 => Instruction::SUB(A, B),
                0x22 => Instruction::LD(HL, A),
                0x53 => Instruction::LD(D, E),
                0x3B => Instruction::DEC(SP),
                0xE9 => Instruction::JP(HL),
                0x7E => Instruction::LD(A, HL),
                0xEE => Instruction::XOR(A, n8),
                0x0F => Instruction::RRCA(),
                0xF4 => Instruction::ILLEGAL_F4(),
                0x5D => Instruction::LD(E, L),
                0x7C => Instruction::LD(A, H),
                0xBF => Instruction::CP(A, A),
                0x46 => Instruction::LD(B, HL),
                0x0A => Instruction::LD(A, BC),
                0x44 => Instruction::LD(B, H),
                0xC0 => Instruction::RET(NZ),
                0x64 => Instruction::LD(H, H),
                0xB9 => Instruction::CP(A, C),
                0x03 => Instruction::INC(BC),
                0x62 => Instruction::LD(H, D),
                0x71 => Instruction::LD(HL, C),
                0x7F => Instruction::LD(A, A),
                0x88 => Instruction::ADC(A, B),
                0xC8 => Instruction::RET(Z),
                0x07 => Instruction::RLCA(),
                0x41 => Instruction::LD(B, C),
                0x2D => Instruction::DEC(L),
                0xF8 => Instruction::LD(HL, SP, e8),
                0x28 => Instruction::JR(Z, e8),
                0x3C => Instruction::INC(A),
                0xD3 => Instruction::ILLEGAL_D3(),
                0x40 => Instruction::LD(B, B),
                0xE1 => Instruction::POP(HL),
                0xDC => Instruction::CALL(C, a16),
                0x7D => Instruction::LD(A, L),
                0xEC => Instruction::ILLEGAL_EC(),
                0x96 => Instruction::SUB(A, HL),
                0x59 => Instruction::LD(E, C),
                0x0B => Instruction::DEC(BC),
                0xE5 => Instruction::PUSH(HL),
                0xF6 => Instruction::OR(A, n8),
            }
        }
        */
        todo!()
    }

    fn from_byte_ext(byte: u8) -> Option<Instruction> {
        /*
        match byte {
            0x89 => Instruction::RES(1, C),
            0x6D => Instruction::BIT(5, L),
            0x75 => Instruction::BIT(6, L),
            0x19 => Instruction::RR(C),
            0x49 => Instruction::BIT(1, C),
            0x69 => Instruction::BIT(5, C),
            0x31 => Instruction::SWAP(C),
            0x78 => Instruction::BIT(7, B),
            0x4A => Instruction::BIT(1, D),
            0x61 => Instruction::BIT(4, C),
            0x71 => Instruction::BIT(6, C),
            0xB1 => Instruction::RES(6, C),
            0xE5 => Instruction::SET(4, L),
            0xFF => Instruction::SET(7, A),
            0x9A => Instruction::RES(3, D),
            0x20 => Instruction::SLA(B),
            0x42 => Instruction::BIT(0, D),
            0xF4 => Instruction::SET(6, H),
            0x54 => Instruction::BIT(2, H),
            0xAD => Instruction::RES(5, L),
            0x47 => Instruction::BIT(0, A),
            0x24 => Instruction::SLA(H),
            0xDA => Instruction::SET(3, D),
            0x29 => Instruction::SRA(C),
            0x1B => Instruction::RR(E),
            0x2E => Instruction::SRA(HL),
            0x26 => Instruction::SLA(HL),
            0x5D => Instruction::BIT(3, L),
            0xE1 => Instruction::SET(4, C),
            0xA4 => Instruction::RES(4, H),
            0xF7 => Instruction::SET(6, A),
            0x95 => Instruction::RES(2, L),
            0xD7 => Instruction::SET(2, A),
            0x6E => Instruction::BIT(5, HL),
            0xC7 => Instruction::SET(0, A),
            0xBA => Instruction::RES(7, D),
            0x8B => Instruction::RES(1, E),
            0x41 => Instruction::BIT(0, C),
            0xBF => Instruction::RES(7, A),
            0x00 => Instruction::RLC(B),
            0xC2 => Instruction::SET(0, D),
            0x17 => Instruction::RL(A),
            0x7D => Instruction::BIT(7, L),
            0x3F => Instruction::SRL(A),
            0x74 => Instruction::BIT(6, H),
            0x12 => Instruction::RL(D),
            0x80 => Instruction::RES(0, B),
            0xAF => Instruction::RES(5, A),
            0xBE => Instruction::RES(7, HL),
            0xCE => Instruction::SET(1, HL),
            0x0C => Instruction::RRC(H),
            0x0E => Instruction::RRC(HL),
            0x04 => Instruction::RLC(H),
            0xE4 => Instruction::SET(4, H),
            0x3E => Instruction::SRL(HL),
            0x7F => Instruction::BIT(7, A),
            0x1A => Instruction::RR(D),
            0xCA => Instruction::SET(1, D),
            0x90 => Instruction::RES(2, B),
            0xCC => Instruction::SET(1, H),
            0xE8 => Instruction::SET(5, B),
            0x65 => Instruction::BIT(4, L),
            0x36 => Instruction::SWAP(HL),
            0x76 => Instruction::BIT(6, HL),
            0xE9 => Instruction::SET(5, C),
            0x1D => Instruction::RR(L),
            0x2A => Instruction::SRA(D),
            0x56 => Instruction::BIT(2, HL),
            0x7C => Instruction::BIT(7, H),
            0x43 => Instruction::BIT(0, E),
            0x14 => Instruction::RL(H),
            0x1C => Instruction::RR(H),
            0x55 => Instruction::BIT(2, L),
            0x5F => Instruction::BIT(3, A),
            0x66 => Instruction::BIT(4, HL),
            0x82 => Instruction::RES(0, D),
            0x5C => Instruction::BIT(3, H),
            0xA2 => Instruction::RES(4, D),
            0xB0 => Instruction::RES(6, B),
            0xC8 => Instruction::SET(1, B),
            0xCB => Instruction::SET(1, E),
            0xE2 => Instruction::SET(4, D),
            0xEC => Instruction::SET(5, H),
            0xF0 => Instruction::SET(6, B),
            0x18 => Instruction::RR(B),
            0xF1 => Instruction::SET(6, C),
            0x86 => Instruction::RES(0, HL),
            0xF2 => Instruction::SET(6, D),
            0x64 => Instruction::BIT(4, H),
            0x91 => Instruction::RES(2, C),
            0xC5 => Instruction::SET(0, L),
            0xF8 => Instruction::SET(7, B),
            0x72 => Instruction::BIT(6, D),
            0xA1 => Instruction::RES(4, C),
            0xB7 => Instruction::RES(6, A),
            0xD3 => Instruction::SET(2, E),
            0x83 => Instruction::RES(0, E),
            0x88 => Instruction::RES(1, B),
            0x9C => Instruction::RES(3, H),
            0xAC => Instruction::RES(5, H),
            0xC4 => Instruction::SET(0, H),
            0x53 => Instruction::BIT(2, E),
            0x7A => Instruction::BIT(7, D),
            0xAE => Instruction::RES(5, HL),
            0x21 => Instruction::SLA(C),
            0x8E => Instruction::RES(1, HL),
            0x28 => Instruction::SRA(B),
            0xB9 => Instruction::RES(7, C),
            0xDB => Instruction::SET(3, E),
            0xA7 => Instruction::RES(4, A),
            0x9D => Instruction::RES(3, L),
            0xFE => Instruction::SET(7, HL),
            0x25 => Instruction::SLA(L),
            0xE0 => Instruction::SET(4, B),
            0x05 => Instruction::RLC(L),
            0x5A => Instruction::BIT(3, D),
            0x32 => Instruction::SWAP(D),
            0xD1 => Instruction::SET(2, C),
            0xEE => Instruction::SET(5, HL),
            0x3D => Instruction::SRL(L),
            0x81 => Instruction::RES(0, C),
            0x7E => Instruction::BIT(7, HL),
            0x11 => Instruction::RL(C),
            0x62 => Instruction::BIT(4, D),
            0x99 => Instruction::RES(3, C),
            0xD0 => Instruction::SET(2, B),
            0xF3 => Instruction::SET(6, E),
            0xFC => Instruction::SET(7, H),
            0x67 => Instruction::BIT(4, A),
            0x45 => Instruction::BIT(0, L),
            0x97 => Instruction::RES(2, A),
            0xA0 => Instruction::RES(4, B),
            0xD9 => Instruction::SET(3, C),
            0x58 => Instruction::BIT(3, B),
            0x30 => Instruction::SWAP(B),
            0xD6 => Instruction::SET(2, HL),
            0x2F => Instruction::SRA(A),
            0xF5 => Instruction::SET(6, L),
            0xB6 => Instruction::RES(6, HL),
            0xBB => Instruction::RES(7, E),
            0x07 => Instruction::RLC(A),
            0x3B => Instruction::SRL(E),
            0x48 => Instruction::BIT(1, B),
            0xC1 => Instruction::SET(0, C),
            0x27 => Instruction::SLA(A),
            0x6F => Instruction::BIT(5, A),
            0x6B => Instruction::BIT(5, E),
            0x84 => Instruction::RES(0, H),
            0x15 => Instruction::RL(L),
            0x70 => Instruction::BIT(6, B),
            0x06 => Instruction::RLC(HL),
            0xFB => Instruction::SET(7, E),
            0x4F => Instruction::BIT(1, A),
            0x9E => Instruction::RES(3, HL),
            0x1F => Instruction::RR(A),
            0x2C => Instruction::SRA(H),
            0x23 => Instruction::SLA(E),
            0xEB => Instruction::SET(5, E),
            0xC0 => Instruction::SET(0, B),
            0x63 => Instruction::BIT(4, E),
            0xAB => Instruction::RES(5, E),
            0x9F => Instruction::RES(3, A),
            0xAA => Instruction::RES(5, D),
            0xA9 => Instruction::RES(5, C),
            0x40 => Instruction::BIT(0, B),
            0xE3 => Instruction::SET(4, E),
            0x08 => Instruction::RRC(B),
            0xD5 => Instruction::SET(2, L),
            0xEF => Instruction::SET(5, A),
            0x39 => Instruction::SRL(C),
            0x8D => Instruction::RES(1, L),
            0x98 => Instruction::RES(3, B),
            0x60 => Instruction::BIT(4, B),
            0x2B => Instruction::SRA(E),
            0x94 => Instruction::RES(2, H),
            0xF9 => Instruction::SET(7, C),
            0xEA => Instruction::SET(5, D),
            0x03 => Instruction::RLC(E),
            0x5E => Instruction::BIT(3, HL),
            0x3C => Instruction::SRL(H),
            0x8F => Instruction::RES(1, A),
            0x38 => Instruction::SRL(B),
            0x77 => Instruction::BIT(6, A),
            0xA8 => Instruction::RES(5, B),
            0xED => Instruction::SET(5, L),
            0x1E => Instruction::RR(HL),
            0x2D => Instruction::SRA(L),
            0x35 => Instruction::SWAP(L),
            0x8C => Instruction::RES(1, H),
            0x8A => Instruction::RES(1, D),
            0xE6 => Instruction::SET(4, HL),
            0x51 => Instruction::BIT(2, C),
            0xB5 => Instruction::RES(6, L),
            0xDD => Instruction::SET(3, L),
            0x37 => Instruction::SWAP(A),
            0x9B => Instruction::RES(3, E),
            0xFD => Instruction::SET(7, L),
            0x92 => Instruction::RES(2, D),
            0x09 => Instruction::RRC(C),
            0xD2 => Instruction::SET(2, D),
            0xA6 => Instruction::RES(4, HL),
            0x0A => Instruction::RRC(D),
            0x22 => Instruction::SLA(D),
            0xB4 => Instruction::RES(6, H),
            0x7B => Instruction::BIT(7, E),
            0xC3 => Instruction::SET(0, E),
            0xB8 => Instruction::RES(7, B),
            0xF6 => Instruction::SET(6, HL),
            0x34 => Instruction::SWAP(H),
            0xC6 => Instruction::SET(0, HL),
            0x93 => Instruction::RES(2, E),
            0x6A => Instruction::BIT(5, D),
            0x02 => Instruction::RLC(D),
            0x79 => Instruction::BIT(7, C),
            0x10 => Instruction::RL(B),
            0x46 => Instruction::BIT(0, HL),
            0x6C => Instruction::BIT(5, H),
            0xD4 => Instruction::SET(2, H),
            0x52 => Instruction::BIT(2, D),
            0x4D => Instruction::BIT(1, L),
            0x96 => Instruction::RES(2, HL),
            0xBD => Instruction::RES(7, L),
            0x01 => Instruction::RLC(C),
            0x3A => Instruction::SRL(D),
            0xB2 => Instruction::RES(6, D),
            0x13 => Instruction::RL(E),
            0xFA => Instruction::SET(7, D),
            0x85 => Instruction::RES(0, L),
            0x87 => Instruction::RES(0, A),
            0x16 => Instruction::RL(HL),
            0xA3 => Instruction::RES(4, E),
            0xB3 => Instruction::RES(6, E),
            0x59 => Instruction::BIT(3, C),
            0xD8 => Instruction::SET(3, B),
            0x33 => Instruction::SWAP(E),
            0x73 => Instruction::BIT(6, E),
            0xBC => Instruction::RES(7, H),
            0xDF => Instruction::SET(3, A),
            0xA5 => Instruction::RES(4, L),
            0xDE => Instruction::SET(3, HL),
            0xCD => Instruction::SET(1, L),
            0x68 => Instruction::BIT(5, B),
            0xCF => Instruction::SET(1, A),
            0x44 => Instruction::BIT(0, H),
            0x5B => Instruction::BIT(3, E),
            0xE7 => Instruction::SET(4, A),
            0x4C => Instruction::BIT(1, H),
            0xDC => Instruction::SET(3, H),
            0x50 => Instruction::BIT(2, B),
            0x4B => Instruction::BIT(1, E),
            0x0D => Instruction::RRC(L),
            0x0B => Instruction::RRC(E),
            0xC9 => Instruction::SET(1, C),
            0x4E => Instruction::BIT(1, HL),
            0x0F => Instruction::RRC(A),
            0x57 => Instruction::BIT(2, A),
        }
        */
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use std::path;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    struct Flags {
        z: char,
        n: char,
        h: char,
        c: char,
    }

    #[derive(Debug, serde::Deserialize)]
    struct Operand {
        name: String,
        immediate: bool,
        bytes: Option<u8>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct Instruction {
        mnemonic: String,
        bytes: u8,
        operands: Vec<Operand>,
        immediate: bool,
        flags: Flags,
    }

    #[derive(Debug, serde::Deserialize)]
    struct Mapping {
        unprefixed: HashMap<String, Instruction>,
        cbprefixed: HashMap<String, Instruction>,
    }

    fn write_instr_set(out: &mut fs::File, hm: HashMap<String, Instruction>) {
        write!(out, "match 0 {{");
        for (opcode, instr) in hm {
            let operands = instr
                .operands
                .iter()
                .map(|op| {
                    if op.name.starts_with("$") {
                        &op.name[1..]
                    } else {
                        op.name.as_str()
                    }
                })
                .collect::<Vec<_>>()
                .join(",");

            write!(
                out,
                "{} => Instruction::{}({}),",
                opcode, instr.mnemonic, operands
            );
        }
        write!(out, "}}");
    }

    #[test]
    fn parse_instructions() {
        let root = path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let json_path = root.join("instructions.json");
        let json = fs::read_to_string(json_path).unwrap();
        let instructions: Mapping = serde_json::from_str(&json).unwrap();

        let outname = root.join("instructions.rs");
        let mut out = fs::File::create(&outname).unwrap();

        writeln!(out, "fn main() {{");
        write_instr_set(&mut out, instructions.unprefixed);
        writeln!(out);
        write_instr_set(&mut out, instructions.cbprefixed);
        writeln!(out, "}}");
    }
}
