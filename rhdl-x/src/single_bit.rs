use crate::dff;
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQZ)]
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
    type Kernel = single_bit;
}

#[kernel]
pub fn single_bit(cr: ClockReset, i: bool, q: Q) -> (bool, D) {
    let next_state = if i { !q.state } else { q.state };
    let output = q.state;
    if cr.reset.any() {
        (false, D { state: false })
    } else {
        (output, D { state: next_state })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::hash::Hasher;

    #[test]
    fn test_single_bit() -> miette::Result<()> {
        let uut = U::default();
        let rtl = compile_design::<single_bit>(CompilationMode::Synchronous)?;
        eprintln!("RTL: {:?}", rtl);
        let uut_fg = &uut.descriptor("uut")?.flow_graph;
        let mut dot_string = vec![0_u8; 0];
        write_dot(uut_fg, &mut dot_string).unwrap();
        // Calculate the fnv hash of the dot string
        let mut hasher = fnv::FnvHasher::default();
        hasher.write(&dot_string);
        let hash = hasher.finish();
        eprintln!("Dot hash: {:x}", hash);
        let mut dot = std::fs::File::create("single_bit.dot").unwrap();
        write_dot(uut_fg, &mut dot).unwrap();
        //eprintln!("RTL: {:?}", rtl);
        Ok(())
    }
}
