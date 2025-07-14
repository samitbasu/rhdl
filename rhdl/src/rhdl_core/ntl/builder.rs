use crate::core::types::digital::Digital;
use crate::prelude::{Circuit, ClockReset, Kind};
use crate::rhdl_core::common::symtab::RegisterId;
use crate::rhdl_core::ntl::object::WireDetails;
use crate::rhdl_core::ntl::spec::{self, Assign, BlackBoxId, WireKind};
use crate::{
    prelude::{RHDLError, Synchronous},
    rhdl_core::{
        ast::source::spanned_source_set::SpannedSourceSet,
        compiler::optimize_ntl,
        ntl::{
            object::{BlackBox, BlackBoxMode, LocatedOpCode, Object},
            spec::{OpCode, Wire},
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
    pub fn add_code(&mut self, code: &SpannedSourceSet) {
        self.object.code.extend(code.sources.clone());
    }
    pub fn add_input(&mut self, kind: Kind) -> Vec<RegisterId<WireKind>> {
        let ret = (0..kind.bits())
            .map(|ndx| {
                let wd = WireDetails {
                    source_details: None,
                    kind,
                    bit: ndx,
                };
                self.object.symtab.reg((), wd)
            })
            .flat_map(Wire::reg)
            .collect::<Vec<_>>();
        self.object.inputs.push(ret.clone());
        ret
    }
    pub fn allocate_outputs(&mut self, kind: Kind) -> Vec<RegisterId<WireKind>> {
        let ret = (0..kind.bits())
            .map(|ndx| {
                let wd = WireDetails {
                    source_details: None,
                    kind,
                    bit: ndx,
                };
                self.object.symtab.reg((), wd)
            })
            .collect::<Vec<_>>();
        self.object.outputs = ret.clone();
        ret.into_iter().flat_map(Wire::reg).collect()
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
    pub fn import(&mut self, other: &Object) -> impl Fn(Wire) -> Wire + use<> {
        self.add_code(&other.code);
        self.object.import(other)
    }
    pub fn copy_from_to<T: Into<Wire>, S: Into<Wire>>(&mut self, rhs: T, lhs: S) {
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
    let arg0 = builder.add_input(C::I::static_kind());
    let out = builder.allocate_outputs(C::O::static_kind());
    builder.object.black_boxes.push(BlackBox {
        code: hdl,
        mode: BlackBoxMode::Asynchronous,
    });
    let arg0 = arg0.into_iter().map(Wire::Register).collect();
    let lhs = out.iter().copied().map(Wire::Register).collect();
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
    // This is the Clock/Reset input
    let arg0 = builder.add_input(ClockReset::static_kind());
    let arg1 = builder.add_input(S::I::static_kind());
    let out = builder.allocate_outputs(S::O::static_kind());
    builder.object.black_boxes.push(BlackBox {
        code: hdl,
        mode: BlackBoxMode::Synchronous,
    });
    let arg0 = arg0.into_iter().map(Wire::Register).collect();
    let arg1 = arg1.into_iter().map(Wire::Register).collect();
    let lhs = out.iter().copied().map(Wire::Register).collect();
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
    let bits = val
        .bin()
        .into_iter()
        .enumerate()
        .map(|(ndx, val)| {
            let wire_details = WireDetails {
                source_details: None,
                kind: T::static_kind(),
                bit: ndx,
            };
            builder.object.symtab.lit(val, wire_details)
        })
        .collect();
    builder.object.outputs = bits;
    builder.build(BuilderMode::Synchronous)
}
