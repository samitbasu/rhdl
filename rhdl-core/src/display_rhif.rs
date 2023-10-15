use std::fmt::Display;

use crate::rhif::{
    AluBinary, AluUnary, ArrayOp, AssignOp, BinaryOp, BlockId, CaseArgument, CaseOp, CopyOp,
    ExecOp, FieldOp, FieldRefOp, FieldValue, IfOp, IndexOp, IndexRefOp, Member, OpCode, RefOp,
    RepeatOp, Slot, StructOp, TupleOp, UnaryOp,
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
            OpCode::Block(block) => write!(f, " blk {}", block),
            OpCode::Copy(op) => write!(f, "{}", op),
            OpCode::Tuple(op) => write!(f, "{}", op),
            OpCode::Field(op) => write!(f, "{}", op),
            OpCode::Case(op) => write!(f, "{}", op),
            OpCode::Exec(op) => write!(f, "{}", op),
            OpCode::Struct(op) => write!(f, "{}", op),
            OpCode::Index(op) => write!(f, "{}", op),
            OpCode::Array(op) => write!(f, "{}", op),
            OpCode::Repeat(op) => write!(f, "{}", op),
            _ => todo!("OpCode {:?} not covered", self),
        }
    }
}

impl Display for RepeatOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- [{}; {}]", self.lhs, self.value, self.len)
    }
}

impl Display for ArrayOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- [", self.lhs)?;
        for (i, arg) in self.elements.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, "]")
    }
}

impl Display for IndexOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- {}[{}]", self.lhs, self.arg, self.index)
    }
}

impl Display for StructOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- {} {{", self.lhs, self.path.join("::"))?;
        for (i, field) in self.fields.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", field)?;
        }
        write!(f, " }}")
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.member, self.value)
    }
}

impl Display for ExecOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- {}(", self.lhs, self.path.join("::"))?;
        for (i, arg) in self.args.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, ")")
    }
}

impl Display for CaseOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " {} <- case {}", self.lhs, self.expr)?;
        for (cond, val) in self.table.iter() {
            writeln!(f, "         {} => {}", cond, val)?;
        }
        Ok(())
    }
}

impl Display for CaseArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaseArgument::Literal(l) => write!(f, "{}", l),
            CaseArgument::Wild => write!(f, "_"),
            CaseArgument::Path(p) => write!(f, "{}", p.join("::")),
        }
    }
}

impl Display for FieldOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- {}.{}", self.lhs, self.arg, self.member)
    }
}

impl Display for TupleOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} <- (", self.lhs)?;
        for (i, arg) in self.fields.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, ")")
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
            Slot::Literal(l) => write!(f, "l{}", l),
            // Use 4 spaces for alignment
            Slot::Register(usize) => write!(f, "r{}", usize),
            Slot::Empty => write!(f, "{{}}"),
        }
    }
}
