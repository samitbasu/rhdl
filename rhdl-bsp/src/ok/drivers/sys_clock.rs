use crate::constraints::{BGARow, Location, SignalType};
use crate::drivers::xilinx::ibufds;
use crate::{drivers::Driver, error::BspError};
use rhdl::{prelude::*, rtt::TraceType};

// Create a driver that provides the sys clock (200 MHz)
// You must connect it to an input that expects a Signal<Clock, D> input
pub fn sys_clock<T: CircuitIO>(path: &Path) -> Result<Driver, BspError> {
    let trace_type = <T::I as Digital>::static_trace_type();
    let target_trace = sub_trace_type(trace_type, path)?;
    if target_trace != TraceType::Clock {
        return Err(BspError::NotAClockInput(path.clone()));
    }
    let (bits, sub) = bit_range(<T::I as Timed>::static_kind(), &path)?;
    if bits.len() != 1 || sub.is_signal() {
        return Err(BspError::NotAClockInput(path.clone()));
    }
    ibufds::build::<T>(
        "sysclk",
        path,
        &ibufds::Options {
            diff_term: false,
            ibuf_low_pwr: true,
            io_standard: Some(SignalType::LowVoltageDifferentialSignal_2v5),
            pos_pin: Location::BGABall {
                row: BGARow::K,
                col: 4,
            },
            neg_pin: Location::BGABall {
                row: BGARow::J,
                col: 4,
            },
        },
    )
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use rhdl::prelude::*;

    #[test]
    fn test_sys_clock() {
        #[derive(PartialEq, Digital, Timed)]
        struct I {
            clock: Signal<Clock, Red>,
            reset: Signal<Reset, Red>,
        }

        #[derive(Clone)]
        struct T;

        impl CircuitDQ for T {
            type D = ();
            type Q = ();
        }

        impl CircuitIO for T {
            type I = I;
            type O = ();
            type Kernel = NoKernel2<I, (), ((), ())>;
        }

        let clock_driver = super::sys_clock::<T>(&path!(.clock.val())).unwrap();
        let expect = expect![[r#"
            Driver {
                ports: [
                    Port {
                        name: "sysclk_p",
                        direction: Input,
                        width: 1,
                    },
                    Port {
                        name: "sysclk_n",
                        direction: Input,
                        width: 1,
                    },
                ],
                hdl: "\nIBUFDS #(\n   .DIFF_TERM(\"false\"),       // Differential Termination\n   .IBUF_LOW_PWR(\"true\"),     // Low power=\"TRUE\", Highest performance=\"FALSE\"\n   .IOSTANDARD(\"LVDS_25\")     // Specify the input I/O standard\n) ibufds_sysclk (\n   .O(inner_input[0]),  // Buffer output\n   .I(sysclk_p),  // Diff_p buffer input (connect directly to top-level port)\n   .IB(sysclk_n) // Diff_n buffer input (connect directly to top-level port)\n);\n",
                constraints: "\nset_property IOSTANDARD LVDS_25 [get_ports { sysclk_p }]\nset_property PACKAGE_PIN K4 [get_ports { sysclk_p }]\nset_property IOSTANDARD LVDS_25 [get_ports { sysclk_n }]\nset_property PACKAGE_PIN J4 [get_ports { sysclk_n }]\n",
            }
        "#]];
        expect.assert_debug_eq(&clock_driver);
    }
}
