use anyhow::Result;
use std::collections::BTreeMap;

use crate::{
    ast::FunctionId,
    compiler::ty::Ty,
    rhif::rhif_spec::{Block, BlockId, ExternalFunction, Slot},
    TypedBits,
};

#[derive(Debug, Clone)]
pub struct Object {
    pub literals: Vec<TypedBits>,
    pub ty: BTreeMap<Slot, Ty>,
    pub blocks: Vec<Block>,
    pub return_slot: Slot,
    pub externals: Vec<ExternalFunction>,
    pub main_block: BlockId,
    pub arguments: Vec<Slot>,
    pub name: String,
    pub fn_id: FunctionId,
}

impl Object {
    pub fn literal(&self, slot: Slot) -> Result<&TypedBits> {
        match slot {
            Slot::Literal(l) => Ok(&self.literals[l]),
            _ => Err(anyhow::anyhow!("Not a literal")),
        }
    }
    pub fn reg_count(&self) -> usize {
        self.ty
            .keys()
            .filter_map(|slot| match slot {
                Slot::Register(ndx) => Some(ndx),
                _ => None,
            })
            .max()
            .copied()
            .unwrap_or(0)
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Object {}", self.name)?;
        writeln!(f, "  fn_id {}", self.fn_id)?;
        for regs in self.ty.keys() {
            if let Slot::Register(ndx) = regs {
                writeln!(f, "Reg r{} : {}", ndx, self.ty[regs])?;
            }
        }
        for (ndx, literal) in self.literals.iter().enumerate() {
            writeln!(
                f,
                "Literal l{} : {} = {}",
                ndx,
                self.ty[&Slot::Literal(ndx)],
                literal
            )?;
        }
        for (ndx, func) in self.externals.iter().enumerate() {
            writeln!(
                f,
                "Function f{} name: {} code: {} signature: {}",
                ndx, func.path, func.code, func.signature
            )?;
        }
        for block in &self.blocks {
            if block.id == self.main_block {
                writeln!(f, "Main block {}", block.id.0)?;
            } else {
                writeln!(f, "Block {}", block.id.0)?;
            }
            for op in &block.ops {
                writeln!(f, "  {}", op)?;
            }
        }
        Ok(())
    }
}
