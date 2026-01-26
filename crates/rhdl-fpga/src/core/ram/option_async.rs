//! An asynchronous RAM with an `Option<T` interface
//!
//! This version of the asynchornous BRAM replaces the
//! write interface with an `Option` driven interface
//! to make it more idiomatic to use in RHDL.  The schematic
//! symbol looks like this
//!
#![doc = badascii_doc::badascii_formal!(r"
           +---+OptionAsyncRAM+--------+     
      B<N> |                           | T   
     +---->| read.addr          output +---->
      clk  |                           |     
     +---->| read.clock    ^  R domain |     
           |              +++          |     
?<(B<N>,T)>|               v  W domain |     
     +---->| write.data                |     
      clk  |                           |     
     +---->| write.clock               |     
           |                           |     
           +---------------------------+     
")]
//! Internally, the circuitry is composed of a combinatorial
//! unpacker.
//!
#![doc = badascii_doc::badascii!("
          +-+OptionAsyncBRAM+---------------------------------+     
          |                                                   |     
          |                            +--+AsyncRAM+----+     |     
 B<N>     |                            |                | T   |     
+---------+--------------------------->| read.addr  out +-----+---->
  clk     |                            |                |     |     
+---------+--------------------------->| read.clock     |     |     
          |                            |                |     |     
          |                            |                |     |     
          |                  .0   B<N> |                |     |     
?(B<N>,T) |  ++Unpack+-+   +---------->| write.addr     |     |     
+---------+->|         |   | .1    T   |                |     |     
          |  | (B<N>,T)+---+---------->| write.data     |     |     
          |  |         |          bool |                |     |     
          |  |    valid+-------------->| write.enable   |     |     
          |  +---------+               |                |     |     
 clk      |                            |                |     |     
+---------+--------------------------->| write.clock    |     |     
          |                            +----------------+     |     
          |                                                   |     
          +---------------------------------------------------+     
")]
//!
//!# Example
//!
//! Here is the simple example that demonstrates how to use
//! the option interface.
//!
//!```
#![doc = include_str!("../../../examples/option_async_bram.rs")]
//!```
//!
//!With a resulting trace file here.
#![doc = include_str!("../../../doc/option_async_bram.md")]
use rhdl::prelude::*;

#[derive(PartialEq, Debug, Clone, Default, Circuit, CircuitDQ)]
/// The unit to include for an option interface to the asynchronous
/// BRAM.  
///
/// The `T` parameter indicates the type of element stored in the
/// BRAM.  It must implement [Digital].
/// The `N` parameters indicates the number of address bits. Thus,
/// the BRAM will hold `2^N` elements.
/// The `W` domain is the clock domain where data is written.
/// The `R` domain is the clock domain where the reads run.
pub struct OptionAsyncBram<T: Digital, W: Domain, R: Domain, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    inner: super::asynchronous::AsyncBRAM<T, W, R, N>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> OptionAsyncBram<T, W, R, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// Create a new [OptionAsyncBRAM] with the provided initial contents.
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        Self {
            inner: super::asynchronous::AsyncBRAM::new(initial),
        }
    }
}

type ReadI<const N: usize> = super::asynchronous::ReadI<N>;

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// The write interface for the [OptionAsyncBRAM].
pub struct WriteI<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The clock signal for the write.
    pub clock: Clock,
    /// The data command for writing
    pub data: Option<(Bits<N>, T)>,
}

#[derive(PartialEq, Debug, Digital, Copy, Timed, Clone)]
/// The input struct for the [OptionAsyncBRAM]
pub struct In<T: Digital, W: Domain, R: Domain, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The write instruction
    pub write: Signal<WriteI<T, N>, W>,
    /// The read instruction
    pub read: Signal<ReadI<N>, R>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> CircuitIO for OptionAsyncBram<T, W, R, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T, W, R, N>;
    type O = Signal<T, R>;
    type Kernel = ram_kernel<T, W, R, N>;
}

