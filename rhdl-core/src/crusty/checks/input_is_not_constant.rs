use miette::Report;

pub(crate) fn check_input_is_not_constant(
    is: &crate::crusty::index::IndexedSchematic,
    constraint: &crate::schematic::constraints::NotConstantValuedConstraint,
) -> Result<(), Report> {
    Ok(())
}
