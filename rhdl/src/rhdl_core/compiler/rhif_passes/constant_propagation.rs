use crate::rhdl_core::{
    ast::source::source_location::SourceLocation,
    rhif::{
        object::LocatedOpCode,
        runtime_ops::{array, binary, tuple, unary},
        spec::{
            Array, Assign, Binary, Case, CaseArgument, Cast, Enum, Exec, Index, LiteralId, OpCode,
            Repeat, Select, Slot, Splice, Struct, Tuple, Unary, Wrap,
        },
        vm::execute,
        Object,
    },
    types::path::Path,
    RHDLError, TypedBits,
};

use super::pass::Pass;

pub struct ConstantPropagation {}

fn assign_literal(loc: SourceLocation, value: TypedBits, obj: &mut Object) -> Slot {
    let literal = LiteralId(obj.literal_max_index().0 + 1);
    obj.literals.insert(literal, value);
    obj.symbols.slot_map.insert(Slot::Literal(literal), loc);
    Slot::Literal(literal)
}

fn propogate_array(
    loc: SourceLocation,
    param: Array,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Array { lhs, elements } = param;
    if elements.iter().all(|x| x.is_literal()) {
        let elements = elements
            .iter()
            .map(|x| obj.literals[&x.as_literal().unwrap()].clone())
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

fn propogate_unary(
    loc: SourceLocation,
    params: Unary,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Unary { op, lhs, arg1 } = params;
    if let Slot::Literal(arg1_lit) = arg1 {
        let arg1_val = obj.literals[&arg1_lit].clone();
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

fn propogate_binary(
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
        let arg1_val = obj.literals[&arg1_lit].clone();
        let arg2_val = obj.literals[&arg2_lit].clone();
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

fn propogate_tuple(
    loc: SourceLocation,
    params: Tuple,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Tuple { lhs, fields } = &params;
    if fields.iter().all(|x| x.is_literal()) {
        let fields = fields
            .iter()
            .map(|x| obj.literals[&x.as_literal().unwrap()].clone())
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

fn propogate_wrap(
    loc: SourceLocation,
    wrap: Wrap,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Wrap { lhs, op, arg, kind } = &wrap;
    if let (Slot::Literal(arg_lit), Some(kind)) = (arg, kind) {
        let arg_val = obj.literals[arg_lit].clone();
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

fn propogate_as_bits(
    loc: SourceLocation,
    params: Cast,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Cast { lhs, arg, len } = params;
    if let (Slot::Literal(arg_lit), Some(len)) = (arg, len) {
        let arg_val = obj.literals[&arg_lit].clone();
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

fn propogate_resize(
    loc: SourceLocation,
    params: Cast,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Cast { lhs, arg, len } = params;
    if let (Slot::Literal(arg_lit), Some(len)) = (arg, len) {
        let arg_val = obj.literals[&arg_lit].clone();
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

fn propogate_as_signed(
    loc: SourceLocation,
    params: Cast,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Cast { lhs, arg, len } = params;
    if let (Slot::Literal(arg_lit), Some(len)) = (arg, len) {
        let arg_val = obj.literals[&arg_lit].clone();
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

fn propogate_repeat(
    loc: SourceLocation,
    params: Repeat,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Repeat { lhs, value, len } = params;
    if let Slot::Literal(arg_lit) = value {
        let arg_val = obj.literals[&arg_lit].clone();
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

fn propogate_index(
    loc: SourceLocation,
    params: Index,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Index { lhs, arg, path } = &params;
    if let (Slot::Literal(arg_lit), false) = (arg, path.any_dynamic()) {
        let arg_val = obj.literals[arg_lit].clone();
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

fn propogate_splice(
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
        let orig_val = obj.literals[orig_lit].clone();
        let subst_val = obj.literals[subst_lit].clone();
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

fn propogate_select(
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
        let cond_val = obj.literals[&cond_lit].clone();
        let true_val = obj.literals[&true_lit].clone();
        let false_val = obj.literals[&false_lit].clone();
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

fn propogate_struct(
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
    let rest_is_not_literal = rest.map(|x| !x.is_literal()).unwrap_or(false);
    if fields.iter().all(|x| x.value.is_literal()) && !rest_is_not_literal {
        let mut rhs = if let Some(rest) = rest {
            obj.literals[&rest.as_literal().unwrap()].clone()
        } else {
            template.clone()
        };
        for field in fields {
            let value = obj.literals[&field.value.as_literal().unwrap()].clone();
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

fn propogate_enum(
    loc: SourceLocation,
    params: Enum,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Enum {
        lhs,
        fields,
        template,
    } = &params;
    if fields.iter().all(|x| x.value.is_literal()) {
        let mut rhs = template.clone();
        let discriminant = rhs.discriminant()?.as_i64()?;
        for field in fields {
            let value = obj.literals[&field.value.as_literal().unwrap()].clone();
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
        CaseArgument::Slot(slot) => slot.is_literal(),
        CaseArgument::Wild => true,
    }
}

fn propogate_case(
    loc: SourceLocation,
    params: Case,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Case {
        lhs,
        discriminant,
        table,
    } = &params;
    if discriminant.is_literal()
        && table
            .iter()
            .all(|(arg, val)| case_argument_is_literal(arg) && val.is_literal())
    {
        let discriminant_val = obj.literals[&discriminant.as_literal().unwrap()].clone();
        let rhs = table
            .iter()
            .find(|(disc, _)| {
                if let CaseArgument::Slot(slot) = disc {
                    discriminant_val == obj.literals[&slot.as_literal().unwrap()]
                } else {
                    true
                }
            })
            .unwrap()
            .1;
        let rhs = obj.literals[&rhs.as_literal().unwrap()].clone();
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

fn propogate_exec(
    loc: SourceLocation,
    params: Exec,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Exec { lhs, id, args } = &params;
    if args.iter().all(|x| x.is_literal()) {
        let args = args
            .iter()
            .map(|x| obj.literals[&x.as_literal().unwrap()].clone())
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
    fn run(mut input: crate::rhdl_core::rhif::Object) -> Result<crate::rhdl_core::rhif::Object, crate::rhdl_core::RHDLError> {
        let ops = std::mem::take(&mut input.ops);
        input.ops = ops
            .into_iter()
            .map(|lop| match lop.op {
                OpCode::Binary(binary) => propogate_binary(lop.loc, binary, &mut input),
                OpCode::Unary(unary) => propogate_unary(lop.loc, unary, &mut input),
                OpCode::Array(array) => propogate_array(lop.loc, array, &mut input),
                OpCode::Tuple(tuple) => propogate_tuple(lop.loc, tuple, &mut input),
                OpCode::AsBits(cast) => propogate_as_bits(lop.loc, cast, &mut input),
                OpCode::AsSigned(cast) => propogate_as_signed(lop.loc, cast, &mut input),
                OpCode::Resize(cast) => propogate_resize(lop.loc, cast, &mut input),
                OpCode::Repeat(repeat) => propogate_repeat(lop.loc, repeat, &mut input),
                OpCode::Index(index) => propogate_index(lop.loc, index, &mut input),
                OpCode::Splice(splice) => propogate_splice(lop.loc, splice, &mut input),
                OpCode::Select(select) => propogate_select(lop.loc, select, &mut input),
                OpCode::Struct(strukt) => propogate_struct(lop.loc, strukt, &mut input),
                OpCode::Enum(enumerate) => propogate_enum(lop.loc, enumerate, &mut input),
                OpCode::Case(case) => propogate_case(lop.loc, case, &mut input),
                OpCode::Exec(exec) => propogate_exec(lop.loc, exec, &mut input),
                OpCode::Wrap(wrap) => propogate_wrap(lop.loc, wrap, &mut input),
                OpCode::Assign(_) | OpCode::Noop | OpCode::Comment(_) | OpCode::Retime(_) => {
                    Ok(lop)
                }
            })
            .collect::<Result<Vec<_>, RHDLError>>()?;
        Ok(input)
    }
}
