use crate::dff;
use core::hash::Hasher;
use rhdl::{core::compiler::codegen::verilog::generate_verilog, prelude::*};

#[derive(Clone, Debug, Synchronous)]
#[rhdl(kernel=single_bit)]
#[rhdl(auto_dq)]
pub struct U {
    state: dff::U<bool>,
}

impl Default for U {
    fn default() -> Self {
        Self {
            state: dff::U::new(false),
        }
    }
}

impl SynchronousIO for U {
    type I = bool;
    type O = bool;
}

#[kernel]
pub fn single_bit(reset: Reset, i: bool, q: Q) -> (bool, D) {
    let next_state = if i { !q.state } else { q.state };
    let output = q.state;
    if reset.any() {
        (false, D { state: false })
    } else {
        (output, D { state: next_state })
    }
}

#[test]
fn test_single_bit() -> miette::Result<()> {
    let uut = U::default();
    let rtl = compile_design::<single_bit>(CompilationMode::Synchronous)?;
    eprintln!("RTL: {:?}", rtl);
    let uut_fg = build_synchronous_flow_graph(&uut.descriptor()?)?;
    let mut dot_string = vec![0_u8; 0];
    write_dot(&uut_fg, &mut dot_string).unwrap();
    // Calculate the fnv hash of the dot string
    let mut hasher = fnv::FnvHasher::default();
    hasher.write(&dot_string);
    let hash = hasher.finish();
    eprintln!("Dot hash: {:x}", hash);
    let mut dot = std::fs::File::create("single_bit.dot").unwrap();
    write_dot(&uut_fg, &mut dot).unwrap();
    //eprintln!("RTL: {:?}", rtl);
    Ok(())
}
