use log::{debug, info};

use crate::{
    prelude::RHDLError,
    rhdl_core::{
        compiler::ntl_passes::{
            constant_propagation::ConstantPropagationPass,
            constant_reg_elimination::ConstantRegisterElimination, lower_case::LowerCase,
            lower_selects::LowerSelects, pass::Pass,
            remove_extra_registers::RemoveExtraRegistersPass,
            reorder_instructions::ReorderInstructions,
        },
        ntl::Object,
    },
};

fn wrap_pass<P: Pass>(obj: Object) -> Result<Object, RHDLError> {
    info!("Running Stage 3 compiler Pass {}", P::description());
    P::run(obj)
}

pub fn optimize_ntl(mut input: Object) -> Result<Object, RHDLError> {
    let mut hash = input.hash_value();
    debug!("NTL before optimize {:?}", input);
    loop {
        input = wrap_pass::<ConstantRegisterElimination>(input)?;
        input = wrap_pass::<LowerCase>(input)?;
        input = wrap_pass::<LowerSelects>(input)?;
        input = wrap_pass::<RemoveExtraRegistersPass>(input)?;
        input = wrap_pass::<ConstantPropagationPass>(input)?;
        let new_hash = input.hash_value();
        if new_hash == hash {
            break;
        }
        hash = new_hash;
    }
    input = wrap_pass::<ReorderInstructions>(input)?;
    debug!("NTL after optimize {:?}", input);
    Ok(input)
}
