#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use crate::registers::*;
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::fs;
    use std::io::Write;
    use std::path;

    #[derive(Clone)]
    pub struct HexU8(u8);

    impl TryFrom<&str> for HexU8 {
        type Error = std::num::ParseIntError;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            u8::from_str_radix(value, 16).map(HexU8)
        }
    }

    impl std::fmt::Debug for HexU8 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:X}", self.0)
        }
    }

    #[derive(Debug, Clone)]
    #[allow(clippy::upper_case_acronyms)]
    pub enum Mod {
        INC,
        DEC,
    }

    #[allow(clippy::upper_case_acronyms)]
    #[derive(Clone)]
    pub enum Mem {
        Reg8(Reg8),
        Reg16(Reg16),
        SP,
        Imm8,
        Imm16,
        Imm8S,
        Addr8(Box<Mem>),
        Addr16(Box<Mem>),
        Flag(Flag),
        NoFlag(Flag),
        Rst(HexU8),
        Bit(u8, Box<Mem>),
        HLI,
        HLD,
        SPMod(Option<Box<Mem>>),
    }

    impl Debug for Mem {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Mem::Reg8(r) => write!(f, "{r:?}"),
                Mem::Reg16(r) => write!(f, "{r:?}"),
                Mem::SP => write!(f, "SP"),
                Mem::Imm8 => write!(f, "Imm8"),
                Mem::Imm16 => write!(f, "Imm16"),
                Mem::Imm8S => write!(f, "Imm8S"),
                Mem::Addr8(m) => write!(f, "Addr::{m:?}"),
                Mem::Addr16(m) => write!(f, "Addr::{m:?}"),
                Mem::Flag(flg) => write!(f, "{flg:?}"),
                Mem::NoFlag(flg) => write!(f, "No{flg:?}"),
                Mem::Rst(addr) => write!(f, "{addr:?}"),
                Mem::Bit(bit, m) => write!(f, "{bit}, {m:?}"),
                Mem::HLI => write!(f, "Addr::HLI"),
                Mem::HLD => write!(f, "Addr::HLD"),
                Mem::SPMod(_) => write!(f, "SP"),
            }
        }
    }

    impl Mem {
        pub fn size(&self) -> u8 {
            match self {
                Mem::Reg8(_)
                | Mem::Imm8
                | Mem::Imm8S
                | Mem::Addr8(_)
                | Mem::Addr16(_)
                | Mem::Flag(_)
                | Mem::NoFlag(_)
                | Mem::Bit(..)
                | Mem::HLI
                | Mem::HLD
                | Mem::SPMod(_) => 1,
                Mem::Reg16(_) | Mem::Imm16 | Mem::SP => 2,
                Mem::Rst(_) => 1,
            }
        }
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    struct Flags {
        z: char,
        n: char,
        h: char,
        c: char,
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct Operand {
        name: String,
        immediate: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        bytes: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bit_index: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        increment: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        decrement: Option<bool>,
    }

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    struct Instruction {
        #[serde(skip_serializing_if = "short_serialize")]
        mnemonic: String,
        #[serde(skip_serializing_if = "short_serialize")]
        bytes: u8,
        operands: Vec<Operand>,
        immediate: bool,
        #[serde(skip_serializing)]
        flags: Flags,
    }

    impl Instruction {
        fn takes_flag_cond(&self) -> bool {
            ["CALL", "JP", "JR", "RET"].contains(&self.mnemonic.as_ref())
        }

        fn operands_as_mem(&self) -> Vec<Mem> {
            let mut count_ops = self.operands.len();
            let v = self
                .operands
                .iter()
                .map(|op| operand_to_mem(op, self, &mut count_ops))
                .collect::<Vec<_>>();
            v.into_iter().take(count_ops).collect::<Vec<_>>()
        }

        fn fnname(&self) -> String {
            let mems = self.operands_as_mem();
            let dim8 = mems.iter().all(|m| m.size() == 1);
            let dim16 = mems.iter().all(|m| m.size() == 2);
            let is_cc = if !mems.is_empty() {
                matches!(mems[0], Mem::Flag(_) | Mem::NoFlag(_))
            } else {
                false
            };

            let by_size = |base| {
                if dim8 {
                    format!("{}8", base)
                } else if dim16 {
                    format!("{}16", base)
                } else {
                    panic!("Unexpected {self:?}");
                }
            };

            let fnname = match self.mnemonic.as_str() {
                "LD" => {
                    if mems.len() == 2
                        && let Mem::SPMod(_) = mems[1]
                    {
                        "load16_hl_sp_e8"
                    } else if mems.len() == 2
                        && let Mem::SP = mems[1]
                    {
                        "load16_imm_sp"
                    } else if mems.len() == 2
                        && mems.iter().all(|m| matches!(m, Mem::Reg8(Reg8::B)))
                    {
                        "load8_b_b"
                    } else if mems.len() == 2
                        && mems.iter().all(|m| matches!(m, Mem::Reg8(Reg8::D)))
                    {
                        "load8_d_d"
                    } else {
                        &by_size("load")
                    }
                }
                "LDH" => {
                    if let Mem::Addr8(_) = mems[0] {
                        "ldh8_a_addr"
                    } else {
                        "ldh8_addr_a"
                    }
                }
                "ADC" => "adc8",
                "ADD" => {
                    if let Mem::SP = mems[0] {
                        "add16_sp_e"
                    } else {
                        &by_size("add")
                    }
                }
                "CP" => "cmp8",
                "DEC" => &by_size("dec"),
                "INC" => &by_size("inc"),
                "SBC" => "sbc8",
                "SUB" => "sub8",
                "AND" => "and8",
                "CPL" => "cpl",
                "OR" => "or8",
                "XOR" => "xor8",
                "BIT" => "bit8",
                "RES" => "res8",
                "SET" => "set8",
                "RL" => "rl8",
                "RLA" => "rla",
                "RLC" => "rlc8",
                "RLCA" => "rlca",
                "RR" => "rr8",
                "RRA" => "rra",
                "RRC" => "rrc8",
                "RRCA" => "rrca",
                "SLA" => "sla8",
                "SRA" => "sra8",
                "SRL" => "srl8",
                "SWAP" => "swap8",
                "CALL" => {
                    if is_cc {
                        "call16_cc"
                    } else {
                        "call16"
                    }
                }
                "JP" => {
                    if is_cc {
                        "jump16_cc"
                    } else {
                        "jump16"
                    }
                }
                "JR" => {
                    if is_cc {
                        "jumpr16_cc"
                    } else {
                        "jumpr16"
                    }
                }
                "RET" => {
                    if is_cc {
                        "ret_cc"
                    } else {
                        "ret"
                    }
                }
                "CCF" => "ccf",
                "SCF" => "scf",
                "POP" => "pop16",
                "PUSH" => "push16",
                "NOP" => "noop",
                "STOP" => "stop",
                // TODO:
                "RETI" => "todo!(), //reti",
                "RST" => "todo!(), //rst",
                "DI" => "todo!(), //di",
                "EI" => "todo!(), //ei",
                "HALT" => "todo!(), //halt",
                "DAA" => "todo!(), //daa",
                _ => "todo!",
            };

            if !fnname.starts_with("todo") {
                format!("self.{fnname}")
            } else {
                fnname.into()
            }
        }
    }

    #[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
    struct OpTable {
        unprefixed: HashMap<String, Instruction>,
        cbprefixed: HashMap<String, Instruction>,
    }

    thread_local! {
        static SHORT_SERIALIZE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    fn short_serialize<T>(_t: &T) -> bool {
        SHORT_SERIALIZE.get()
    }

    fn operand_to_mem(op: &Operand, instr: &Instruction, pcount_ops: &mut usize) -> Mem {
        let mut done_inc = false;

        let mut mem = match op.name.as_str() {
            "A" => Mem::Reg8(Reg8::A),
            "B" => Mem::Reg8(Reg8::B),
            "C" => {
                if instr.takes_flag_cond() {
                    Mem::Flag(Flag::CF)
                } else {
                    Mem::Reg8(Reg8::C)
                }
            }
            "D" => Mem::Reg8(Reg8::D),
            "E" => Mem::Reg8(Reg8::E),
            "F" => unreachable!(), //Mem::Reg8(Reg8::F),
            "H" => Mem::Reg8(Reg8::H),
            "L" => Mem::Reg8(Reg8::L),

            //"C" => Mem::Flag(Flag::Carry),   todo!() --- distinguish from C register
            "NC" => Mem::NoFlag(Flag::CF),
            "Z" => Mem::Flag(Flag::NF),
            "NZ" => Mem::NoFlag(Flag::NF),

            "AF" => Mem::Reg16(Reg16::AF),
            "BC" => Mem::Reg16(Reg16::BC),
            "DE" => Mem::Reg16(Reg16::DE),
            "HL" => {
                if op.increment.is_some() {
                    done_inc = true;
                    Mem::HLI
                } else if op.decrement.is_some() {
                    done_inc = true;
                    Mem::HLD
                } else {
                    Mem::Reg16(Reg16::HL)
                }
            }
            "SP" => {
                if op.increment.is_some() {
                    done_inc = true;
                    Mem::SPMod(None)
                } else {
                    Mem::SP
                }
            }

            "n8" => Mem::Imm8,
            "e8" => Mem::Imm8S,
            "n16" => Mem::Imm16,
            "a8" => Mem::Addr8(Box::new(Mem::Imm8)),
            "a16" => Mem::Addr16(Box::new(Mem::Imm16)),

            s if s.starts_with("$") => Mem::Rst(s[1..].try_into().unwrap()),

            // todo!() --- WRONG!
            s if s.parse::<u8>().is_ok() => {
                Mem::Bit(s.parse::<u8>().unwrap(), Box::from(Mem::Reg8(Reg8::A)))
            }

            _ => unreachable!("{op:?}"),
        };

        if let Mem::SPMod(None) = mem {
            let idx = instr
                .operands
                .iter()
                .position(|op2| std::ptr::eq(op, op2))
                .unwrap();
            if let Some(opx) = instr.operands.get(idx + 1) {
                // it will be converted but ignored in the end
                *pcount_ops -= 1;
                let memx = operand_to_mem(opx, instr, &mut 0);
                mem = Mem::SPMod(Some(Box::new(memx)));
            }
        }

        if !op.immediate && !matches!(mem, Mem::Addr8(_) | Mem::Addr16(_)) {
            mem = match mem.size() {
                1 => Mem::Addr8(Box::from(mem)),
                2 => Mem::Addr16(Box::from(mem)),
                _ => unreachable!(),
            }
        }

        if (op.increment.is_some() || op.decrement.is_some()) && !done_inc {
            panic!("not <done_inc>; op: {op:?}, mem: {mem:?}");
        }

        // if let Some(size) = op.bytes {
        //     assert!(mem.size() == size, "wrong size: {size}, {mem}")
        // }

        mem
    }

    impl Debug for Instruction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let ops_s = self
                .operands_as_mem()
                .into_iter()
                .map(|mem| format!("{mem:?}"))
                .collect::<Vec<_>>();

            write!(f, "{}({})", self.fnname(), ops_s.join(","))
        }
    }

    fn join_bit_index_operands(map: &mut HashMap<String, Instruction>) {
        for instr in map.values_mut() {
            if instr.operands.len() == 2
                && let Ok(bit_index) = instr.operands[0].name.parse::<u8>()
                && bit_index < 8
            {
                let mut reg = instr.operands[1].clone();
                reg.bit_index = Some(bit_index);
                reg.bytes = Some(1);
                instr.operands = vec![reg]
            }
        }
    }

    fn codegen_match_cases<'a>(
        w: &mut impl std::io::Write,
        optable: impl IntoIterator<Item = (&'a String, &'a Instruction)>,
    ) {
        for (opcode, instr) in optable {
            //let opcode = format!("0b{:0<8b}", u8::from_str_radix(&opcode[2..], 16).unwrap());
            writeln!(w, "{opcode} => {instr:?},").unwrap();
        }
    }

    fn sort_by_mnemonic(optable: &HashMap<String, Instruction>) -> Vec<(&String, &Instruction)> {
        let mut v = optable.iter().collect::<Vec<_>>();
        v.sort_by(|(opcode1, instr1), (opcode2, instr2)| {
            instr1
                .mnemonic
                .cmp(&instr2.mnemonic)
                .then(opcode1.cmp(opcode2))
        });

        v
    }

    fn codegen(w: &mut impl std::io::Write, optables: &OpTable) -> std::io::Result<()> {
        let src = "
use crate::cpu::Cpu;

use crate::registers::{
    Flag::{CF, NF},
    Reg16::{AF, BC, DE, HL},
    Reg8::{A, B, C, D, E, H, L},
    SP,
};
use crate::instructions::Addr;

impl Cpu {
    pub fn decode_exec_instr(&mut self, opcode: u8) {
        match opcode {
            0xCB => {
                let opcode_cb = self.fetch8();
                self.decode_exec_instr_cb(opcode_cb);
            }
        ";
        write!(w, "{src}")?;
        codegen_match_cases(w, sort_by_mnemonic(&optables.unprefixed));
        let src= "
        }
    }

    pub fn decode_exec_instr_cb(&mut self, opcode_cb: u8) {
        match opcode_cb {
        ";
        write!(w, "{src}")?;
        codegen_match_cases(w, sort_by_mnemonic(&optables.cbprefixed));
        let src = "
        }
    }
}
        ";
        write!(w, "{src}")
    }

    fn filter_istructions(
        filters: &[&str],
        map: &HashMap<String, Instruction>,
    ) -> HashMap<String, Instruction> {
        map.iter()
            .filter(|(_opcode, instr)| filters.contains(&instr.mnemonic.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    // optables provided by https://gbdev.io/gb-opcodes/optables/
    #[test]
    fn codegen_from_optables_json() {
        let root = path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("optable");
        let codegen_base = "opcodes.rs";
        let json_base = "optable.json";

        let file_in = fs::File::open(root.join(json_base)).unwrap();
        let mut optable: OpTable = serde_json::from_reader(file_in).unwrap();

        join_bit_index_operands(&mut optable.unprefixed);
        join_bit_index_operands(&mut optable.cbprefixed);

        // codegen of the code to handle them
        let name_out = root.join(codegen_base);
        let mut file_out = fs::File::create(&name_out).unwrap();
        codegen(&mut file_out, &optable).unwrap();
        file_out.flush().unwrap();
        std::process::Command::new("rustfmt")
            .arg(&name_out)
            .status()
            .expect("failed to format generated code");

        // (debug) do only one category
        SHORT_SERIALIZE.set(true);
        //let filters = "LD";
        let filter_name = "mixed";
        let filters = [
            "LD", "LDH", "ADD", "ADC", "SUB", "AND", "OR", "XOR", "INC", "DEC", "JP", "CALL",
            "RET", "PUSH", "POP",
        ];

        let filtered = OpTable {
            unprefixed: filter_istructions(&filters, &optable.unprefixed),
            cbprefixed: filter_istructions(&filters, &optable.cbprefixed),
        };

        let file_out = fs::File::create(root.join(format!("{filter_name}_{json_base}"))).unwrap();
        serde_json::to_writer(file_out, &filtered).unwrap();
    }
}
