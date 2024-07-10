use rhdl::prelude::*;

use crate::dff;

#[derive(PartialEq, Clone, Copy, Debug, Digital)]
pub struct I {
    pub enable: bool,
}

#[derive(Clone, Debug, Synchronous)]
#[rhdl(kernel=counter::<{N}>)]
#[rhdl(auto_dq)]
pub struct U<const N: usize> {
    count: dff::U<Bits<N>>,
}

#[derive(Clone, Debug, PartialEq, Copy, Digital)]
struct MyD<const N: usize> {
    count: Bits<N>,
}

#[derive(Clone, Debug, PartialEq, Copy, Digital)]
struct MyQ<const N: usize> {
    count: Bits<N>,
}

impl<const N: usize> U<N> {
    pub fn new() -> Self {
        Self {
            count: dff::U::new(Bits::ZERO),
        }
    }
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = I;
    type O = Bits<N>;
}

#[kernel]
pub fn counter<const N: usize>(reset: bool, i: I, q: Q<N>) -> (Bits<N>, D<N>) {
    let next_count = if i.enable { q.count + 1 } else { q.count };
    let output = q.count;
    if reset {
        (bits(0), D::<{ N }> { count: bits(0) })
    } else {
        (output, D::<{ N }> { count: next_count })
    }
}

struct TestComputer {}

impl CostEstimator for TestComputer {
    fn cost(&self, obj: &Object, opcode: usize) -> f64 {
        if matches!(obj.ops[opcode], OpCode::Binary(_) | OpCode::Select(_)) {
            -1.0
        } else {
            0.0
        }
    }
}

#[test]
fn test_counter_timing_root() -> miette::Result<()> {
    let uut: U<4> = U::new();
    let uut_module = compile_design::<<U<4> as Synchronous>::Update>(CompilationMode::Synchronous)?;
    let top = uut_module.objects[&uut_module.top];
    let path = rhdl::core::types::path::Path::default().tuple_index(1);
    let timing = compute_timing_graph(&uut_module, uut_module.top, &path, &TestComputer {})?;
    eprintln!("timing: {:?}", timing);
    Ok(())
}

/*
The function to compute timing needs to look something like this:

fn timing(path: &Path, computer: &'dyn CostEstimator) -> Result<CostGraph, RHDLError> {
    let module = compile_design::<<Self as Synchronous>::Update>(CompilationMode::Synchronous)?;
    let top = &module.objects[&module.top];
    let timing = compute_timing_graph(&module, module.top, path, computer)?;
    // Check for inputs to the timing graph that come via the Q path
    let target_argument = top.arguments[2]; // The arguments are Reset, Input, Q
    for input in timing.inputs {
        if input.slot == target_argument {
        }
    }
}

*/
