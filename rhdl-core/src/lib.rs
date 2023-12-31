pub use types::kind::Kind;
pub mod clock_details;

pub use clock_details::ClockDetails;
pub use types::digital::Digital;
pub use types::digital_fn::DigitalFn;
pub use types::kernel::KernelFnKind;
pub use types::kind::DiscriminantAlignment;

#[cfg(feature = "svg")]
pub use types::kind::kind_svg::svg_grid;
#[cfg(feature = "svg")]
pub use types::kind::kind_svg::svg_grid_vertical;

pub use types::kind::text_grid;
pub mod ast;
pub mod codegen;
pub mod compiler;
pub mod dyn_bit_manip;
pub mod note_db;
pub mod path;
pub mod test_module;
pub mod types;
pub mod util;

#[cfg(feature = "iverilog")]
pub use test_module::test_with_iverilog;

pub use codegen::verilog::as_verilog_literal;
pub use codegen::verilog::generate_verilog;
pub use codegen::verilog::VerilogModule;
pub use compiler::compile_design;
pub use note_db::note;
pub use note_db::note_init_db;
pub use note_db::note_pop_path;
pub use note_db::note_push_path;
pub use note_db::note_take;
pub use note_db::note_time;
pub use note_db::NoteDB;
pub use test_module::test_kernel_vm_and_verilog;
pub use types::kind::DiscriminantType;
pub use types::note::NoteKey;
pub use types::note::NoteWriter;
pub use types::typed_bits::TypedBits;
pub mod rhif;
pub use ast::ast_builder;
pub use rhif::design::Design;
pub use types::digital_fn;
pub use types::digital_fn::DigitalSignature;
pub use types::kernel;
pub use types::synchronous::Synchronous;
pub use types::synchronous::UpdateFn;
