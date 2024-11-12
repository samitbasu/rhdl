use crate::core::dff;
use rhdl::prelude::*;

/// The read side of a FIFO.  In this design (which is meant to be maximally
/// simple, but also as robust as possible), the read side of the FIFO stores
/// an internal read address and an underflow flag.  The write address (owned by
/// the write side of the FIFO) is provided to the read logic as an input.
///
/// Critical assumption:
///   - We assume that the write address received from the write side of the FIFO
///   is conservative, meaning that the real write location is at least as great
///   as the write address provided - i.e., that the writer has definitely put
///   data into the given address when we get it.  It may have written additional
///   data that we don't know about, but we can be sure that the data at the
///   write address is valid.
///
/// Note that this design will waste a slot in the FIFO when the read and write
/// addresses are equal, as it cannot otherwise distinguish between a full and
/// empty FIFO.  So for N bits, this design can store 2^N-1 elements.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<const N: usize> {
    read_address: dff::U<Bits<N>>,
    underflow: dff::U<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct I<const N: usize> {
    pub write_address: Bits<N>,
    pub next: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct O<const N: usize> {
    pub empty: bool,
    pub almost_empty: bool,
    pub underflow: bool,
    pub read_address: Bits<N>,
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
    // Compute the almost empty flag
    let almost_empty = empty || (q.read_address + 1) == i.write_address;
    // Check for an underflow condition
    // We will underflow if we try to read when the FIFO is empty
    // and the condition is latching.
    let underflow = q.underflow || (i.next && empty);
    // Decide if we will read
    let will_read = !empty && i.next;
    // If we will read, advance the read address
    let read_address = q.read_address + if will_read { 1 } else { 0 };
    let mut d = D::<{ N }>::init();
    d.read_address = read_address;
    d.underflow = underflow;
    let mut o = O::<{ N }>::init();
    o.empty = empty;
    o.almost_empty = almost_empty;
    o.read_address = read_address;
    o.underflow = underflow;
    // Handle the reset logic
    if cr.reset.any() {
        d.read_address = bits(0);
        d.underflow = false;
        o.empty = true;
        o.almost_empty = true;
        o.read_address = bits(0);
        o.underflow = false;
    }
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_condition() {
        let q = Q::<4> {
            read_address: bits(0),
            underflow: false,
        };
        let i = I::<4> {
            write_address: bits(0),
            next: false,
        };
        let (o, d) = read_logic::<4>(ClockReset::init(), i, q);
        assert!(o.empty);
        assert!(o.almost_empty);
        assert!(!o.underflow);
        assert_eq!(o.read_address, bits(0));
        assert_eq!(d.read_address, bits(0));
        assert!(!d.underflow);
    }

    #[test]
    fn test_next_increments_read_address() {
        let q = Q::<4> {
            read_address: bits(0),
            underflow: false,
        };
        let i = I::<4> {
            write_address: bits(1),
            next: true,
        };
        let (o, d) = read_logic::<4>(ClockReset::init(), i, q);
        assert!(!o.empty);
        assert!(o.almost_empty);
        assert!(!o.underflow);
        assert_eq!(o.read_address, bits(1));
        assert_eq!(d.read_address, bits(1));
        assert!(!d.underflow);
    }

    #[test]
    fn test_empty_with_read_leads_to_underflow() {
        let q = Q::<4> {
            read_address: bits(0),
            underflow: false,
        };
        let i = I::<4> {
            write_address: bits(0),
            next: true,
        };
        let (o, d) = read_logic::<4>(ClockReset::init(), i, q);
        assert!(o.empty);
        assert!(o.almost_empty);
        assert!(o.underflow);
        assert_eq!(o.read_address, bits(0));
        assert_eq!(d.read_address, bits(0));
        assert!(d.underflow);
    }

    #[test]
    fn test_underflow_is_latching() {
        let q = Q::<4> {
            read_address: bits(0),
            underflow: true,
        };
        let i = I::<4> {
            write_address: bits(0),
            next: false,
        };
        let (o, d) = read_logic::<4>(ClockReset::init(), i, q);
        assert!(o.empty);
        assert!(o.almost_empty);
        assert!(o.underflow);
        assert_eq!(o.read_address, bits(0));
        assert_eq!(d.read_address, bits(0));
        assert!(d.underflow);
    }

    #[test]
    fn test_almost_empty_flag_is_clear_with_at_least_2_elements() {
        let q = Q::<4> {
            read_address: bits(0),
            underflow: false,
        };
        let i = I::<4> {
            write_address: bits(2),
            next: true,
        };
        let (o, d) = read_logic::<4>(ClockReset::init(), i, q);
        assert!(!o.empty);
        assert!(!o.almost_empty);
        assert!(!o.underflow);
        assert_eq!(o.read_address, bits(1));
        assert_eq!(d.read_address, bits(1));
        assert!(!d.underflow);
    }

    #[test]
    fn test_reset_condition() {
        let q = Q::<4> {
            read_address: bits(3),
            underflow: true,
        };
        let i = I::<4> {
            write_address: bits(3),
            next: false,
        };
        let cr = clock_reset(clock(false), reset(true));
        let (o, d) = read_logic::<4>(cr, i, q);
        assert!(o.empty);
        assert!(o.almost_empty);
        assert!(!o.underflow);
        assert_eq!(o.read_address, bits(0));
        assert_eq!(d.read_address, bits(0));
        assert!(!d.underflow);
    }
}