#[kernel(allow_weak_partial)]
/// Kernel function for [OptionAsyncBRAM]
pub fn ram_kernel<T: Digital, W: Domain, R: Domain, const N: usize>(
    i: In<T, W, R, N>,
    q: OptionAsyncBramQ<T, W, R, N>,
) -> (Signal<T, R>, OptionAsyncBramD<T, W, R, N>)
where
    rhdl::bits::W<N>: BitWidth,
{
    // We need a struct for the write inputs to the RAM
    let mut w = super::asynchronous::WriteI::<T, N>::dont_care();
    // These are mapped from our input signals
    let i_val = i.write.val();
    w.clock = i_val.clock;
    if let Some((addr, data)) = i_val.data {
        w.data = data;
        w.enable = true;
        w.addr = addr;
    } else {
        w.data = T::dont_care();
        w.enable = false;
        w.addr = bits(0);
    }
    let mut d = OptionAsyncBramD::<T, W, R, N>::dont_care();
    d.inner.write = signal(w);
    d.inner.read = i.read;
    let o = q.inner;
    (o, d)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use expect_test::{expect, expect_file};

    use super::*;

    fn get_scan_out_stream<const N: usize>(
        read_clock: u64,
        count: usize,
    ) -> impl Iterator<Item = TimedSample<ReadI<N>>> + Clone
    where
        rhdl::bits::W<N>: BitWidth,
    {
        let scan_addr = (0..(1 << N)).map(bits::<N>).cycle().take(count);
        let stream_read = scan_addr.without_reset().clock_pos_edge(read_clock);
        stream_read.map(|t| {
            t.map(|(cr, val)| ReadI {
                addr: val,
                clock: cr.clock,
            })
        })
    }

    fn get_write_stream<T: Digital, const N: usize>(
        write_clock: u64,
        write_data: impl Iterator<Item = Option<(Bits<N>, T)>> + Clone,
    ) -> impl Iterator<Item = TimedSample<WriteI<T, N>>> + Clone
    where
        rhdl::bits::W<N>: BitWidth,
    {
        let stream_write = write_data.without_reset().clock_pos_edge(write_clock);
        stream_write.map(|t| {
            t.map(|(cr, val)| WriteI {
                data: val,
                clock: cr.clock,
            })
        })
    }

    #[test]
    fn test_ram_netlist() -> miette::Result<()> {
        let uut = OptionAsyncBram::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let desc = uut.descriptor("uut".into())?;
        let netlist = desc.netlist()?;
        let hdl = netlist.as_vlog("uut")?;
        let expect = expect_file!["ram_fg.expect"];
        expect.assert_eq(&hdl.modules.to_string());
        Ok(())
    }

    #[test]
    fn test_ram_as_verilog() -> miette::Result<()> {
        let uut = OptionAsyncBram::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let stream_read = get_scan_out_stream(100, 34);
        // The write interface will be dormant
        let stream_write = get_write_stream(70, std::iter::repeat_n(None, 50));
        // Stitch the two streams together
        let stream = stream_read.merge_map(stream_write, |r, w| In {
            read: signal(r),
            write: signal(w),
        });
        let test_bench = uut.run(stream).collect::<TestBench<_, _>>();
        let test_mod = test_bench.rtl(&uut, &TestBenchOptions::default().skip(10))?;
        test_mod.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ram_write_behavior() -> miette::Result<()> {
        let uut = OptionAsyncBram::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits(0))),
        );
        let writes = vec![
            Some((bits(0), bits(142))),
            Some((bits(5), bits(89))),
            Some((bits(2), bits(100))),
            None,
            Some((bits(15), bits(23))),
        ];
        let stream_read = get_scan_out_stream(100, 32);
        let stream_write = get_write_stream(70, writes.into_iter().chain(std::iter::repeat(None)));
        let stream = stream_read.merge_map(stream_write, |r, w| In {
            read: signal(r),
            write: signal(w),
        });
        let expected = vec![142, 0, 100, 0, 0, 89, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23]
            .into_iter()
            .map(|x| signal(bits(x)));
        let vcd = uut.run(stream.clone()).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ram")
            .join("option_async");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["0cbe54fb4eaae5186f03c00081325213ea39e02e23805a3d44fb24e726e26579"];
        let digest = vcd.dump_to_file(root.join("ram_write.vcd")).unwrap();
        expect.assert_eq(&digest);
        let output = uut
            .run(stream)
            .glitch_check(|x| (x.input.read.val().clock, x.output.val()))
            .sample_at_neg_edge(|x| x.input.read.val().clock)
            .skip(16)
            .map(|x| x.output);
        let expected = expected.collect::<Vec<_>>();
        let output = output.collect::<Vec<_>>();
        assert_eq!(expected, output);
        Ok(())
    }

    #[test]
    fn test_ram_read_only_behavior() -> miette::Result<()> {
        // Let's start with a simple test where the RAM is pre-initialized,
        // and we just want to read it.
        let uut = OptionAsyncBram::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let stream_read = get_scan_out_stream(100, 32);
        // The write interface will be dormant
        let stream_write = get_write_stream(70, std::iter::repeat_n(None, 50));
        // Stitch the two streams together
        let stream = merge_map(stream_read, stream_write, |r, w| In {
            read: signal(r),
            write: signal(w),
        });
        let values = (0..16).map(|x| bits(15 - x)).cycle().take(32);
        let samples = uut
            .run(stream)
            .sample_at_neg_edge(|i| i.input.read.val().clock);
        let output = samples.map(|x| x.output.val());
        assert!(values.eq(output));
        Ok(())
    }
}
