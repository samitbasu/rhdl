//! FIFO Read Core
//! The read side of a FIFO.  In this design (which is meant to be maximally
//! simple, but also as robust as possible), the read side of the FIFO stores
//! an internal read address and an underflow flag.  The write address (owned by
//! the write side of the FIFO) is provided to the read logic as an input.
//!
//! Critical assumption:
//! We assume that the write address received from the write side of the FIFO
//! is conservative, meaning that the real write location is at least as great
//! as the write address provided - i.e., that the writer has definitely put
//! data into the given address when we get it.  It may have written additional
//! data that we don't know about, but we can be sure that the data at the
//! write address is valid.
//!
//! Note that this design will waste a slot in the FIFO when the read and write
//! addresses are equal, as it cannot otherwise distinguish between a full and
//! empty FIFO.  So for N bits, this design can store 2^N-1 elements.
//!
//! Here is the schematic symbol.
//!
#![doc = badascii_doc::badascii_formal!("
                  +----+FIFOReadCore+-------------------+      
                  |                                     |      
             +--->| write_address                 empty +----> 
       To         |                                     |      
FIFOWriteCore     |                        almost_empty +----> 
             <----+ will_advance                        |      
                  |                           underflow +----> 
       To         |                                     |      
       BRAM  <----+ ram_read_address               next |<----+
                  |                                     |      
                  +-------------------------------------+      
")]
use crate::core::dff;
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
/// The FIFO read logic as a core
pub struct FIFOReadCore<N: BitWidth> {
    ram_read_address: dff::DFF<Bits<N>>,
    underflow: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital)]
/// The inputs to the [FIFOReadCore]
pub struct In<N: BitWidth> {
    /// The current write address
    pub write_address: Bits<N>,
    /// The next signal advances the read pointer
    pub next: bool,
}

#[derive(PartialEq, Debug, Digital)]
/// The outputs from the [FIFOReadCore]
pub struct Out<N: BitWidth> {
    /// The empty signal
    pub empty: bool,
    /// The almost empty signal
    pub almost_empty: bool,
    /// The underflow signal
    pub underflow: bool,
    /// The current RAM read address
    pub ram_read_address: Bits<N>,
    /// Will pulse each time an item is consumed
    pub will_advance: bool,
}

impl<N: BitWidth> SynchronousIO for FIFOReadCore<N> {
    type I = In<N>;
    type O = Out<N>;
    type Kernel = read_logic<N>;
}

#[kernel]
/// Kernel for the [FIFOReadCore]
pub fn read_logic<N: BitWidth>(cr: ClockReset, i: In<N>, q: Q<N>) -> (Out<N>, D<N>) {
    // Compute the empty flag
    let empty = i.write_address == q.ram_read_address;
    // Compute the almost empty flag
    let almost_empty = empty || (q.ram_read_address + 1) == i.write_address;
    // Check for an underflow condition
    // We will underflow if we try to read when the FIFO is empty
    // and the condition is latching.
    let underflow = q.underflow || (i.next && empty);
    // Decide if we will advance the read pointer
    let will_advance = !empty && i.next;
    // If we will read, advance the read address - this is done
    // combinatorially to ensure that by the next clock edge, the
    // next value is already on the bus.
    let read_address = q.ram_read_address + if will_advance { 1 } else { 0 };
    let mut d = D::<N>::dont_care();
    d.ram_read_address = read_address;
    d.underflow = underflow;
    let mut o = Out::<N>::dont_care();
    o.empty = empty;
    o.almost_empty = almost_empty;
    o.ram_read_address = read_address;
    o.underflow = underflow;
    o.will_advance = will_advance;
    // Handle the reset logic
    if cr.reset.any() {
        d.ram_read_address = bits(0);
        d.underflow = false;
        o.empty = true;
        o.almost_empty = true;
        o.ram_read_address = bits(0);
        o.underflow = false;
        o.will_advance = false;
    }
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_condition() {
        let q = Q::<U4> {
            ram_read_address: bits(0),
            underflow: false,
        };
        let i = In::<U4> {
            write_address: bits(0),
            next: false,
        };
        let (o, d) = read_logic::<U4>(ClockReset::dont_care(), i, q);
        assert!(o.empty);
        assert!(o.almost_empty);
        assert!(!o.underflow);
        assert_eq!(o.ram_read_address, bits(0));
        assert_eq!(d.ram_read_address, bits(0));
        assert!(!d.underflow);
    }

    #[test]
    fn test_next_increments_read_address() {
        let q = Q::<U4> {
            ram_read_address: bits(0),
            underflow: false,
        };
        let i = In::<U4> {
            write_address: bits(1),
            next: true,
        };
        let (o, d) = read_logic::<U4>(ClockReset::dont_care(), i, q);
        assert!(!o.empty);
        assert!(o.almost_empty);
        assert!(!o.underflow);
        assert_eq!(o.ram_read_address, bits(1));
        assert_eq!(d.ram_read_address, bits(1));
        assert!(!d.underflow);
    }

    #[test]
    fn test_empty_with_read_leads_to_underflow() {
        let q = Q::<U4> {
            ram_read_address: bits(0),
            underflow: false,
        };
        let i = In::<U4> {
            write_address: bits(0),
            next: true,
        };
        let (o, d) = read_logic::<U4>(ClockReset::dont_care(), i, q);
        assert!(o.empty);
        assert!(o.almost_empty);
        assert!(o.underflow);
        assert_eq!(o.ram_read_address, bits(0));
        assert_eq!(d.ram_read_address, bits(0));
        assert!(d.underflow);
    }

    #[test]
    fn test_underflow_is_latching() {
        let q = Q::<U4> {
            ram_read_address: bits(0),
            underflow: true,
        };
        let i = In::<U4> {
            write_address: bits(0),
            next: false,
        };
        let (o, d) = read_logic::<U4>(ClockReset::dont_care(), i, q);
        assert!(o.empty);
        assert!(o.almost_empty);
        assert!(o.underflow);
        assert_eq!(o.ram_read_address, bits(0));
        assert_eq!(d.ram_read_address, bits(0));
        assert!(d.underflow);
    }

    #[test]
    fn test_almost_empty_flag_is_clear_with_at_least_2_elements() {
        let q = Q::<U4> {
            ram_read_address: bits(0),
            underflow: false,
        };
        let i = In::<U4> {
            write_address: bits(2),
            next: true,
        };
        let (o, d) = read_logic::<U4>(ClockReset::dont_care(), i, q);
        assert!(!o.empty);
        assert!(!o.almost_empty);
        assert!(!o.underflow);
        assert_eq!(o.ram_read_address, bits(1));
        assert_eq!(d.ram_read_address, bits(1));
        assert!(!d.underflow);
    }

    #[test]
    fn test_reset_condition() {
        let q = Q::<U4> {
            ram_read_address: bits(3),
            underflow: true,
        };
        let i = In::<U4> {
            write_address: bits(3),
            next: false,
        };
        let cr = clock_reset(clock(false), reset(true));
        let (o, d) = read_logic::<U4>(cr, i, q);
        assert!(o.empty);
        assert!(o.almost_empty);
        assert!(!o.underflow);
        assert_eq!(o.ram_read_address, bits(0));
        assert_eq!(d.ram_read_address, bits(0));
        assert!(!d.underflow);
    }
}
