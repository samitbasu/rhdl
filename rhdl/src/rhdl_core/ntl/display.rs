use crate::rhdl_core::ntl::{
    object::Object,
    spec::{Assign, Binary, OpCode, Operand},
};

fn vec_disp(f: &mut std::fmt::Formatter<'_>, data: &[Operand]) -> std::fmt::Result {
    write!(f, "{{")?;
    for (ndx, op) in data.iter().enumerate() {
        write!(f, "{:?}", op)?;
        if ndx != (data.len() - 1) {
            write!(f, ",")?;
        }
    }
    write!(f, "}}")
}

impl std::fmt::Debug for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Noop => write!(f, "Noop"),
            OpCode::Assign(Assign { lhs, rhs }) => write!(f, " {:?} <- {:?}", lhs, rhs),
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => write!(f, " {:?} <- {:?} {:?} {:?}", lhs, arg1, op, arg2),
            OpCode::Vector(vector) => {
                write!(f, " ")?;
                vec_disp(f, &vector.lhs)?;
                write!(f, " <- ")?;
                vec_disp(f, &vector.arg1)?;
                write!(f, " {:?} ", vector.op)?;
                vec_disp(f, &vector.arg2)
            }
            OpCode::Case(case) => {
                write!(f, " {:?} <- case ", case.lhs)?;
                vec_disp(f, &case.discriminant)?;
                writeln!(f, " {{")?;
                for (cond, val) in &case.entries {
                    writeln!(f, "          {:?} => {:?}", cond, val)?;
                }
                writeln!(f, " }}")
            }
            OpCode::Comment(comment) => {
                write!(f, "// {}", comment)
            }
            OpCode::DynamicIndex(dynamic_index) => {
                write!(f, " ")?;
                vec_disp(f, &dynamic_index.lhs)?;
                write!(f, " <- ")?;
                vec_disp(f, &dynamic_index.arg)?;
                write!(f, "[")?;
                vec_disp(f, &dynamic_index.offset)?;
                write!(f, " +: {}]", dynamic_index.lhs.len())
            }
            OpCode::DynamicSplice(dynamic_splice) => {
                write!(f, " ")?;
                vec_disp(f, &dynamic_splice.lhs)?;
                write!(f, " <- ")?;
                vec_disp(f, &dynamic_splice.arg)?;
                write!(f, "; ")?;
                vec_disp(f, &dynamic_splice.lhs)?;
                write!(f, "[")?;
                vec_disp(f, &dynamic_splice.offset)?;
                write!(f, " +: {}] <- ", dynamic_splice.lhs.len())?;
                vec_disp(f, &dynamic_splice.value)
            }
            OpCode::Select(select) => {
                write!(
                    f,
                    " {:?} <- {:?} ? {:?} : {:?}",
                    select.lhs, select.selector, select.true_case, select.false_case
                )
            }
            OpCode::Not(not) => {
                write!(f, " {:?} <- !{:?}", not.lhs, not.arg)
            }
            OpCode::BlackBox(black_box) => {
                write!(f, " ")?;
                vec_disp(f, &black_box.lhs)?;
                write!(f, " <- black_box_{:?}(", black_box.code)?;
                for a in &black_box.arg {
                    vec_disp(f, a)?;
                    write!(f, ", ")?;
                }
                write!(f, ")")
            }
            OpCode::Unary(unary) => {
                write!(f, " ")?;
                vec_disp(f, &unary.lhs)?;
                write!(f, " <- {:?} ", unary.op)?;
                vec_disp(f, &unary.arg)
            }
        }
    }
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BTL {}", self.name)?;
        write!(f, "   arguments [")?;
        for (ndx, arg) in self.inputs.iter().enumerate() {
            let arg_as_ops = arg
                .iter()
                .copied()
                .map(Operand::Register)
                .collect::<Vec<_>>();
            vec_disp(f, &arg_as_ops)?;
            if ndx != self.inputs.len() - 1 {
                write!(f, ",")?;
            }
        }
        writeln!(f, "]")?;
        write!(f, "   return ")?;
        vec_disp(f, &self.outputs)?;
        writeln!(f)?;
        for lop in &self.ops {
            writeln!(f, "{:?}", lop.op)?;
        }
        Ok(())
    }
}
