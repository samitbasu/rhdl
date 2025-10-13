// Simple LED blinker for an XEM7010....

// The blinker itself is a simple synchronous counter
// with a bit selecting output.

use camino::Utf8PathBuf;
use rhdl::prelude::*;
use rhdl_toolchains::vivado::tcl::{GenerateBitstream, UpdateCompileOrder};

mod blinker {
    use super::*;

    #[derive(Clone, Synchronous, SynchronousDQ, Default)]
    pub struct U {
        // We need a 32 bit counter.
        counter: rhdl_fpga::core::counter::Counter<32>,
    }

    impl SynchronousIO for U {
        type I = ();
        type O = b8; // Needed to drive all 8 LEDs
        type Kernel = blinker;
    }

    #[kernel]
    pub fn blinker(_cr: ClockReset, _i: (), q: Q) -> (b8, D) {
        let mut d = D::dont_care();
        // The counter is always enabled.
        d.counter = true;
        let output_bit = (q.counter >> 28) & 1 != 0;
        let o = if output_bit { bits(0xaa) } else { bits(0x55) };
        (o, d)
    }
}

#[test]
fn test_blinker_fixture() -> miette::Result<()> {
    type T = Adapter<blinker::U, Red>;
    let blinker: T = Adapter::new(blinker::U::default());
    //    let inp: <T as CircuitIO>::I;
    //inp.clock_reset.val().clock
    let mut fixture = Fixture::new("top", blinker);
    fixture.add_driver(rhdl_bsp::ok::drivers::xem7010::sys_clock::sys_clock(
        &path!(.clock_reset.val().clock),
    )?);
    fixture.constant_input(reset(false), &path!(.clock_reset.val().reset))?;
    fixture.add_driver(rhdl_bsp::ok::drivers::xem7010::leds::leds(&path!(.val()
    ))?);
    let root = env!("CARGO_TARGET_TMPDIR");
    let path = Utf8PathBuf::from(root);
    let path = path.join("ok").join("xem7010").join("blinker");
    let builder = rhdl_toolchains::vivado::builder::Builder::new(
        path.as_str(),
        "blinker",
        "xc7a50tfgg484-1",
    )?;
    let builder = builder.add_fixture(fixture)?;
    let builder = builder.step(UpdateCompileOrder).step(GenerateBitstream {
        compressed_bitstream: true,
        bit_file: path.join("blinker.bit"),
    });
    builder.build().unwrap();
    Ok(())
}
