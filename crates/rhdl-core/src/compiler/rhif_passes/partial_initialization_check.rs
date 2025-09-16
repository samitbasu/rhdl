use std::collections::BTreeMap;

use log::trace;

use crate::{
    BitX, RHDLError, TypedBits,
    ast::{KernelFlags, SourceLocation},
    compiler::mir::error::RHDLPartialInitializationError,
    error::rhdl_error,
    rhif::{
        Object,
        spec::{OpCode, Slot},
    },
    types::path::{Path, bit_range, leaf_paths},
};

use super::pass::Pass;

pub struct PartialInitializationCheck;

// Give a variable that is either a struct, tuple or array,
// We want to prove that it cannot be partially initialized.

impl Pass for PartialInitializationCheck {
    fn description() -> &'static str {
        "Check for incomplete initialization"
    }
    fn run(input: Object) -> Result<Object, RHDLError> {
        let mut map: CoverageMap = CoverageMap {
            obj: &input,
            map: Default::default(),
        };
        check_for_partial_initialization(&mut map)?;
        Ok(input)
    }
}

struct CoverageMap<'a> {
    obj: &'a Object,
    map: BTreeMap<Slot, Vec<bool>>,
}

impl CoverageMap<'_> {
    fn build_coverage_details(&self, slot: Slot) -> String {
        let kind = self.obj.kind(slot);
        let paths = leaf_paths(&kind, Path::default());
        let mut details = String::new();
        let coverage = self.map.get(&slot).cloned().unwrap_or_default();
        for path in paths {
            let (bits, _) = bit_range(kind, &path).unwrap();
            let covered = coverage[bits].iter().all(|b| *b);
            if !covered {
                details.push_str(&format!("Path {path:?} is not covered\n"));
            }
        }
        details
    }
    fn raise_error(&self, slot: Slot, loc: SourceLocation) -> RHDLError {
        let fn_loc = self.obj.symbols.fallback(self.obj.fn_id);
        rhdl_error(RHDLPartialInitializationError {
            src: self.obj.symbols.source(),
            err_span: self.obj.symbols.span(loc).into(),
            fn_span: self.obj.symbols.span(fn_loc).into(),
            details: self.build_coverage_details(slot),
        })
    }
    fn declare_covered(&mut self, slot: Slot) {
        trace!("Slot {slot:?} is covered");
        if let Some(entry) = self.map.get_mut(&slot) {
            entry.iter_mut().for_each(|b| *b = true);
        } else {
            let kind = self.obj.kind(slot);
            let all_covered = std::iter::repeat_n(true, kind.bits()).collect();
            self.map.insert(slot, all_covered);
        }
    }
    fn ensure_covered(&self, slot: Slot, loc: SourceLocation) -> Result<(), RHDLError> {
        trace!("Ensure {slot:?} is covered");
        if self.obj.kind(slot).is_empty() {
            return Ok(());
        }
        if let Some(entry) = self.map.get(&slot) {
            if entry.iter().all(|b| *b) {
                Ok(())
            } else {
                Err(self.raise_error(slot, loc))
            }
        } else {
            Err(self.raise_error(slot, loc))
        }
    }
    fn coverage(&mut self, slot: Slot) -> Vec<bool> {
        self.map
            .entry(slot)
            .or_insert_with(|| {
                let kind = self.obj.kind(slot);
                std::iter::repeat_n(false, kind.bits()).collect()
            })
            .clone()
    }
    fn cover(&mut self, slot: Slot, coverage: Vec<bool>) {
        trace!(
            "Cover {slot:?} with pattern {}",
            coverage
                .iter()
                .rev()
                .map(|x| if *x { '1' } else { '0' })
                .collect::<String>()
        );
        self.map.insert(slot, coverage);
    }
}

fn typed_bit_cover(tb: &TypedBits) -> Vec<bool> {
    tb.bits.iter().map(|b| *b != BitX::X).collect()
}

fn merge_cover(a: &mut [bool], b: &[bool], weak_mode: bool) {
    a.iter_mut()
        .zip(b.iter())
        .for_each(|(a, b)| if weak_mode { *a |= *b } else { *a &= *b });
}

