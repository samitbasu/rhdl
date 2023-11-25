mod digital;
mod kind;
pub use kind::Kind;
pub mod clock_details;
pub mod tag_id;

pub use clock_details::ClockDetails;
pub use digital::Digital;
pub use kind::DiscriminantAlignment;
pub use tag_id::TagID;

#[cfg(feature = "svg")]
pub use kind::kind_svg::svg_grid;
#[cfg(feature = "svg")]
pub use kind::kind_svg::svg_grid_vertical;

pub use kind::text_grid;
pub mod ascii;
pub mod assign_node;
pub mod ast;
pub mod ast_builder;
pub mod compiler;
pub mod display_ast;
pub mod display_rhif;
//pub mod dot;
pub mod digital_fn;
pub mod infer_types;
pub mod kernel;
pub mod note;
pub mod path;
pub mod rhif;
pub mod rhif_type;
pub mod test_module;
pub mod ty;
pub mod typer;
pub mod unify;
pub mod util;
pub mod visit;
pub mod visit_mut;

#[cfg(feature = "iverilog")]
pub use test_module::test_with_iverilog;
