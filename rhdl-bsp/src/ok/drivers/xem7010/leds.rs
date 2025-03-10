use crate::bga_pin;
use crate::drivers::xilinx::open_collector::Options;
use crate::error::BspError;
use rhdl::prelude::*;

// Create a driver for the LEDs.  These are open-collector type outputs.
pub fn leds<T: CircuitIO>(path: &Path) -> Result<Driver, BspError> {
    let (bits, _sub) = bit_range(<T::O as Timed>::static_kind(), path)?;
    if bits.len() != 8 {
        return Err(BspError::SignalWidthMismatch {
            expected: 8,
            actual: bits.len(),
        });
    }
    let options = Options {
        io_standard: crate::constraints::IOStandard::LowVoltageCMOS_3v3,
        pins: vec![
            bga_pin!(N, 13),
            bga_pin!(N, 14),
            bga_pin!(P, 15),
            bga_pin!(P, 16),
            bga_pin!(N, 17),
            bga_pin!(P, 17),
            bga_pin!(R, 16),
            bga_pin!(R, 17),
        ],
    };
    crate::drivers::xilinx::open_collector::build::<T>("led", path, &options)
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;
    use rhdl::prelude::*;

    #[test]
    fn test_led_driver() {
        #[derive(PartialEq, Digital, Timed)]
        struct O {
            leds: Signal<b8, Red>,
        }

        #[derive(Clone)]
        struct U;

        impl CircuitDQ for U {
            type D = ();
            type Q = ();
        }

        impl CircuitIO for U {
            type I = ();
            type O = O;
            type Kernel = NoKernel2<(), (), (O, ())>;
        }

        let led_driver = super::leds::<U>(&path!(.leds.val())).unwrap();
        let expect = expect_file!["led.expect"];
        expect.assert_debug_eq(&led_driver);
    }
}
