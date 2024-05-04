use crate::path::PathElement;
use crate::rhif::spec::AluUnary;
use crate::rhif::spec::OpCode;
use crate::Kind;

use super::mir::Mir;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Default)]
struct MirTypeDb {
    strukts: HashMap<u64, String>,
}

impl MirTypeDb {
    fn translate(&mut self, kind: &Kind) -> String {
        match kind {
            Kind::Empty => "()".to_string(),
            Kind::Array(array) => {
                format!("[{}; {}]", self.translate(&array.base), array.size)
            }
            Kind::Tuple(tuple) => {
                let mut result = "(".to_string();
                for kind in &tuple.elements {
                    write!(result, "{}, ", self.translate(kind)).unwrap();
                }
                write!(result, ")").unwrap();
                result
            }
            Kind::Bits(len) => format!("Bits<{}>", len),
            Kind::Signed(len) => format!("Signed<{}>", len),
            _ => todo!(),
        }
    }
}

fn write_rust_stub(mir: &Mir) -> Result<String> {
    // Create a rust program that will be used to typecheck the
    // MIR code.
    let mut rp = String::new();
    write!(rp, "fn main(")?;
    /*    for slot in &mir.arguments {
        write!(
            rp,
            "{}: {},",
            slot,
            mir.ty
                .get(slot)
                .ok_or(anyhow!("argument type {slot} not known"))?
        )?;
    }*/
    writeln!(
        rp,
        ") -> {} {{",
        mir.ty
            .get(&mir.return_slot)
            .ok_or(anyhow!("Return slot not found"))?
    )?;
    for literal in &mir.literals {
        writeln!(rp, "  let {} = {};", literal.0, literal.1)?;
    }
    for op in &mir.ops {
        let op = &op.op;
        match op {
            OpCode::Array(array) => {
                write!(rp, "  let {} = [", array.lhs)?;
                for element in array.elements.iter() {
                    write!(rp, "{},", element)?;
                }
                writeln!(rp, "];")?;
            }
            OpCode::AsBits(as_bits) => {
                writeln!(
                    rp,
                    "  let {} : Bits<{}> = {}.as_bits::<{}>();",
                    as_bits.lhs, as_bits.len, as_bits.arg, as_bits.len
                )?;
            }
            OpCode::AsKind(as_kind) => {
                writeln!(
                    rp,
                    "  let {} = {}.as_kind({:?});",
                    as_kind.lhs, as_kind.arg, as_kind.kind
                )?;
            }
            OpCode::AsSigned(as_signed) => {
                writeln!(
                    rp,
                    "  let {} = {}.as_signed({});",
                    as_signed.lhs, as_signed.arg, as_signed.len
                )?;
            }
            OpCode::Assign(assign) => {
                writeln!(rp, "  let {} = {};", assign.lhs, assign.rhs)?;
            }
            OpCode::Binary(binary) => {
                writeln!(
                    rp,
                    "  let {} = {} {} {};",
                    binary.lhs, binary.arg1, binary.op, binary.arg2
                )?;
            }
            OpCode::Select(select) => {
                writeln!(
                    rp,
                    "  let {} = if {} {{ {} }} else {{ {} }};",
                    select.lhs, select.cond, select.true_value, select.false_value
                )?;
            }
            OpCode::Tuple(tuple) => {
                write!(rp, "  let {} = (", tuple.lhs)?;
                for element in tuple.fields.iter() {
                    write!(rp, "{},", element)?;
                }
                writeln!(rp, "  );")?;
            }
            OpCode::Unary(unary) => match &unary.op {
                AluUnary::Any => {
                    writeln!(rp, "  let {} = {}.any();", unary.lhs, unary.arg1)?;
                }
                AluUnary::All => {
                    writeln!(rp, "  let {} = {}.all();", unary.lhs, unary.arg1)?;
                }
                AluUnary::Xor => {
                    writeln!(rp, "  let {} = {}.xor();", unary.lhs, unary.arg1)?;
                }
                AluUnary::Unsigned => {
                    writeln!(rp, "  let {} = {}.as_unsigned();", unary.lhs, unary.arg1)?;
                }
                AluUnary::Signed => {
                    writeln!(rp, "  let {} = {}.as_signed();", unary.lhs, unary.arg1)?;
                }
                AluUnary::Neg => {
                    writeln!(rp, "  let {} = -{};", unary.lhs, unary.arg1)?;
                }
                AluUnary::Not => {
                    writeln!(rp, "  let {} = !{};", unary.lhs, unary.arg1)?;
                }
            },
            OpCode::Noop => {}
            OpCode::Index(index) => {
                write!(rp, "  let {} = {}", index.lhs, index.arg)?;
                for segment in index.path.iter() {
                    match segment {
                        PathElement::DynamicIndex(index) => {
                            write!(rp, "[{}]", index)?;
                        }
                        PathElement::Index(index) => {
                            write!(rp, "[{}]", index)?;
                        }
                        PathElement::Field(field) => {
                            write!(rp, ".{}", field)?;
                        }
                        PathElement::EnumDiscriminant => {
                            write!(rp, ".discriminant()")?;
                        }
                        PathElement::SignalValue => {
                            write!(rp, ".val()")?;
                        }
                        PathElement::EnumPayload(payload) => {
                            write!(rp, ".payload({})", payload)?;
                        }
                        PathElement::EnumPayloadByValue(payload) => {
                            write!(rp, ".payload_by_value({})", payload)?;
                        }
                    };
                }
                writeln!(rp, ";")?;
            }
            OpCode::Splice(splice) => {
                writeln!(rp, "  let mut {} = {};", splice.lhs, splice.orig)?;
                write!(rp, "  {}", splice.lhs)?;
                for segment in splice.path.iter() {
                    match segment {
                        PathElement::DynamicIndex(index) => {
                            write!(rp, "[{}]", index)?;
                        }
                        PathElement::Index(index) => {
                            write!(rp, "[{}]", index)?;
                        }
                        PathElement::Field(field) => {
                            write!(rp, ".{}", field)?;
                        }
                        PathElement::EnumDiscriminant => {
                            write!(rp, ".discriminant()")?;
                        }
                        PathElement::SignalValue => {
                            write!(rp, ".val()")?;
                        }
                        PathElement::EnumPayload(payload) => {
                            write!(rp, ".payload({})", payload)?;
                        }
                        PathElement::EnumPayloadByValue(payload) => {
                            write!(rp, ".payload_by_value({})", payload)?;
                        }
                    };
                }
                writeln!(rp, " = {};", splice.subst)?;
            }
            OpCode::Repeat(repeat) => {
                write!(rp, "  let {} = [", repeat.lhs)?;
                write!(rp, "{}; {}", repeat.value, repeat.len)?;
                writeln!(rp, "];")?;
            }
            OpCode::Struct(strukt) => {
                write!(
                    rp,
                    "  let {} = {} {{",
                    strukt.lhs,
                    strukt.template.kind.get_name()
                )?;
                for field in strukt.fields.iter() {
                    write!(rp, "    {}: {},", field.member, field.value)?;
                }
                if let Some(rest) = &strukt.rest {
                    write!(rp, "    ..{}", rest)?;
                }
                writeln!(rp, "  }};")?;
            }
            OpCode::Case(kase) => {
                writeln!(rp, "  let {} = match {} {{", kase.lhs, kase.discriminant)?;
                for arm in kase.table.iter() {
                    writeln!(rp, "    {} => {},", arm.0, arm.1)?;
                }
                writeln!(rp, "  }};")?;
            }
            OpCode::Exec(exec) => {
                write!(rp, "  let {} = {}(", exec.lhs, exec.id)?;
                for (ndx, arg) in exec.args.iter().enumerate() {
                    if ndx > 0 {
                        write!(rp, ", ")?;
                    }
                    write!(rp, "{}", arg)?;
                }
                writeln!(rp, ");")?;
            }
            OpCode::Enum(enumerate) => {
                write!(
                    rp,
                    "  let {} = {} {{",
                    enumerate.lhs,
                    enumerate.template.kind.get_name(),
                )?;
                for field in enumerate.fields.iter() {
                    write!(rp, "    {}: {},", field.member, field.value)?;
                }
                writeln!(rp, "  }};")?;
            }
            OpCode::Comment(_) => {}
        }
    }
    writeln!(rp, "  {}", mir.return_slot)?;
    writeln!(rp, "}}")?;
    Ok(rp)
}

pub fn infer(mir: &Mir) -> Result<()> {
    eprintln!("{}", write_rust_stub(mir)?);
    Ok(())
}
