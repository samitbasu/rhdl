use std::fmt::Display;

use crate::rhif::{
    AluBinary, AluUnary, AssignOp, BinaryOp, BlockId, CopyOp, FieldRefOp, IfOp, IndexRefOp, Member,
    OpCode, RefOp, Slot, UnaryOp,
};

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Binary(op) => write!(f, "{}", op),
            OpCode::Unary(op) => write!(f, "{}", op),
            OpCode::Assign(op) => write!(f, "{}", op),
            OpCode::Ref(op) => write!(f, "{}", op),
            OpCode::FieldRef(op) => write!(f, "{}", op),
            OpCode::IndexRef(op) => write!(f, "{}", op),
            OpCode::If(op) => write!(f, "{}", op),
            OpCode::Call(block) => write!(f, "sub {}", block),
            OpCode::Copy(op) => write!(f, "{}", op),
            _ => todo!("opcode {:?}", self),
        }
    }
}

impl Display for CopyOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- {}", self.lhs, self.rhs)
    }
}

impl Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B{}", self.0)
    }
}

impl Display for IfOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " {} <- if {} then {} else {}",
            self.lhs, self.cond, self.then_branch, self.else_branch
        )
    }
}

impl Display for AssignOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*{} <- {}", self.lhs, self.rhs)
    }
}

impl Display for RefOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- &{}", self.lhs, self.arg)
    }
}

impl Display for FieldRefOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- &{}.{}", self.lhs, self.arg, self.member)
    }
}

impl Display for Member {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Member::Named(s) => write!(f, "{}", s),
            Member::Unnamed(i) => write!(f, "{}", i),
        }
    }
}

impl Display for IndexRefOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- &{}[{}]", self.lhs, self.arg, self.index)
    }
}

impl Display for AluBinary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AluBinary::Add => write!(f, "+"),
            AluBinary::Sub => write!(f, "-"),
            AluBinary::Mul => write!(f, "*"),
            AluBinary::BitAnd => write!(f, "&"),
            AluBinary::BitOr => write!(f, "|"),
            AluBinary::BitXor => write!(f, "^"),
            AluBinary::Shl => write!(f, "<<"),
            AluBinary::Shr => write!(f, ">>"),
            AluBinary::Eq => write!(f, "=="),
            AluBinary::Ne => write!(f, "!="),
            AluBinary::Lt => write!(f, "<"),
            AluBinary::Le => write!(f, "<="),
            AluBinary::Gt => write!(f, ">"),
            AluBinary::Ge => write!(f, ">="),
            AluBinary::And => write!(f, "&&"),
            AluBinary::Or => write!(f, "||"),
        }
    }
}

impl Display for AluUnary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AluUnary::Neg => write!(f, "-"),
            AluUnary::Not => write!(f, "!"),
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " {} <- {} {} {}",
            self.lhs, self.arg1, self.op, self.arg2
        )
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- {} {}", self.lhs, self.op, self.arg1)
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Slot::Literal(l) => write!(f, "{}", l),
            // Use 4 spaces for alignment
            Slot::Register(usize) => write!(f, "r{}", usize),
            Slot::Empty => write!(f, "{{}}"),
        }
    }
}
