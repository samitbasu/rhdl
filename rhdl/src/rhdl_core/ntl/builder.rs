use crate::core::types::digital::Digital;
use crate::prelude::Circuit;
use crate::rhdl_core::ntl::spec::{self, Assign, BlackBoxId};
use crate::{
    prelude::{RHDLError, Synchronous},
    rhdl_core::{
        ast::source::spanned_source_set::SpannedSourceSet,
        compiler::optimize_ntl,
        ntl::{
            object::{BlackBox, BlackBoxMode, LocatedOpCode, Object},
            spec::{OpCode, Operand, RegisterId},
        },
    },
};

pub(crate) struct Builder {
    object: Object,
    reg_count: u32,
}

pub enum BuilderMode {
    Asynchronous,
    Synchronous,
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
    pub fn build(mut self, mode: BuilderMode) -> Result<Object, RHDLError> {
        match mode {
            BuilderMode::Asynchronous => {
                if self.object.inputs.is_empty() {
                    self.object.inputs.push(vec![]);
                }
            }
            BuilderMode::Synchronous => {
                if self.object.inputs.is_empty() {
                    self.object.inputs.push(vec![]);
                    self.object.inputs.push(vec![]);
                }
            }
        }
        optimize_ntl(self.object)
    }
    pub fn import(&mut self, other: &Object) -> u32 {
        self.add_code(&other.code);
        self.object.import(other)
    }
    pub fn copy_from_to<T: Into<Operand>, S: Into<Operand>>(&mut self, rhs: T, lhs: S) {
        self.object.ops.push(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: lhs.into(),
                rhs: rhs.into(),
            }),
            loc: None,
        })
    }
}

pub fn circuit_black_box<C: Circuit>(circuit: &C, name: &str) -> Result<Object, RHDLError> {
    let mut builder = Builder::new(name);
    let hdl = circuit.hdl(name)?;
    let input_len = C::I::BITS;
    let output_len = C::O::BITS;
    let arg0 = builder.add_input(input_len);
    let out = builder.allocate_outputs(output_len);
    builder.object.black_boxes.push(BlackBox {
        code: hdl,
        mode: BlackBoxMode::Asynchronous,
    });
    let arg0 = arg0.into_iter().map(Operand::Register).collect();
    let lhs = out.iter().copied().map(Operand::Register).collect();
    builder.object.ops.push(LocatedOpCode {
        op: OpCode::BlackBox(spec::BlackBox {
            lhs,
            arg: vec![arg0],
            code: BlackBoxId::new(0),
        }),
        loc: None,
    });
    builder.build(BuilderMode::Asynchronous)
}

pub fn synchronous_black_box<S: Synchronous>(circuit: &S, name: &str) -> Result<Object, RHDLError> {
    let mut builder = Builder::new(name);
    let hdl = circuit.hdl(name)?;
    let input_len = S::I::BITS;
    let output_len = S::O::BITS;
    // This is the Clock/Reset input
    let arg0 = builder.add_input(2);
    let arg1 = builder.add_input(input_len);
    let out = builder.allocate_outputs(output_len);
    builder.object.black_boxes.push(BlackBox {
        code: hdl,
        mode: BlackBoxMode::Synchronous,
    });
    let arg0 = arg0.into_iter().map(Operand::Register).collect();
    let arg1 = arg1.into_iter().map(Operand::Register).collect();
    let lhs = out.iter().copied().map(Operand::Register).collect();
    builder.object.ops.push(LocatedOpCode {
        op: OpCode::BlackBox(spec::BlackBox {
            lhs,
            arg: vec![arg0, arg1],
            code: BlackBoxId::new(0),
        }),
        loc: None,
    });
    builder.build(BuilderMode::Synchronous)
}

pub fn constant<T: Digital>(val: &T, name: &str) -> Result<Object, RHDLError> {
    let mut builder = Builder::new(name);
    let bits = val.bin();
    builder.object.outputs = bits.into_iter().map(Operand::from).collect();
    builder.build(BuilderMode::Synchronous)
}
