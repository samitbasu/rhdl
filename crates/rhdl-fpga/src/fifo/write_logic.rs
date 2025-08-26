//! FIFO Write Core
//!
//! The write side of the FIFO.  In this design (which is meant to be maximally
//! simple, but also as robust as possible), the write side of the FIFO stores
//! an internal write address and an overflow flag.  The read address (owned by
//! the read side of the FIFO) is provided to the write logic as an input.
//!
//! Critical assumption:
//!  
//! We assume that the read address received from the
//! read side of the FIFO is conservative, meaning that
//! the real read location is at least as great
//! as the read address provided - i.e., that the reader may have already read
//! out the given memory location, but that the writer can safely write into the
//! FIFO provided it does not reach the given address
//!
//! Note that this design will waste a slot in the FIFO when the read and write
//! addresses are equal, as it cannot otherwise distinguish between a full and
//! empty FIFO.  So for N bits, this design can store 2^N-1 elements.
//!
//! Here is the schematic symbol.
//!
#![doc = badascii_formal!("
     +----+FIFOWriteCore+-------------+                    
     |                                |        From        
+--->|write_enable        read_address|<---+   FIFOReadCore
     |                                |                    
<----+full                            |                    
     |               ram_write_address+---->   To          
<----+almost_full                     |        BRAM        
     |                                |                    
<----+overflow           write_address+---->   To          
     |                                |        FIFOReadCore
     +--------------------------------+                    
")]
//!
//! The write core keeps track of the write pointer of the FIFO.

use crate::core::dff;
use badascii_doc::badascii_formal;
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
/// The FIFO write logic as a core
pub struct FIFOWriteCore<N: BitWidth> {
    write_address: dff::DFF<Bits<N>>,
    // We delay the write address by one clock before sending
    // it to the read side of the FIFO.  This is because it will
    // take one clock for the write to actually happen, and we
    // want to make sure the value is valid on the read side before
    // "counting" the write.
    write_address_delayed: dff::DFF<Bits<N>>,
    overflow: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital)]
/// The inputs to the [FIFOWriteCore]
pub struct In<N: BitWidth> {
    /// The current read address
    pub read_address: Bits<N>,
    /// The write signal that advances the write pointer
    pub write_enable: bool,
}

#[derive(PartialEq, Debug, Digital)]
/// The outputs from the [FIFOWriteCore]
pub struct Out<N: BitWidth> {
    /// The generated full signal
    pub full: bool,
    /// The generated almost full signal
    pub almost_full: bool,
    /// The overflow signal
    pub overflow: bool,
    /// The write address to send to the BRAM
    pub ram_write_address: Bits<N>,
    /// The write address to send to the read side (delayed)
    pub write_address: Bits<N>,
}

impl<N: BitWidth> SynchronousIO for FIFOWriteCore<N> {
    type I = In<N>;
    type O = Out<N>;
    type Kernel = write_logic<N>;
}

