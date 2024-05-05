use crate::ast::ast_impl::ExprLit;
use crate::path::PathElement;
use crate::rhif::spec::AluUnary;
use crate::rhif::spec::CaseArgument;
use crate::rhif::spec::OpCode;
use crate::rhif::spec::Slot;
use crate::types::kind::Enum;
use crate::types::kind::Struct;
use crate::DiscriminantType;
use crate::Kind;

use super::mir::Mir;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Write;

struct MirTypeDb<'a> {
    strukts: HashMap<u64, String>,
    ty: &'a BTreeMap<Slot, Kind>,
}

impl<'a> MirTypeDb<'a> {
    fn new(ty: &'a BTreeMap<Slot, Kind>) -> Self {
        MirTypeDb {
            ty,
            strukts: HashMap::new(),
        }
    }
    fn enum_(&mut self, enum_: &Enum) -> Result<()> {
        if self.strukts.contains_key(&enum_.id) {
            return Ok(());
        }
        let mut result = String::new();
        writeln!(result, "#[derive(Default, Copy, Clone)]")?;
        writeln!(result, "enum E{} {{", enum_.id)?;
        writeln!(result, "  #[default]")?;
        writeln!(result, "  __default,")?;
        for variant in &enum_.variants {
            write!(result, "  {}", variant.name)?;
            match &variant.kind {
                Kind::Empty => {}
                Kind::Tuple(tuple) => {
                    write!(result, "  (")?;
                    for element in tuple.elements.iter() {
                        write!(result, "{},", self.translate(element)?)?;
                    }
                    write!(result, ")")?;
                }
                Kind::Struct(strukt) => {
                    write!(result, "  {{")?;
                    for field in &strukt.fields {
                        write!(
                            result,
                            "    {}: {},",
                            field.name,
                            self.translate(&field.kind)?
                        )?;
                    }
                    write!(result, "  }}")?;
                }
                _ => bail!("Unsupported enum variant kind {:?}", variant.kind),
            }
            writeln!(result, ",")?;
        }
        writeln!(result, "}}")?;
        writeln!(result, "impl E{} {{", enum_.id)?;
        let ret_type = match enum_.discriminant_layout.ty {
            DiscriminantType::Signed => {
                format!("SignedBits::<{}>", enum_.discriminant_layout.width)
            }
            DiscriminantType::Unsigned => {
                format!("Bits::<{}>", enum_.discriminant_layout.width)
            }
        };
        writeln!(
            result,
            "  fn discriminant(&self) -> {} {{ Default::default() }}",
            ret_type
        )?;
        for variant in &enum_.variants {
            writeln!(
                result,
                "  fn payload_{}(&self) -> {} {{ Default::default() }}",
                variant.name,
                self.translate(&variant.kind)?
            )?;
        }
        writeln!(result, "}}")?;
        self.strukts.insert(enum_.id, result);
        Ok(())
    }
    fn strukt(&mut self, strukt: &Struct) -> Result<()> {
        if self.strukts.contains_key(&strukt.id) {
            return Ok(());
        }
        let mut result = String::new();
        writeln!(result, "#[derive(Default, Copy, Clone)]")?;
        if strukt.is_tuple_struct() {
            write!(result, "struct S{}(", strukt.id)?;
            for field in &strukt.fields {
                write!(result, "{},", self.translate(&field.kind)?)?;
            }
            writeln!(result, ");")?;
            self.strukts.insert(strukt.id, result);
            return Ok(());
        }
        writeln!(result, "struct S{} {{", strukt.id)?;
        for field in &strukt.fields {
            writeln!(
                result,
                "  {}: {},",
                field.name,
                self.translate(&field.kind)?
            )?;
        }
        writeln!(result, "}}")?;
        self.strukts.insert(strukt.id, result);
        Ok(())
    }
    fn translate(&mut self, kind: &Kind) -> Result<String> {
        Ok(match kind {
            Kind::Empty => "()".to_string(),
            Kind::Array(array) => {
                format!("[{}; {}]", self.translate(&array.base)?, array.size)
            }
            Kind::Tuple(tuple) => {
                let mut result = "(".to_string();
                for kind in &tuple.elements {
                    write!(result, "{}, ", self.translate(kind)?)?;
                }
                write!(result, ")")?;
                result
            }
            Kind::Bits(len) => format!("Bits::<{}>", len),
            Kind::Signed(len) => format!("SignedBits::<{}>", len),
            Kind::Struct(strukt) => {
                self.strukt(strukt)?;
                format!("S{}", strukt.id)
            }
            Kind::Enum(enum_) => {
                self.enum_(enum_)?;
                format!("E{}", enum_.id)
            }
            Kind::Signal(_, _) => todo!(),
        })
    }
    fn let_binding(&mut self, slot: Slot) -> Result<String> {
        Ok(if let Some(kind) = self.ty.get(&slot) {
            format!("{}: {}", slot, self.translate(kind)?)
        } else {
            format!("{}", slot)
        })
    }
}

