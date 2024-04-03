use miette::Report;

pub(crate) fn check_output_is_synchronous(
    is: &crate::crusty::index::IndexedSchematic,
    c: &crate::schematic::constraints::OutputSynchronousConstraint,
) -> Result<(), Report> {
    Ok(())
}
