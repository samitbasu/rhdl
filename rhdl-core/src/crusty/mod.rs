use crate::{schematic::components::ComponentKind, BlackBoxTrait, Constraint};

use self::index::IndexedSchematic;

pub mod checks;
pub mod downstream;
pub mod index;
pub mod source_pool;
pub mod timing;
pub mod upstream;
pub mod utils;

pub fn check_schematic(is: &mut IndexedSchematic) -> Vec<miette::Report> {
    let constraints = is
        .schematic
        .components
        .iter()
        .flat_map(|component| match &component.kind {
            ComponentKind::BlackBox(bb) => bb.constraints(),
            _ => vec![],
        })
        .collect::<Vec<_>>();
    constraints
        .iter()
        .flat_map(|constraint| {
            match constraint {
                Constraint::MustClock(c) => checks::pin_is_clocked::check_pin_is_clocked(is, c),
                Constraint::NotConstantValued(c) => {
                    checks::input_is_not_constant::check_input_is_not_constant(is, c)
                }
                Constraint::OutputSynchronous(c) => {
                    checks::output_is_synchronous::check_output_is_synchronous(is, c)
                }
                Constraint::InputSynchronous(c) => {
                    checks::input_is_synchronous::check_input_is_synchronous(is, c)
                }
            }
            .err()
        })
        .collect()
}
