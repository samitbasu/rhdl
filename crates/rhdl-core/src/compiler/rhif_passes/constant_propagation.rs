use crate::{
    RHDLError, TypedBits,
    ast::SourceLocation,
    rhif::{
        Object,
        object::{LocatedOpCode, SourceDetails},
        runtime_ops::{array, binary, tuple, unary},
        spec::{
            Array, Assign, Binary, Case, CaseArgument, Cast, Enum, Exec, Index, OpCode, Repeat,
            Select, Slot, Splice, Struct, Tuple, Unary, Wrap,
        },
        vm::execute,
    },
    types::path::Path,
};

use super::pass::Pass;

pub struct ConstantPropagation {}

fn assign_literal(loc: SourceLocation, value: TypedBits, obj: &mut Object) -> Slot {
    obj.symtab.lit(
        value,
        SourceDetails {
            location: loc,
            name: None,
        },
    )
}

fn propagate_array(
    loc: SourceLocation,
    param: Array,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Array { lhs, elements } = param;
    if elements.iter().all(|x| x.is_lit()) {
        let elements = elements
            .iter()
            .map(|x| obj.symtab[&x.lit().unwrap()].clone())
            .collect::<Vec<_>>();
        let rhs = array(&elements);
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Array(Array { lhs, elements }),
            loc,
        })
    }
}

fn propagate_unary(
    loc: SourceLocation,
    params: Unary,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Unary { op, lhs, arg1 } = params;
    if let Slot::Literal(arg1_lit) = arg1 {
        let arg1_val = obj.symtab[&arg1_lit].clone();
        let rhs = unary(op, arg1_val)?;
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Unary(params),
            loc,
        })
    }
}

fn propagate_binary(
    loc: SourceLocation,
    params: Binary,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Binary {
        op,
        lhs,
        arg1,
        arg2,
    } = params;
    if let (Slot::Literal(arg1_lit), Slot::Literal(arg2_lit)) = (arg1, arg2) {
        let arg1_val = obj.symtab[&arg1_lit].clone();
        let arg2_val = obj.symtab[&arg2_lit].clone();
        let rhs = binary(op, arg1_val, arg2_val)?;
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Binary(params),
            loc,
        })
    }
}

fn propagate_tuple(
    loc: SourceLocation,
    params: Tuple,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Tuple { lhs, fields } = &params;
    if fields.iter().all(|x| x.is_lit()) {
        let fields = fields
            .iter()
            .map(|x| obj.symtab[&x.lit().unwrap()].clone())
            .collect::<Vec<_>>();
        let rhs = tuple(&fields);
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Tuple(params),
            loc,
        })
    }
}

fn propagate_wrap(
    loc: SourceLocation,
    wrap: Wrap,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Wrap { lhs, op, arg, kind } = &wrap;
    if let (Slot::Literal(arg_lit), Some(kind)) = (arg, kind) {
        let arg_val = obj.symtab[arg_lit].clone();
        let rhs = arg_val.wrap(*op, kind)?;
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Wrap(wrap),
            loc,
        })
    }
}

fn propagate_as_bits(
    loc: SourceLocation,
    params: Cast,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Cast { lhs, arg, len } = params;
    if let (Slot::Literal(arg_lit), Some(len)) = (arg, len) {
        let arg_val = obj.symtab[&arg_lit].clone();
        let rhs = arg_val.unsigned_cast(len)?;
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::AsBits(params),
            loc,
        })
    }
}

fn propagate_resize(
    loc: SourceLocation,
    params: Cast,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Cast { lhs, arg, len } = params;
    if let (Slot::Literal(arg_lit), Some(len)) = (arg, len) {
        let arg_val = obj.symtab[&arg_lit].clone();
        let rhs = arg_val.resize(len)?;
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Resize(params),
            loc,
        })
    }
}

fn propagate_as_signed(
    loc: SourceLocation,
    params: Cast,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Cast { lhs, arg, len } = params;
    if let (Slot::Literal(arg_lit), Some(len)) = (arg, len) {
        let arg_val = obj.symtab[&arg_lit].clone();
        let rhs = arg_val.signed_cast(len)?;
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::AsSigned(params),
            loc,
        })
    }
}

fn propagate_repeat(
    loc: SourceLocation,
    params: Repeat,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Repeat { lhs, value, len } = params;
    if let Slot::Literal(arg_lit) = value {
        let arg_val = obj.symtab[&arg_lit].clone();
        let rhs = arg_val.repeat(len as usize);
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Repeat(params),
            loc,
        })
    }
}

fn propagate_index(
    loc: SourceLocation,
    params: Index,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Index { lhs, arg, path } = &params;
    if let (Slot::Literal(arg_lit), false) = (arg, path.any_dynamic()) {
        let arg_val = obj.symtab[arg_lit].clone();
        let rhs = arg_val.path(path)?;
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Index(params),
            loc,
        })
    }
}

fn propagate_splice(
    loc: SourceLocation,
    params: Splice,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Splice {
        lhs,
        orig,
        path,
        subst,
    } = &params;
    if let (Slot::Literal(orig_lit), Slot::Literal(subst_lit), false) =
        (orig, subst, path.any_dynamic())
    {
        let orig_val = obj.symtab[orig_lit].clone();
        let subst_val = obj.symtab[subst_lit].clone();
        let rhs = orig_val.splice(path, subst_val)?;
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Splice(params),
            loc,
        })
    }
}

