use crate::rhdl_core::{
    ast::source::{source_location::SourceLocation, spanned_source_set::SpannedSourceSet},
    ntl::{
        object::{LocatedOpCode, Object},
        spec::{Assign, OpCode, Operand, RegisterId},
    },
};

pub(crate) struct Builder {
    object: Object,
    reg_count: u32,
}

impl Builder {
    pub fn new(name: &str) -> Self {
        Self {
            object: Object {
                name: name.into(),
                ..Default::default()
            },
            reg_count: 0,
        }
    }
    fn reg(&mut self) -> RegisterId {
        let ret = RegisterId::new(self.reg_count);
        self.reg_count += 1;
        ret
    }
    pub fn add_code(&mut self, code: &SpannedSourceSet) {
        self.object.code.extend(code.sources.clone());
    }
    pub fn allocate_reg(&mut self, len: usize) -> Vec<RegisterId> {
        (0..len).map(|_| self.reg()).collect()
    }
    pub fn add_input(&mut self, len: usize) -> Vec<RegisterId> {
        let ret = (0..len).map(|_| self.reg()).collect::<Vec<_>>();
        self.object.inputs.push(ret.clone());
        ret
    }
    pub fn allocate_outputs(&mut self, len: usize) -> Vec<RegisterId> {
        let ret = (0..len).map(|_| self.reg()).collect::<Vec<_>>();
        self.object.outputs = ret.iter().copied().map(Operand::Register).collect();
        ret
    }
    pub fn build(self) -> Object {
        self.object
    }
    pub fn link(&mut self, other: &Object) -> u32 {
        self.add_code(&other.code);
        self.object.link(other)
    }
    pub fn copy_from_to<T: Into<Operand>, S: Into<Operand>>(&mut self, rhs: T, lhs: S) {
        self.object.ops.push(LocatedOpCode {
            loc: None,
            op: OpCode::Assign(Assign {
                lhs: lhs.into(),
                rhs: rhs.into(),
            }),
        })
    }
}
