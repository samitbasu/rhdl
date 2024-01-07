mod digital;
mod kind;
pub use kind::Kind;
pub mod clock_details;
pub mod tag_id;

pub use clock_details::ClockDetails;
pub use digital::Digital;
pub use digital_fn::DigitalFn;
pub use kernel::KernelFnKind;
pub use kind::DiscriminantAlignment;
pub use tag_id::TagID;

#[cfg(feature = "svg")]
pub use kind::kind_svg::svg_grid;
#[cfg(feature = "svg")]
pub use kind::kind_svg::svg_grid_vertical;

pub use kind::text_grid;
pub mod ascii;
pub mod ast;
pub mod ast_builder;
pub mod codegen;
pub mod compiler;
pub mod design;
pub mod digital_fn;
pub mod display_ast;
pub mod display_rhif;
pub mod dyn_bit_manip;
pub mod kernel;
pub mod note;
pub mod note_db;
pub mod path;
pub mod test_module;
pub mod typed_bits;
pub mod util;
pub mod visit;
pub mod visit_mut;

#[cfg(feature = "iverilog")]
pub use test_module::test_with_iverilog;

pub use note::NoteKey;
pub use note::NoteWriter;
pub use note_db::note;
pub use note_db::note_init_db;
pub use note_db::note_pop_path;
pub use note_db::note_push_path;
pub use note_db::note_take;
pub use note_db::note_time;
pub mod check_inference;
pub mod check_rhif_type;
pub use check_inference::check_inference;
pub use check_rhif_type::check_type_correctness;
pub use kind::DiscriminantType;
pub use typed_bits::TypedBits;
pub mod check_rhif_flow;
pub use check_rhif_flow::check_rhif_flow;
pub use codegen::verilog::as_verilog_literal;
pub use codegen::verilog::generate_verilog;
pub use codegen::verilog::VerilogModule;
pub use compiler::compile_design;
pub use note_db::NoteDB;
pub use test_module::test_kernel_vm_and_verilog;
pub mod rhif;