fn check_for_partial_initialization(map: &mut CoverageMap) -> Result<(), RHDLError> {
    let obj = map.obj;
    let weak_mode = obj
        .flags
        .iter()
        .any(|x| matches!(x, KernelFlags::AllowWeakPartial));
    obj.arguments.iter().for_each(|arg| {
        map.declare_covered(Slot::Register(*arg));
    });
    // Check the literals...  For enums, we declare them covered
    for (literal, (tb, _)) in obj.symtab.iter_lit() {
        if tb.kind.is_enum() {
            map.declare_covered(Slot::Literal(literal));
            continue;
        }
        let coverage = typed_bit_cover(tb);
        map.cover(Slot::Literal(literal), coverage);
    }
    for lop in &obj.ops {
        trace!("Analyzing op {:?}", lop.op);
        let op = &lop.op;
        let loc = lop.loc;
        match op {
            OpCode::Noop => {}
            OpCode::Binary(inner) => {
                map.ensure_covered(inner.arg1, loc)?;
                map.ensure_covered(inner.arg2, loc)?;
                map.declare_covered(inner.lhs);
            }
            OpCode::Array(inner) => {
                let rhs = inner
                    .elements
                    .iter()
                    .flat_map(|slot| map.coverage(*slot))
                    .collect::<Vec<_>>();
                map.cover(inner.lhs, rhs);
            }
            OpCode::Assign(inner) => {
                let rhs = map.coverage(inner.rhs);
                map.cover(inner.lhs, rhs);
            }
            OpCode::Resize(inner) | OpCode::AsBits(inner) | OpCode::AsSigned(inner) => {
                map.ensure_covered(inner.arg, loc)?;
                map.declare_covered(inner.lhs);
            }
            OpCode::Retime(inner) => {
                map.ensure_covered(inner.arg, loc)?;
                map.declare_covered(inner.lhs);
            }
            OpCode::Wrap(inner) => {
                map.ensure_covered(inner.arg, loc)?;
                map.declare_covered(inner.lhs);
            }
            OpCode::Exec(inner) => {
                inner
                    .args
                    .iter()
                    .try_for_each(|arg| map.ensure_covered(*arg, loc))?;
                map.declare_covered(inner.lhs);
            }
            OpCode::Unary(inner) => {
                map.ensure_covered(inner.arg1, loc)?;
                map.declare_covered(inner.lhs);
            }
            OpCode::Select(inner) => {
                map.ensure_covered(inner.cond, loc)?;
                let mut true_cover = map.coverage(inner.true_value);
                let false_cover = map.coverage(inner.false_value);
                merge_cover(&mut true_cover, &false_cover, weak_mode);
                map.cover(inner.lhs, true_cover);
            }
            OpCode::Index(inner) => {
                if inner.path.any_dynamic() {
                    map.ensure_covered(inner.arg, loc)?;
                    for slot in inner.path.dynamic_slots() {
                        map.ensure_covered(*slot, loc)?;
                    }
                    map.declare_covered(inner.lhs);
                } else {
                    let kind = obj.kind(inner.arg);
                    let (bits, _) = bit_range(kind, &inner.path)?;
                    let arg = map.coverage(inner.arg);
                    map.cover(inner.lhs, arg[bits].to_vec());
                }
            }
            OpCode::Splice(inner) => {
                if inner.path.any_dynamic() {
                    let arg = map.coverage(inner.orig);
                    map.cover(inner.lhs, arg);
                } else {
                    let kind = obj.kind(inner.orig);
                    let (bits, _) = bit_range(kind, &inner.path)?;
                    let orig = map.coverage(inner.orig);
                    let subst = map.coverage(inner.subst);
                    let mut lhs = orig.clone();
                    lhs.splice(bits, subst);
                    map.cover(inner.lhs, lhs);
                }
            }
            OpCode::Repeat(inner) => {
                let value = map.coverage(inner.value);
                let value_len = value.len();
                let lhs = value
                    .iter()
                    .copied()
                    .cycle()
                    .take(inner.len as usize * value_len)
                    .collect();
                map.cover(inner.lhs, lhs);
            }
            OpCode::Struct(inner) => {
                let kind = obj.kind(inner.lhs);
                let mut template = typed_bit_cover(&inner.template);
                if let Some(rest) = inner.rest {
                    template = map.coverage(rest);
                }
                for field in inner.fields.iter() {
                    let path = Path::default().member(&field.member);
                    let (bits, _) = bit_range(kind, &path)?;
                    let value = map.coverage(field.value);
                    template.splice(bits, value);
                }
                map.cover(inner.lhs, template);
            }
            OpCode::Tuple(inner) => {
                let coverage = inner
                    .fields
                    .iter()
                    .flat_map(|field| map.coverage(*field))
                    .collect();
                map.cover(inner.lhs, coverage);
            }
            OpCode::Case(case) => {
                map.ensure_covered(case.discriminant, loc)?;
                let lhs_kind = obj.kind(case.lhs);
                let mut lhs_cover =
                    std::iter::repeat_n(true, lhs_kind.bits()).collect::<Vec<bool>>();
                for (_, val) in &case.table {
                    let case_cover = map.coverage(*val);
                    merge_cover(&mut lhs_cover, &case_cover, weak_mode);
                }
                map.cover(case.lhs, lhs_cover);
            }
            OpCode::Enum(inner) => {
                inner
                    .fields
                    .iter()
                    .map(|field| map.ensure_covered(field.value, loc))
                    .collect::<Result<Vec<_>, _>>()?;
                map.declare_covered(inner.lhs);
            }
            OpCode::Comment(_) => {}
        }
    }
    // Check that the return value is fully initialized
    let fallback_position = obj.symbols.fallback(obj.fn_id);
    map.ensure_covered(obj.return_slot, fallback_position)?;
    Ok(())
}
