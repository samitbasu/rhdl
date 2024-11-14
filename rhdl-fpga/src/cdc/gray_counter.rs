use rhdl::prelude::*;

use crate::{core::counter, gray};

use super::synchronizer;

/// A counter crosser that uses gray code to cross the clock domain.
/// The reasons this is safe are subtle.  But the basic idea is that
/// Gray code only changes one bit at a time.  The other bits are
/// held constant.  The effect of a synchronizer is to potentially
/// delay a bit change by one clock in the destination domain.  This
/// may result in a two bit Gray code change appearing at the output
/// but the end effect (once converted back to a binary count) is still
/// correct.
///
#[derive(Clone, Circuit, CircuitDQ)]
pub struct U<R: Domain, W: Domain, const N: usize> {
    counter: Adapter<counter::U<N>, R>,
    cdc: [synchronizer::U<R, W>; N],
}
