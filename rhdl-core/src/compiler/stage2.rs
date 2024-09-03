use crate::{rtl, RHDLError};

use super::{
    lower_rhif_to_rtl::compile_to_rtl,
    rtl_passes::{
        lower_signal_casts::LowerSignalCasts, pass::Pass,
        remove_extra_registers::RemoveExtraRegistersPass,
    },
};

type Result<T> = std::result::Result<T, RHDLError>;

pub(crate) fn compile(object: &crate::rhif::Object) -> Result<rtl::Object> {
    let rtl = compile_to_rtl(object)?;
    let rtl = LowerSignalCasts::run(rtl)?;
    let rtl = RemoveExtraRegistersPass::run(rtl)?;
    eprintln!("{rtl:?}");
    Ok(rtl)
}
