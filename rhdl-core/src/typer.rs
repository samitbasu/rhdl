use crate::{
    compiler::Compiler,
    digital::Digital,
    rhif::{
        AluBinary, AssignOp, BinaryOp, CopyOp, FieldOp, FieldRefOp, IfOp, Member, OpCode, RefOp,
        Slot,
    },
    rhif_type::Ty,
    Kind,
};

// Some more notes on type inference.
//
// Each register and literal needs a type variable.  This should just be a number, but for
// safety, it will be a newtype, something like struct TypeId(pub u32).
//
// Next, we need a Term to contain elements that can be stored in the unification table.
//
// It should be something like:
//
// pub enum Term {
//     Var(TypeId),
//     Ty(Ty),
//     Tuple(Vec<Term>),
// }
//
// The substitution table is a mapping from TypeId to Term.

use anyhow::{bail, Result};

// Run type inference on the RHIF.
pub fn infer_type(compiler: &mut Compiler) -> Result<()> {
    // The simplest step is to look for copy operations.
    // A copy implies the types of the two arguments are equal.
    // This is a simple case of unification.
    propogate_copies(compiler)?;
    map_assignments(compiler)?;
    map_comparisons_to_boolean(compiler)?;
    map_addresses_to_types(compiler)?;
    arithmetic_ops(compiler)?;
    boolean_ops(compiler)?;
    struct_ops(compiler)?;
    ref_struct_ops(compiler)?;
    Ok(())
}

