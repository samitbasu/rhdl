use crate::{
    Kind,
    ntl::{
        object::Object,
        spec::{Assign, Binary, OpCode, Wire},
    },
    types::path::{Path, bit_range, leaf_paths},
};

fn vec_disp(f: &mut std::fmt::Formatter<'_>, data: &[Wire]) -> std::fmt::Result {
    write!(f, "{{")?;
    for (ndx, op) in data.iter().enumerate() {
        write!(f, "{op}")?;
        if ndx != (data.len() - 1) {
            write!(f, ",")?;
        }
    }
    write!(f, "}}")
}

pub fn summarize_path(ty: Kind, bit: usize) -> String {
    let paths = leaf_paths(&ty, Path::default());
    if let Some(path) = paths.iter().find(|p| {
        let Ok((bits1, _)) = bit_range(ty, p) else {
            return false;
        };
        bits1.contains(&bit)
    }) {
        if !path.is_empty() {
            format!("Bit {bit} of {{ {path:?} }}")
        } else {
            "All".to_string()
        }
    } else {
        "Unknown".to_string()
    }
}

impl std::fmt::Debug for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Noop => write!(f, "Noop"),
            OpCode::Assign(Assign { lhs, rhs }) => {
                write!(f, " {lhs} <- {rhs}")
            }
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => write!(f, " {lhs} <- {arg1} {op:?} {arg2}"),
            OpCode::Vector(vector) => {
                write!(f, " ")?;
                vec_disp(f, &vector.lhs)?;
                write!(f, " <- ")?;
                vec_disp(f, &vector.arg1)?;
                write!(f, " {:?} ", vector.op)?;
                vec_disp(f, &vector.arg2)
            }
            OpCode::Case(case) => {
                write!(f, " {} <- case ", case.lhs)?;
                vec_disp(f, &case.discriminant)?;
                writeln!(f, " {{")?;
                for (cond, val) in &case.entries {
                    writeln!(f, "          {cond:?} => {val}")?;
                }
                writeln!(f, " }}")
            }
            OpCode::Select(select) => {
                write!(
                    f,
                    " {} <- {} ? {} : {}",
                    select.lhs, select.selector, select.true_case, select.false_case
                )
            }
            OpCode::Not(not) => {
                write!(f, " {} <- !{}", not.lhs, not.arg)
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
            let arg_as_ops = arg.iter().copied().map(Wire::Register).collect::<Vec<_>>();
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
