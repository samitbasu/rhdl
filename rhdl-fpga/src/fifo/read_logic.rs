use rhdl::prelude::*;
use crate::core::dff;

/// The read side of a FIFO.  The logic for reading needs to know the following:
///  1. - the depth of the FIFO (in elements) - essentially the size of the FIFO in bits
///  2. - the current number of elements in the FIFO (upper bound)
///  3. - the current read address
/// 
/// The current number of elements in the FIFO is stored by the read side in a counter.
/// The read side state machine will need a signal to indicate that more elements are 
/// available to read.  
/// 
/// 


#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<const N: usize> {
    read_address: dff::U<Bits<N>>,
    underflow: dff::U<bool>,
}


#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct I<const N: usize> {
    write_address: Bits<N>,
    next: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct O<const N: usize> {
    empty: bool,
    underflow: bool,
    read_address: Bits<N>,
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = I<N>;
    type O = O<N>;
    type Kernel = read_logic<N>;
}


#[kernel]
pub fn read_logic<const N: usize>(cr: ClockReset, i: I<N>, q: Q<N>) -> (O<N>, D<N>) {
    // Compute the empty flag
    let empty = i.write_address == q.read_address;
    // Check for an underflow condition 
    // We will underflow if we try to read when the FIFO is empty
    // and the condition is latching.
    let underflow = q.underflow || (i.next && empty);
    // Decide if we will read
    let will_read = !empty && i.next;
    // If we will read, advance the read address
    let read_address = q.read_address + if will_read {1} else {0};
    let mut d = D::<{N}>::init();
    d.read_address = read_address;
    d.underflow = underflow;
    let mut o = O::<{N}>::init();
    o.empty = empty;
    o.read_address = read_address;
    o.underflow = underflow;
    // Handle the reset logic
    if cr.reset.any() {
        d.read_address = bits(0);
        d.underflow = false;
        o.empty = true;
        o.read_address = bits(0);
        o.underflow = false;
    }
    (o, d)
}


#[cfg(test)]
mod tests {
    use super::*;

}    
}