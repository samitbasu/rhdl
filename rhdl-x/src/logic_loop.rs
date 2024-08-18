use petgraph::algo::{feedback_arc_set, is_cyclic_directed};
use petgraph::visit::EdgeRef;
// This component contains an intentional logic loop.
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, Default)]
#[rhdl(kernel=logic_loop)]
pub struct U {
    left: crate::inverter::U,
    right: crate::inverter::U,
}

#[derive(Clone, Copy, PartialEq, Debug, Digital, Default)]
pub struct D {
    left: bool,
    right: bool,
}

#[derive(Clone, Copy, PartialEq, Debug, Digital, Default)]
pub struct Q {
    left: bool,
    right: bool,
}

impl SynchronousIO for U {
    type I = bool;
    type O = bool;
}

impl SynchronousDQ for U {
    type D = D;
    type Q = Q;
}

#[kernel]
pub fn logic_loop(_reset: bool, i: bool, q: Q) -> (bool, D) {
    let mut d = D::default();
    if i {
        d.left = q.right;
        d.right = q.left;
    }
    (q.left, d)
}

#[test]
fn test_logic_loop() -> miette::Result<()> {
    let uut = U::default();
    let uut_fg = build_synchronous_flow_graph(&uut.descriptor()?);
    // Look for loops
    if is_cyclic_directed(&uut_fg.graph) {
        let feedback = feedback_arc_set::greedy_feedback_arc_set(&uut_fg.graph);
        for edge in feedback {
            let source = edge.source();
            let dest = edge.target();
            let source = &uut_fg.graph[source];
            let dest = &uut_fg.graph[dest];
            eprintln!("{:?} -> {:?}", source, dest);
        }
        panic!("Logic loop detected");
    }
    let mut dot = std::fs::File::create("logic_loop.dot").unwrap();
    write_dot(&uut_fg, &mut dot).unwrap();
    Ok(())
}