fn propagate_select(
    loc: SourceLocation,
    params: Select,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Select {
        lhs,
        cond,
        true_value,
        false_value,
    } = params;
    if let (Slot::Literal(cond_lit), Slot::Literal(true_lit), Slot::Literal(false_lit)) =
        (cond, true_value, false_value)
    {
        let cond_val = obj.symtab[&cond_lit].clone();
        let true_val = obj.symtab[&true_lit].clone();
        let false_val = obj.symtab[&false_lit].clone();
        let rhs = if cond_val.any().as_bool()? {
            true_val
        } else {
            false_val
        };
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Select(params),
            loc,
        })
    }
}

fn propagate_struct(
    loc: SourceLocation,
    params: Struct,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Struct {
        lhs,
        fields,
        rest,
        template,
    } = &params;
    let rest_is_not_literal = rest.map(|x| !x.is_lit()).unwrap_or(false);
    if fields.iter().all(|x| x.value.is_lit()) && !rest_is_not_literal {
        let mut rhs = if let Some(rest) = rest {
            obj.symtab[&rest.lit().unwrap()].clone()
        } else {
            template.clone()
        };
        for field in fields {
            let value = obj.symtab[&field.value.lit().unwrap()].clone();
            let path = Path::default().member(&field.member);
            rhs = rhs.splice(&path, value)?;
        }
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Struct(params),
            loc,
        })
    }
}

fn propagate_enum(
    loc: SourceLocation,
    params: Enum,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Enum {
        lhs,
        fields,
        template,
    } = &params;
    if fields.iter().all(|x| x.value.is_lit()) {
        let mut rhs = template.clone();
        let discriminant = rhs.discriminant()?.as_i64()?;
        for field in fields {
            let value = obj.symtab[&field.value.lit().unwrap()].clone();
            let path = Path::default()
                .payload_by_value(discriminant)
                .member(&field.member);
            rhs = rhs.splice(&path, value)?;
        }
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Enum(params),
            loc,
        })
    }
}

fn case_argument_is_literal(x: &CaseArgument) -> bool {
    match x {
        CaseArgument::Slot(slot) => slot.is_lit(),
        CaseArgument::Wild => true,
    }
}

fn propagate_case(
    loc: SourceLocation,
    params: Case,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Case {
        lhs,
        discriminant,
        table,
    } = &params;
    if discriminant.is_lit()
        && table
            .iter()
            .all(|(arg, val)| case_argument_is_literal(arg) && val.is_lit())
    {
        let discriminant_val = obj.symtab[&discriminant.lit().unwrap()].clone();
        let rhs = table
            .iter()
            .find(|(disc, _)| {
                if let CaseArgument::Slot(slot) = disc {
                    discriminant_val == obj.symtab[&slot.lit().unwrap()]
                } else {
                    true
                }
            })
            .unwrap()
            .1;
        let rhs = obj.symtab[&rhs.lit().unwrap()].clone();
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Case(params),
            loc,
        })
    }
}

fn propagate_exec(
    loc: SourceLocation,
    params: Exec,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Exec { lhs, id, args } = &params;
    if args.iter().all(|x| x.is_lit()) {
        let args = args
            .iter()
            .map(|x| obj.symtab[&x.lit().unwrap()].clone())
            .collect::<Vec<_>>();
        let rhs = execute(&obj.externals[id], args)?;
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, rhs, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Exec(params),
            loc,
        })
    }
}

impl Pass for ConstantPropagation {
    fn description() -> &'static str {
        "RHIF constant propogation"
    }
    fn run(mut input: crate::rhif::Object) -> Result<crate::rhif::Object, crate::RHDLError> {
        let ops = std::mem::take(&mut input.ops);
        input.ops = ops
            .into_iter()
            .map(|lop| match lop.op {
                OpCode::Binary(binary) => propagate_binary(lop.loc, binary, &mut input),
                OpCode::Unary(unary) => propagate_unary(lop.loc, unary, &mut input),
                OpCode::Array(array) => propagate_array(lop.loc, array, &mut input),
                OpCode::Tuple(tuple) => propagate_tuple(lop.loc, tuple, &mut input),
                OpCode::AsBits(cast) => propagate_as_bits(lop.loc, cast, &mut input),
                OpCode::AsSigned(cast) => propagate_as_signed(lop.loc, cast, &mut input),
                OpCode::Resize(cast) => propagate_resize(lop.loc, cast, &mut input),
                OpCode::Repeat(repeat) => propagate_repeat(lop.loc, repeat, &mut input),
                OpCode::Index(index) => propagate_index(lop.loc, index, &mut input),
                OpCode::Splice(splice) => propagate_splice(lop.loc, splice, &mut input),
                OpCode::Select(select) => propagate_select(lop.loc, select, &mut input),
                OpCode::Struct(strukt) => propagate_struct(lop.loc, strukt, &mut input),
                OpCode::Enum(enumerate) => propagate_enum(lop.loc, enumerate, &mut input),
                OpCode::Case(case) => propagate_case(lop.loc, case, &mut input),
                OpCode::Exec(exec) => propagate_exec(lop.loc, exec, &mut input),
                OpCode::Wrap(wrap) => propagate_wrap(lop.loc, wrap, &mut input),
                OpCode::Assign(_) | OpCode::Noop | OpCode::Comment(_) | OpCode::Retime(_) => {
                    Ok(lop)
                }
            })
            .collect::<Result<Vec<_>, RHDLError>>()?;
        Ok(input)
    }
}