#[kernel]
/// Kernel for the [FIFOWriteCore]
pub fn write_logic<N: BitWidth>(cr: ClockReset, i: In<N>, q: Q<N>) -> (Out<N>, D<N>) {
    // Compute the full flag
    let full = (q.write_address + 1) == i.read_address;
    // Compute the almost full flag
    let almost_full = full || (q.write_address + 2) == i.read_address;
    // Check for an overflow condition
    // We will overflow if we try to write when the FIFO is full
    // and the condition is latching.
    let overflow = q.overflow || (i.write_enable && full);
    // Decide if we will write
    let will_write = !full && i.write_enable;
    // If we will write, advance the write address
    let write_address = q.write_address + if will_write { 1 } else { 0 };
    let mut d = D::<N>::dont_care();
    d.write_address_delayed = q.write_address;
    d.write_address = write_address;
    d.overflow = overflow;
    let mut o = Out::<N>::dont_care();
    o.full = full;
    o.almost_full = almost_full;
    // We output the current write address delayed by one clock, not the future one
    o.write_address = q.write_address_delayed;
    o.ram_write_address = q.write_address;
    o.overflow = overflow;
    // Handle the reset logic
    if cr.reset.any() {
        d.write_address = bits(0);
        d.overflow = false;
        o.full = false;
        // Note that this assumes the FIFO is at least 2 elements deep
        o.almost_full = false;
        o.write_address = bits(0);
        o.ram_write_address = bits(0);
        o.overflow = false;
    }
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_condition() {
        let cr = ClockReset::dont_care();
        let i = In::<U4> {
            read_address: bits(0b0000),
            write_enable: false,
        };
        let q = Q::<U4> {
            write_address: bits(0b1111),
            write_address_delayed: bits(0b1111),
            overflow: false,
        };
        let (o, d) = write_logic(cr, i, q);
        assert!(o.full);
        assert!(o.almost_full);
        assert!(!o.overflow);
        assert_eq!(o.write_address, bits(0b1111));
        assert_eq!(d.write_address, bits(0b1111));
        assert!(!d.overflow);
    }

    #[test]
    fn test_almost_full_condition() {
        let cr = ClockReset::dont_care();
        let i = In::<U4> {
            read_address: bits(0b0000),
            write_enable: false,
        };
        let q = Q::<U4> {
            write_address: bits(0b1110),
            write_address_delayed: bits(0b1110),
            overflow: false,
        };
        let (o, d) = write_logic(cr, i, q);
        assert!(!o.full);
        assert!(o.almost_full);
        assert!(!o.overflow);
        assert_eq!(o.write_address, bits(0b1110));
        assert_eq!(d.write_address, bits(0b1110));
        assert!(!d.overflow);
    }

    #[test]
    fn test_write_enable_increments_next_write_address() {
        let cr = ClockReset::dont_care();
        let i = In::<U4> {
            read_address: bits(0b0000),
            write_enable: true,
        };
        let q = Q::<U4> {
            write_address: bits(0b1100),
            write_address_delayed: bits(0b1100),
            overflow: false,
        };
        let (o, d) = write_logic(cr, i, q);
        assert!(!o.full);
        assert!(!o.almost_full);
        assert!(!o.overflow);
        assert_eq!(o.write_address, bits(0b1100));
        assert_eq!(d.write_address, bits(0b1101));
        assert!(!d.overflow);
    }

    #[test]
    fn test_full_with_write_enable_leads_to_overflow() {
        let cr = ClockReset::dont_care();
        let i = In::<U4> {
            read_address: bits(0b0000),
            write_enable: true,
        };
        let q = Q::<U4> {
            write_address: bits(0b1111),
            write_address_delayed: bits(0b1111),
            overflow: false,
        };
        let (o, d) = write_logic(cr, i, q);
        assert!(o.full);
        assert!(o.almost_full);
        assert!(o.overflow);
        assert_eq!(o.write_address, bits(0b1111));
        assert_eq!(d.write_address, bits(0b1111));
        assert!(d.overflow);
    }

    #[test]
    fn test_overflow_is_latching() {
        let cr = ClockReset::dont_care();
        let i = In::<U4> {
            read_address: bits(0b0000),
            write_enable: false,
        };
        let q = Q::<U4> {
            write_address: bits(0b1111),
            write_address_delayed: bits(0b1111),
            overflow: true,
        };
        let (o, d) = write_logic(cr, i, q);
        assert!(o.full);
        assert!(o.almost_full);
        assert!(o.overflow);
        assert_eq!(o.write_address, bits(0b1111));
        assert_eq!(d.write_address, bits(0b1111));
        assert!(d.overflow);
    }

    #[test]
    fn test_almost_full_flag_is_clear_with_at_least_2_spots() {
        let cr = ClockReset::dont_care();
        let i = In::<U4> {
            read_address: bits(0b0000),
            write_enable: false,
        };
        let q = Q::<U4> {
            write_address: bits(0b1100),
            write_address_delayed: bits(0b1100),
            overflow: false,
        };
        let (o, d) = write_logic(cr, i, q);
        assert!(!o.full);
        assert!(!o.almost_full);
        assert!(!o.overflow);
        assert_eq!(o.write_address, bits(0b1100));
        assert_eq!(d.write_address, bits(0b1100));
        assert!(!d.overflow);
    }

    #[test]
    fn test_reset_condition() {
        let cr = clock_reset(clock(false), reset(true));
        let i = In::<U4> {
            read_address: bits(0b0000),
            write_enable: false,
        };
        let q = Q::<U4> {
            write_address: bits(0b1111),
            write_address_delayed: bits(0b1111),
            overflow: true,
        };
        let (o, d) = write_logic(cr, i, q);
        assert!(!o.full);
        assert!(!o.almost_full);
        assert!(!o.overflow);
        assert_eq!(o.write_address, bits(0b0000));
        assert_eq!(d.write_address, bits(0b0000));
        assert!(!d.overflow);
    }
}
