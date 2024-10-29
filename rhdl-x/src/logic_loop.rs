// This component contains an intentional logic loop.
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, Default)]
pub struct U {
    left: crate::inverter::U,
    right: crate::inverter::U,
}

#[derive(Clone, Copy, PartialEq, Debug, Notable, Digital, Default)]
pub struct D {
    left: bool,
    right: bool,
}

#[derive(Clone, Copy, PartialEq, Debug, Notable, Digital, Default)]
pub struct Q {
    left: bool,
    right: bool,
}

impl SynchronousIO for U {
    type I = bool;
    type O = bool;
    type Kernel = logic_loop;
}

impl SynchronousDQZ for U {
    type D = D;
    type Q = Q;
    type Z = ((), ());
}

#[kernel]
pub fn logic_loop(_cr: ClockReset, i: bool, q: Q) -> (bool, D) {
    let mut d = D::default();
    if i {
        d.left = q.right;
        d.right = q.left;
    }
    (q.left, d)
}

#[cfg(test)]
mod tests {

    use petgraph::{
        algo::{feedback_arc_set, is_cyclic_directed},
        visit::EdgeRef,
    };

    use super::*;

    #[test]
    fn test_logic_loop() -> miette::Result<()> {
        let uut = U::default();
        let uut_fg = &uut.descriptor("uut")?.flow_graph;
        let mut dot = std::fs::File::create("logic_loop.dot").unwrap();
        write_dot(uut_fg, &mut dot).unwrap();

        // Look for loops
        assert!(is_cyclic_directed(&uut_fg.graph));
        if is_cyclic_directed(&uut_fg.graph) {
            let feedback = feedback_arc_set::greedy_feedback_arc_set(&uut_fg.graph);
            for edge in feedback {
                let source = edge.source();
                let dest = edge.target();
                let source = &uut_fg.graph[source];
                let dest = &uut_fg.graph[dest];
                eprintln!("{:?} -> {:?}", source, dest);
            }
        } else {
            panic!("No loop found");
        }
        let mut dot = std::fs::File::create("logic_loop.dot").unwrap();
        write_dot(uut_fg, &mut dot).unwrap();
        Ok(())
    }
}
