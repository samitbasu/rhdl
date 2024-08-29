use std::collections::BTreeMap;

use crate::RHDLError;

use super::{
    object::BitString,
    spec::{LiteralId, Operand},
    Object,
};

type Result<T> = core::result::Result<T, RHDLError>;

struct VMState<'a> {
    reg_stack: &'a mut [Option<BitString>],
    literals: &'a BTreeMap<LiteralId, BitString>,
    obj: &'a Object,
}

impl<'a> VMState<'a> {
    fn read(&self, operand: Operand) -> Result<BitString> {
        match operand {
            Operand::Literal(l) => Ok(self.literals[&l].clone()),
            Operand::Register(r) => self.reg_stack[r.0]
                .clone()
                .ok_or(anyhow!("ICE Register {r:?} is not initialized")),
        }
    }
    fn write(&mut self, operand: Operand, value: BitString) -> Result<()> {
        match operand {
            Operand::Literal(_) => 
        }
    }
}
