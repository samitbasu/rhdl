// RHDL Intermediate Form (RHIF).

use std::collections::HashMap;

use anyhow::Result;

use crate::{digital_fn::DigitalSignature, ty::Ty, KernelFnKind, TypedBits};

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // lhs <- arg1 op arg2
    Binary {
        op: AluBinary,
        lhs: Slot,
        arg1: Slot,
        arg2: Slot,
    },
    // lhs <- op arg1
    Unary {
        op: AluUnary,
        lhs: Slot,
        arg1: Slot,
    },
    // return a
    Return {
        result: Option<Slot>,
    },
    // lhs <- if cond { then_branch } else { else_branch }
    If {
        lhs: Slot,
        cond: Slot,
        then_branch: BlockId,
        else_branch: BlockId,
    },
    // lhs <- arg[index]
    Index {
        lhs: Slot,
        arg: Slot,
        index: Slot,
    },
    // lhs <- rhs
    Copy {
        lhs: Slot,
        rhs: Slot,
    },
    // *lhs <- rhs
    Assign {
        lhs: Slot,
        rhs: Slot,
    },
    // lhs <- arg.member
    Field {
        lhs: Slot,
        arg: Slot,
        member: Member,
    },
    // lhs <- [value; len]
    Repeat {
        lhs: Slot,
        value: Slot,
        len: Slot,
    },
    // lhs <- Struct@path { fields (..rest) }
    Struct {
        lhs: Slot,
        path: String,
        fields: Vec<FieldValue>,
        rest: Option<Slot>,
    },
    // lhs <- Tuple(fields)
    Tuple {
        lhs: Slot,
        fields: Vec<Slot>,
    },
    // lhs = &arg
    Ref {
        lhs: Slot,
        arg: Slot,
    },
    // lhs = &arg.member
    FieldRef {
        lhs: Slot,
        arg: Slot,
        member: Member,
    },
    // lhs = &arg[index]
    IndexRef {
        lhs: Slot,
        arg: Slot,
        index: Slot,
    },
    // Jump to block
    Block(BlockId),
    // ROM table
    Case {
        lhs: Slot,
        expr: Slot,
        table: Vec<(CaseArgument, BlockId)>,
    },
    // lhs = @path(args)
    Exec {
        lhs: Slot,
        path: String,
        args: Vec<Slot>,
    },
    // x <- [a, b, c, d]
    Array {
        lhs: Slot,
        elements: Vec<Slot>,
    },
    // x <- a#b where a is an enum, and b is the discriminant of the
    // variant.
    Payload {
        lhs: Slot,
        arg: Slot,
        discriminant: Slot,
    },
    // x <- tag where tag is the discriminant of the enum.
    Discriminant {
        lhs: Slot,
        arg: Slot,
    },
    // x <- enum(discriminant, fields)
    Enum {
        lhs: Slot,
        path: String,
        discriminant: Slot,
        fields: Vec<FieldValue>,
    },
    Comment(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaseArgument {
    Literal(Slot),
    Wild,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldValue {
    pub member: Member,
    pub value: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AluBinary {
    Add,
    Sub,
    Mul,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AluUnary {
    Neg,
    Not,
    All,
    Any,
    Xor,
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Slot {
    Literal(usize),
    Register(usize),
    Empty,
}
impl Slot {
    pub fn reg(&self) -> Result<usize> {
        match self {
            Slot::Register(r) => Ok(*r),
            _ => Err(anyhow::anyhow!("Not a register")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub ops: Vec<OpCode>,
}

#[derive(Debug, Clone)]
pub struct ExternalFunction {
    pub path: String,
    pub code: Option<KernelFnKind>,
    pub signature: DigitalSignature,
}

#[derive(Debug, Clone)]
pub struct Object {
    pub literals: Vec<TypedBits>,
    pub ty: HashMap<Slot, Ty>,
    pub blocks: Vec<Block>,
    pub return_slot: Slot,
    pub externals: HashMap<String, ExternalFunction>,
}

impl Object {
    pub fn literal(&self, slot: Slot) -> Result<&TypedBits> {
        match slot {
            Slot::Literal(l) => Ok(&self.literals[l]),
            _ => Err(anyhow::anyhow!("Not a literal")),
        }
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for regs in self.ty.keys() {
            if let Slot::Register(ndx) = regs {
                writeln!(f, "Reg r{} : {}", ndx, self.ty[regs])?;
            }
        }
        for (ndx, literal) in self.literals.iter().enumerate() {
            writeln!(f, "Literal l{} : {}", ndx, literal)?;
        }
        for (name, func) in &self.externals {
            writeln!(
                f,
                "Function name: {} code: {} signature: {}",
                name,
                func.code
                    .as_ref()
                    .map(|x| format!("{}", x))
                    .unwrap_or("none".into()),
                func.signature
            )?;
        }
        for block in &self.blocks {
            writeln!(f, "Block {}", block.id.0)?;
            for op in &block.ops {
                writeln!(f, "  {}", op)?;
            }
        }
        Ok(())
    }
}