fn write_rust_stub(mir: &Mir) -> Result<String> {
    // Create a rust program that will be used to typecheck the
    // MIR code.
    let mut db = MirTypeDb::new(&mir.ty);
    let mut rp = String::new();
    write!(rp, "{}", include_str!("mir_prelude.rs"))?;
    write!(rp, "fn kernel(")?;
    for slot in &mir.arguments {
        write!(
            rp,
            "{}: {},",
            slot,
            db.translate(
                mir.ty
                    .get(slot)
                    .ok_or(anyhow!("argument type {slot} not known"))?
            )?
        )?;
    }
    writeln!(
        rp,
        ") -> {} {{",
        db.translate(
            mir.ty
                .get(&mir.return_slot)
                .ok_or(anyhow!("Return slot not found"))?
        )?
    )?;
    writeln!(rp, "/*")?;
    for (slot, kind) in &mir.ty {
        writeln!(rp, "  {} -> {}", slot, db.translate(kind)?)?;
    }
    writeln!(rp, "*/")?;
    for (slot, literal) in &mir.literals {
        write!(rp, "  let {} = ", db.let_binding(*slot)?)?;
        match literal {
            ExprLit::Bool(x) => {
                writeln!(rp, "{x};")?;
            }
            ExprLit::Int(x) => {
                writeln!(rp, "{x};")?;
            }
            ExprLit::TypedBits(x) => {
                writeln!(
                    rp,
                    "<{} as Default>::default();",
                    db.translate(&x.value.kind)?
                )?;
            }
        }
    }
    for op in &mir.ops {
        let op = &op.op;
        match op {
            OpCode::Array(array) => {
                write!(rp, "  let {} = [", db.let_binding(array.lhs)?)?;
                for element in array.elements.iter() {
                    write!(rp, "{},", element)?;
                }
                writeln!(rp, "];")?;
            }
            OpCode::AsBits(as_bits) => {
                writeln!(
                    rp,
                    "  let {} = bits::<{}>({});",
                    db.let_binding(as_bits.lhs)?,
                    as_bits.len,
                    as_bits.arg,
                )?;
            }
            OpCode::AsKind(as_kind) => {
                writeln!(
                    rp,
                    "  let {} = {}.as_kind({:?});",
                    db.let_binding(as_kind.lhs)?,
                    as_kind.arg,
                    as_kind.kind
                )?;
            }
            OpCode::AsSigned(as_signed) => {
                writeln!(
                    rp,
                    "  let {} = signed::<{}>({});",
                    db.let_binding(as_signed.lhs)?,
                    as_signed.len,
                    as_signed.arg,
                )?;
            }
            OpCode::Assign(assign) => {
                writeln!(
                    rp,
                    "  let {} = {};",
                    db.let_binding(assign.lhs)?,
                    assign.rhs
                )?;
            }
            OpCode::Binary(binary) => {
                writeln!(
                    rp,
                    "  let {} = {} {} {};",
                    db.let_binding(binary.lhs)?,
                    binary.arg1,
                    binary.op,
                    binary.arg2
                )?;
            }
            OpCode::Select(select) => {
                writeln!(
                    rp,
                    "  let {} = select({}.into(),{},{});",
                    db.let_binding(select.lhs)?,
                    select.cond,
                    select.true_value,
                    select.false_value
                )?;
            }
            OpCode::Tuple(tuple) => {
                write!(rp, "  let {} = (", db.let_binding(tuple.lhs)?)?;
                for element in tuple.fields.iter() {
                    write!(rp, "{},", element)?;
                }
                writeln!(rp, "  );")?;
            }
            OpCode::Unary(unary) => match &unary.op {
                AluUnary::Any => {
                    writeln!(
                        rp,
                        "  let {} = {}.any();",
                        db.let_binding(unary.lhs)?,
                        unary.arg1
                    )?;
                }
                AluUnary::All => {
                    writeln!(
                        rp,
                        "  let {} = {}.all();",
                        db.let_binding(unary.lhs)?,
                        unary.arg1
                    )?;
                }
                AluUnary::Xor => {
                    writeln!(
                        rp,
                        "  let {} = {}.xor();",
                        db.let_binding(unary.lhs)?,
                        unary.arg1
                    )?;
                }
                AluUnary::Unsigned => {
                    writeln!(
                        rp,
                        "  let {} = {}.as_unsigned();",
                        db.let_binding(unary.lhs)?,
                        unary.arg1
                    )?;
                }
                AluUnary::Signed => {
                    writeln!(
                        rp,
                        "  let {} = {}.as_signed();",
                        db.let_binding(unary.lhs)?,
                        unary.arg1
                    )?;
                }
                AluUnary::Neg => {
                    writeln!(
                        rp,
                        "  let {} = -{};",
                        db.let_binding(unary.lhs)?,
                        unary.arg1
                    )?;
                }
                AluUnary::Not => {
                    writeln!(
                        rp,
                        "  let {} = !{};",
                        db.let_binding(unary.lhs)?,
                        unary.arg1
                    )?;
                }
            },
            OpCode::Noop => {}
            OpCode::Index(index) => {
                write!(rp, "  let {} = {}", db.let_binding(index.lhs)?, index.arg)?;
                for segment in index.path.iter() {
                    match segment {
                        PathElement::DynamicIndex(index) => {
                            write!(rp, "[{}]", index)?;
                        }
                        PathElement::TupleIndex(index) => {
                            write!(rp, ".{}", index)?;
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
                            write!(rp, ".payload_{}()", payload)?;
                        }
                        PathElement::EnumPayloadByValue(payload) => {
                            write!(rp, ".payload_by_value({})", payload)?;
                        }
                    };
                }
                writeln!(rp, ";")?;
            }
            OpCode::Splice(splice) => {
                writeln!(
                    rp,
                    "  let mut {} = {};",
                    db.let_binding(splice.lhs)?,
                    splice.orig
                )?;
                write!(rp, "  {}", splice.lhs)?;
                for segment in splice.path.iter() {
                    match segment {
                        PathElement::DynamicIndex(index) => {
                            write!(rp, "[{}]", index)?;
                        }
                        PathElement::TupleIndex(index) => {
                            write!(rp, ".{}", index)?;
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
                write!(rp, "  let {} = [", db.let_binding(repeat.lhs)?)?;
                write!(rp, "{}; {}", repeat.value, repeat.len)?;
                writeln!(rp, "];")?;
            }
            OpCode::Struct(strukt) => {
                write!(
                    rp,
                    "  let {} = {} {{",
                    db.let_binding(strukt.lhs)?,
                    db.translate(&strukt.template.kind)?
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
                write!(rp, "  let {} = if ", db.let_binding(kase.lhs)?)?;
                let mut first = true;
                let mut has_else = false;
                for arm in kase.table.iter() {
                    if let CaseArgument::Slot(t) = arm.0 {
                        if !first {
                            write!(rp, "  else if ")?;
                        } else {
                            first = false;
                        }
                        writeln!(
                            rp,
                            "  {}.discriminant() == {} {{ {} }}",
                            t, kase.discriminant, arm.1
                        )?;
                    } else {
                        has_else = true;
                        writeln!(rp, "  else {{ {} }};", arm.1)?;
                    }
                }
                if !has_else {
                    writeln!(rp, "  else {{ panic!(\"No match\") }};")?;
                }
            }
            OpCode::Exec(exec) => {
                write!(rp, "  let {} = {}(", db.let_binding(exec.lhs)?, exec.id)?;
                for (ndx, arg) in exec.args.iter().enumerate() {
                    if ndx > 0 {
                        write!(rp, ", ")?;
                    }
                    write!(rp, "{}", arg)?;
                }
                writeln!(rp, ");")?;
            }
            OpCode::Enum(enumerate) => {
                let enum_kind = &enumerate.template.kind;
                let discriminant_value = enumerate.template.discriminant()?.as_i64()?;
                let variant_name = enumerate
                    .template
                    .kind
                    .lookup_variant_name_by_discriminant(discriminant_value)?;
                let variant = enumerate.template.kind.lookup_variant(discriminant_value)?;
                if variant.is_tuple() {
                    write!(
                        rp,
                        "  let {} = {}::{}(",
                        db.let_binding(enumerate.lhs)?,
                        db.translate(enum_kind)?,
                        variant_name,
                    )?;
                    for field in enumerate.fields.iter() {
                        write!(rp, "{},", field.value)?;
                    }
                    writeln!(rp, ");")?;
                } else {
                    write!(
                        rp,
                        "  let {} = {}::{} {{",
                        db.let_binding(enumerate.lhs)?,
                        db.translate(enum_kind)?,
                        variant_name,
                    )?;
                    for field in enumerate.fields.iter() {
                        write!(rp, "    {}: {},", field.member, field.value)?;
                    }
                    writeln!(rp, "  }};")?;
                }
            }
            OpCode::Comment(_) => {}
        }
    }
    writeln!(rp, "  {}", mir.return_slot)?;
    writeln!(rp, "}}")?;
    let pre_amble = db.strukts.into_values().collect::<Vec<_>>().join("\n");
    Ok(format!("{}\n{}", pre_amble, rp))
}

pub fn infer(mir: &Mir) -> Result<()> {
    // Write the rust code to a dummy cargo project and use clippy to check it
    // for errors.
    let dir = tempfile::tempdir()?;
    // Write a Cargo manifest for this
    let manifest = dir.path().join("Cargo.toml");
    std::fs::write(
        &manifest,
        r#"
[package]
name = "mir_type_check"
version = "0.1.0"
edition = "2018"
"#,
    )?;
    let src = dir.path().join("src");
    std::fs::create_dir(&src)?;
    let src = src.join("lib.rs");
    let code = write_rust_stub(mir)?;
    eprintln!("Code: {}", code);
    std::fs::write(src, code)?;
    let output = std::process::Command::new("cargo")
        .arg("clippy")
        .current_dir(&dir)
        .output()?;
    eprintln!("Output status: {}", output.status);
    eprintln!("Output stdout: {}", String::from_utf8_lossy(&output.stdout));
    eprintln!("Output stderr: {}", String::from_utf8_lossy(&output.stderr));
    if output.status.success() {
        return Ok(());
    }
    bail!("Type check failed");
}