fn map_assignments(compiler: &mut Compiler) -> Result<()> {
    let assignments = compiler
        .iter_ops()
        .filter_map(|x| match x {
            OpCode::Assign(AssignOp { lhs, rhs }) => Some((*lhs, *rhs)),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (lhs, rhs) in assignments {
        let lhs_ty = compiler.ty(lhs);
        let rhs_ty = compiler.ty(rhs);
        if let (Some(lhs_ty), None) = (lhs_ty, rhs_ty) {
            compiler.set_ty(rhs, Ty::Kind(lhs_ty.target_kind()?));
        } else if let (None, Some(rhs_ty)) = (lhs_ty, rhs_ty) {
            compiler.set_ty(lhs, Ty::Address(rhs_ty.kind()?));
        } else if let (Some(lhs_ty), Some(rhs_ty)) = (lhs_ty, rhs_ty) {
            if lhs_ty.target_kind()? != rhs_ty.kind()? {
                return Err(anyhow::anyhow!(
                    "Type mismatch in assignment operation for registers {} and {}",
                    lhs,
                    rhs
                ));
            }
        }
    }
    Ok(())
}

fn propogate_copies(compiler: &mut Compiler) -> Result<()> {
    let copies = compiler
        .iter_ops()
        .filter_map(|x| match x {
            OpCode::Copy(CopyOp { lhs, rhs }) => Some((*lhs, *rhs)),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (lhs, rhs) in copies {
        unify_type(compiler, lhs, rhs)?;
    }
    Ok(())
}

fn map_addresses_to_types(compiler: &mut Compiler) -> Result<()> {
    let addresses = compiler
        .iter_ops()
        .filter_map(|x| match x {
            OpCode::Ref(RefOp { lhs, arg }) => Some((*lhs, *arg)),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (lhs, arg) in addresses {
        let lhs_ty = compiler.ty(lhs);
        let arg_ty = compiler.ty(arg);
        if let (Some(lhs_ty), None) = (lhs_ty, arg_ty) {
            compiler.set_ty(arg, Ty::Kind(lhs_ty.target_kind()?));
        } else if let (None, Some(arg_ty)) = (lhs_ty, arg_ty) {
            compiler.set_ty(lhs, Ty::Address(arg_ty.kind()?));
        } else if let (Some(lhs_ty), Some(arg_ty)) = (lhs_ty, arg_ty) {
            if lhs_ty != &Ty::Address(arg_ty.kind()?) {
                return Err(anyhow::anyhow!(
                    "Type mismatch in address operation for registers {} and {}",
                    lhs,
                    arg
                ));
            }
            if arg_ty != &Ty::Kind(lhs_ty.target_kind()?) {
                return Err(anyhow::anyhow!(
                    "Type mismatch in address operation for registers {} and {}",
                    lhs,
                    arg
                ));
            }
        }
    }
    Ok(())
}

fn map_comparisons_to_boolean(compiler: &mut Compiler) -> Result<()> {
    let comparisons = compiler
        .iter_ops()
        .filter_map(|x| match x {
            OpCode::Binary(BinaryOp {
                op:
                    AluBinary::Ge
                    | AluBinary::Gt
                    | AluBinary::Le
                    | AluBinary::Lt
                    | AluBinary::Eq
                    | AluBinary::Ne,
                lhs,
                ..
            }) => Some(lhs),
            OpCode::If(IfOp { cond, .. }) => Some(cond),
            _ => None,
        })
        .cloned()
        .collect::<Vec<_>>();
    for lhs in comparisons {
        constrain_type(compiler, lhs, &Ty::Kind(bool::static_kind()))?;
    }
    Ok(())
}

fn unify_type(compiler: &mut Compiler, lhs: Slot, rhs: Slot) -> Result<()> {
    let ty_lhs = compiler.ty(lhs);
    let ty_rhs = compiler.ty(rhs);
    if ty_lhs != ty_rhs {
        // The types are unequal...
        // Check for lhs is defined, and rhs is not
        if let (Some(ty_lhs), None) = (ty_lhs, ty_rhs) {
            // Propogate the type of lhs to rhs
            println!("Propogating type {:?} to {:?}", ty_lhs, rhs);
            compiler.set_ty(rhs, ty_lhs.clone());
        } else if let (None, Some(ty_rhs)) = (ty_lhs, ty_rhs) {
            // Propogate the type of rhs to lhs
            println!("Propogating type {:?} to {:?}", ty_rhs, lhs);
            compiler.set_ty(lhs, ty_rhs.clone());
        } else {
            // Both are defined, but unequal.
            // This is an error.
            return Err(anyhow::anyhow!(
                "Type mismatch in copy operation for slots {} and {}",
                lhs,
                rhs
            ));
        }
    }
    Ok(())
}

fn constrain_type(compiler: &mut Compiler, lhs: Slot, ty: &Ty) -> Result<()> {
    let ty_lhs = compiler.ty(lhs);
    if let Some(ty_lhs) = ty_lhs {
        if ty_lhs != ty {
            return Err(anyhow::anyhow!(
                "Type mismatch in constraint operation for register {} (expected {:?}, found {:?})",
                lhs,
                ty,
                ty_lhs
            ));
        }
    } else {
        compiler.set_ty(lhs, ty.clone());
    }
    Ok(())
}

fn arithmetic_ops(compiler: &mut Compiler) -> Result<()> {
    let ops = compiler
        .iter_ops()
        .filter_map(|x| match x {
            OpCode::Binary(BinaryOp {
                op:
                    AluBinary::Add
                    | AluBinary::Sub
                    | AluBinary::Mul
                    | AluBinary::BitXor
                    | AluBinary::BitOr
                    | AluBinary::BitAnd,
                lhs,
                arg1,
                arg2,
            }) => Some((*lhs, *arg1, *arg2)),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (lhs, arg1, arg2) in ops {
        unify_type(compiler, lhs, arg1)?;
        unify_type(compiler, lhs, arg2)?;
        unify_type(compiler, arg1, arg2)?;
    }
    Ok(())
}

fn boolean_ops(compiler: &mut Compiler) -> Result<()> {
    let ops = compiler
        .iter_ops()
        .filter_map(|x| match x {
            OpCode::Binary(BinaryOp {
                op: AluBinary::And | AluBinary::Or,
                lhs,
                arg1,
                arg2,
            }) => Some((*lhs, *arg1, *arg2)),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (lhs, arg1, arg2) in ops {
        constrain_type(compiler, lhs, &Ty::Kind(bool::static_kind()))?;
        constrain_type(compiler, arg1, &Ty::Kind(bool::static_kind()))?;
        constrain_type(compiler, arg2, &Ty::Kind(bool::static_kind()))?;
    }
    Ok(())
}

fn struct_ops(compiler: &mut Compiler) -> Result<()> {
    let ops = compiler
        .iter_ops()
        .filter_map(|x| match x {
            OpCode::Field(FieldOp { lhs, arg, member }) => Some((*lhs, *arg, member.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (lhs, arg, member) in ops {
        let arg_ty = compiler.ty(arg);
        if let Some(arg_ty) = arg_ty {
            let arg_kind = arg_ty.kind()?;
            let sub_kind = get_field_of_type(&arg_kind, &member)?;
            constrain_type(compiler, lhs, &Ty::Kind(sub_kind))?;
        }
    }
    Ok(())
}

fn ref_struct_ops(compiler: &mut Compiler) -> Result<()> {
    let ops = compiler
        .iter_ops()
        .filter_map(|x| match x {
            OpCode::FieldRef(FieldRefOp { lhs, arg, member }) => Some((*lhs, *arg, member.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (lhs, arg, member) in ops {
        let arg_ty = compiler.ty(arg);
        if let Some(arg_ty) = arg_ty {
            let arg_kind = arg_ty.target_kind()?;
            let sub_kind = get_field_of_type(&arg_kind, &member)?;
            constrain_type(compiler, lhs, &Ty::Address(sub_kind))?;
        }
    }
    Ok(())
}

fn get_field_of_type(base_type: &Kind, member: &Member) -> Result<Kind> {
    if let (Kind::Struct(struct_kind), Member::Named(field)) = (base_type, member) {
        // Get the field with the given name or return an error
        if let Some(kind) = struct_kind.fields.iter().find(|x| &x.name == field) {
            Ok(kind.kind.clone())
        } else {
            bail!("Field {} not found in struct", field)
        }
    } else if let (Kind::Tuple(tuple_kind), Member::Unnamed(ndx)) = (base_type, member) {
        if let Some(kind) = tuple_kind.elements.get(*ndx as usize) {
            Ok(kind.clone())
        } else {
            bail!("Field {} not found in tuple", ndx)
        }
    } else {
        bail!("Expected struct or tuple type, found {:?}", base_type)
    }
}
