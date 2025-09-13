use std::collections::HashMap;

use crate::{
    {BitX, RHDLError},
    {
        common::symtab::Symbol,
        compiler::ntl_passes::pass::Pass,
        ntl::{Object, spec::Wire, visit::visit_object_wires_mut},
    },
};

#[derive(Default, Debug, Clone)]
pub struct RemoveExtraLiteralsPass {}

// Rewrite all literals with `replace_val` to point to `lit`
fn rewrite_lits(input: &mut Object, lit: Wire, replace_val: BitX) {
    let remap = input
        .symtab
        .iter_lit()
        .filter_map(|(lid, (value, _))| {
            if value == &replace_val {
                Some((Symbol::Literal(lid), lit))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();
    visit_object_wires_mut(input, |_sense, wire| {
        if let Some(replacement) = remap.get(wire) {
            *wire = *replacement;
        }
    });
}

fn get_root_literal(input: &Object, target: BitX) -> Option<Wire> {
    input.symtab.iter_lit().find_map(|(lid, (value, _))| {
        if value == &target {
            Some(Symbol::Literal(lid))
        } else {
            None
        }
    })
}

impl Pass for RemoveExtraLiteralsPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        if let Some(bit_zero) = get_root_literal(&input, BitX::Zero) {
            rewrite_lits(&mut input, bit_zero, BitX::Zero);
        }
        if let Some(bit_one) = get_root_literal(&input, BitX::One) {
            rewrite_lits(&mut input, bit_one, BitX::One);
        }
        if let Some(bit_x) = get_root_literal(&input, BitX::X) {
            rewrite_lits(&mut input, bit_x, BitX::X);
        }
        Ok(input)
    }

    fn description() -> &'static str {
        "Remove all extra literals - collapse all literals down to a single representation"
    }
}
