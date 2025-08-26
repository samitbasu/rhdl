use log::debug;
use std::collections::BTreeMap;

use super::pass::Pass;
use crate::rhdl_core::{
    Kind, TypedBits,
    compiler::mir::ty::SignFlag,
    error::RHDLError,
    rhif::{
        Object,
        spec::{OpCode, Slot},
    },
};

#[derive(Default, Debug, Clone)]
pub struct PrecastIntegerLiteralsInBinops {}

#[derive(Clone, Debug, Hash, PartialEq)]
struct CastCandidate {
    literal: TypedBits,
    cast_details: Option<(usize, SignFlag)>,
}

impl Pass for PrecastIntegerLiteralsInBinops {
    fn description() -> &'static str {
        "Precast interger literals to concrete types in binary ops"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        // We want to know the bit width assumed by the opcodes
        // for each literal.  Note that in Rust, this is handled
        // by the overload `Add<Rhs=u128>`, so that an expression like
        //  b8 + 3 -> b8 + u128
        // is valid and the result is a b8.  In RHIF, we do not support
        // this kind of operation, so we must precast the u128 to a b8.
        // This is the purpose of this pass.
        let mut generic_int_literals = input
            .symtab
            .iter_lit()
            .filter_map(|(k, (v, _loc))| {
                if matches!(v.kind, Kind::Bits(128) | Kind::Signed(128)) {
                    Some((
                        Slot::Literal(k),
                        CastCandidate {
                            literal: v.clone(),
                            cast_details: None,
                        },
                    ))
                } else {
                    None
                }
            })
            .collect::<BTreeMap<_, _>>();
        if !generic_int_literals.is_empty() {
            debug!("Code: {:?}", input);
            debug!("Generic ints: {:?}", generic_int_literals);
        }
        // Not all generic int literals are a problem.  Only those that hit the
        // operator overload for binary operations.
        input.ops.iter().for_each(|lop| {
            let op = &lop.op;
            if let OpCode::Binary(bin) = op {
                let a_is_generic = generic_int_literals.contains_key(&bin.arg1);
                let b_is_generic = generic_int_literals.contains_key(&bin.arg2);
                let a_kind = &input.kind(bin.arg1);
                let a_sign_flag = if a_kind.is_signed() {
                    SignFlag::Signed
                } else {
                    SignFlag::Unsigned
                };
                let b_kind = &input.kind(bin.arg2);
                let b_sign_flag = if b_kind.is_signed() {
                    SignFlag::Signed
                } else {
                    SignFlag::Unsigned
                };
                if a_is_generic ^ b_is_generic {
                    if a_is_generic {
                        generic_int_literals
                            .get_mut(&bin.arg1)
                            .unwrap()
                            .cast_details = Some((b_kind.bits(), b_sign_flag));
                    }
                    if b_is_generic {
                        if !bin.op.is_shift() {
                            generic_int_literals
                                .get_mut(&bin.arg2)
                                .unwrap()
                                .cast_details = Some((a_kind.bits(), a_sign_flag));
                        } else {
                            // Shifts are special.  The shift amount must be unsigned.
                            generic_int_literals
                                .get_mut(&bin.arg2)
                                .unwrap()
                                .cast_details = Some((a_kind.bits(), SignFlag::Unsigned));
                        }
                    }
                }
            }
        });
        for (k, v) in &generic_int_literals {
            if let Some((len, sign_flag)) = v.cast_details {
                // Attempt to cast the integer literal to the given size.
                // This can cause an error if the literal is too big.
                let new_tb = if sign_flag == SignFlag::Signed {
                    v.literal.signed_cast(len)
                } else {
                    v.literal.unsigned_cast(len)
                };
                if let Ok(new_tb) = new_tb {
                    input.symtab[k.lit().unwrap()] = new_tb;
                }
            }
        }
        Ok(input)
    }
}
