#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use crate::cpu::*;
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
    #[derive(Debug, Clone)]
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

    impl Mem {
        fn read(&self, cpu: &mut Cpu) -> u16 {
            match self {
                &Mem::Reg8(reg) => cpu.regs.read8(reg) as u16,
                &Mem::Reg16(reg) => cpu.regs.read16(reg),
                Mem::SP => cpu.regs.sp(),
                Mem::Imm8 => cpu.fetch8() as u16,
                Mem::Imm8S => cpu.fetch8() as u16,
                Mem::Imm16 => cpu.fetch16(),
                Mem::Addr8(mem) => {
                    let addr = mem.read(cpu);
                    cpu.bus.read8(addr) as u16
                }
                Mem::Addr16(mem) => {
                    let addr = mem.read(cpu);
                    let lo = cpu.bus.read8(addr);
                    let hi = cpu.bus.read8(addr.wrapping_add(1));
                    u16::from_le_bytes([lo, hi])
                }
                &Mem::Flag(flag) => cpu.regs.flag(flag) as u16,
                &Mem::NoFlag(flag) => !cpu.regs.flag(flag) as u16,
                Mem::Bit(bit, mem) => mem.read(cpu) & (1 << bit),
                Mem::Rst(_) => todo!(),
                Mem::HLI => {
                    let value = Mem::Reg16(Reg16::HL).read(cpu);
                    Mem::Reg16(Reg16::HL).write(cpu, value.wrapping_add(1));

                    value
                }
                Mem::HLD => {
                    let value = Mem::Reg16(Reg16::HL).read(cpu);
                    Mem::Reg16(Reg16::HL).write(cpu, value.wrapping_sub(1));

                    value
                }
                Mem::SPMod(off) => {
                    let off = off.as_ref().unwrap().read(cpu);
                    let value = Mem::SP.read(cpu);

                    value.wrapping_add(off)
                }
            }
        }

        fn write(&self, cpu: &mut Cpu, value: u16) {
            match self {
                &Mem::Reg8(reg) => cpu.regs.write8(reg, value as u8),
                &Mem::Reg16(reg) => cpu.regs.write16(reg, value),
                Mem::SP => cpu.regs.set_sp(value),
                Mem::Imm8 | Mem::Imm16 | Mem::Imm8S => {
                    panic!("Can't write to immediate value");
                }
                Mem::Addr8(mem) => {
                    let addr = mem.read(cpu);
                    let [lo, _] = u16::to_le_bytes(value);
                    cpu.bus.write8(addr, lo);
                }
                Mem::Addr16(mem) => {
                    let addr = mem.read(cpu);
                    let [lo, hi] = u16::to_le_bytes(value);
                    cpu.bus.write8(addr, lo);
                    cpu.bus.write8(addr.wrapping_add(1), hi);
                }
                &Mem::Flag(flag) => {
                    cpu.regs.set_flag(flag, true);
                }
                Mem::Bit(bit, mem) => {
                    let old = mem.read(cpu);
                    mem.write(cpu, old | (1 << bit));
                }
                Mem::NoFlag(_) => unreachable!(),
                Mem::Rst(_) => todo!(),
                Mem::HLI | Mem::HLD | Mem::SPMod(_) => {
                    panic!("Can't write directly to {self:?}");
                }
            }
        }

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
            "F" => Mem::Reg8(Reg8::F),
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
            let mut count_ops = self.operands.len();

            let ops_s = self
                .operands
                .iter()
                .map(|op| format!("{:?}", operand_to_mem(op, self, &mut count_ops)))
                .collect::<Vec<_>>();

            write!(
                f,
                "Instruction::{}({})",
                self.mnemonic,
                ops_s[..count_ops].join(",")
            )
        }
    }

    fn join_bit_index_operands(map: &mut HashMap<String, Instruction>) {
        for instr in map.values_mut() {
            if instr.operands.len() == 2 {
                if let Ok(bit_index) = instr.operands[0].name.parse::<u8>() {
                    if bit_index < 8 {
                        let mut reg = instr.operands[1].clone();
                        reg.bit_index = Some(bit_index);
                        reg.bytes = Some(1);
                        instr.operands = vec![reg]
                    }
                }
            }
        }
    }

    fn codegen_fn<'a>(
        file_out: &mut fs::File,
        fnname: &str,
        map: impl IntoIterator<Item = (&'a String, &'a Instruction)>,
    ) {
        writeln!(file_out, "fn {}(code: u8) {{", fnname).unwrap();
        write!(file_out, "match code {{").unwrap();
        for (opcode, instr) in map {
            //let opcode = format!("0b{:0<8b}", u8::from_str_radix(&opcode[2..], 16).unwrap());
            write!(file_out, "{opcode} => {instr:?},").unwrap();
        }
        writeln!(file_out, "}}").unwrap();
        writeln!(file_out, "}}").unwrap();
    }

    fn codegen(file_out: &mut fs::File, optables: &OpTable) {
        codegen_fn(file_out, "unprefixed", &optables.unprefixed);
        codegen_fn(file_out, "cbprefixed", &optables.cbprefixed);
    }

    fn sorted_by_mnemonic(hmap: &HashMap<String, Instruction>) -> Vec<(&String, &Instruction)> {
        let mut items = hmap.iter().collect::<Vec<(_, _)>>();
        items.sort_by(|item1, item2| item1.1.mnemonic.cmp(&item2.1.mnemonic));

        //panic!("{items:?}");

        items
    }

    fn codegen_sorted(file_out: &mut fs::File, optables: &OpTable) {
        codegen_fn(
            file_out,
            "unprefixed",
            sorted_by_mnemonic(&optables.unprefixed),
        );
        codegen_fn(
            file_out,
            "cbprefixed",
            sorted_by_mnemonic(&optables.cbprefixed),
        );
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
        let mut file_out = fs::File::create(root.join(codegen_base)).unwrap();
        codegen(&mut file_out, &optable);

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

        let mut file_out =
            fs::File::create(root.join(format!("{filter_name}_{codegen_base}"))).unwrap();
        codegen_sorted(&mut file_out, &filtered);
    }
}
