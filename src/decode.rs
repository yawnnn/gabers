#![allow(unused)]

use crate::cpu::Cpu;

impl Cpu {
    pub fn exec_next_instr(&mut self) {
        self.fetch8();
        todo!()
    }

    fn exec_next_instr_prefixed(&mut self) {
        self.fetch8();
        todo!()
    }
}