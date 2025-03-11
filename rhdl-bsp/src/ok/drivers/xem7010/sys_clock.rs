use crate::bga_pin;
use crate::constraints::IOStandard;
use crate::drivers::xilinx::ibufds;
use crate::error::BspError;
use rhdl::{prelude::*, rtt::TraceType};

// Create a driver that provides the sys clock (200 MHz)
// You must connect it to an input that expects a Signal<Clock, D> input
pub fn sys_clock<T: CircuitIO>(path: &Path) -> Result<Driver<T>, BspError> {
    let trace_type = <T::I as Digital>::static_trace_type();
    let target_trace = sub_trace_type(trace_type, path)?;
    if target_trace != TraceType::Clock {
        return Err(BspError::NotAClockInput(path.clone()));
    }
    let (bits, sub) = bit_range(<T::I as Timed>::static_kind(), path)?;
    if bits.len() != 1 || sub.is_signal() {
        return Err(BspError::NotAClockInput(path.clone()));
    }
    let mut driver = ibufds::build::<T>(
        "sysclk",
        path,
        &ibufds::Options {
            diff_term: false.into(),
            ibuf_low_pwr: true.into(),
            io_standard: Some(IOStandard::LowVoltageDifferentialSignal_2v5),
            pos_pin: bga_pin!(K, 4),
            neg_pin: bga_pin!(J, 4),
        },
    )?;
    driver.constraints += "create_clock -period 5 [get_ports sysclk_p]\n";
    Ok(driver)
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;
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
        let expect = expect_file!["sys_clock.expect"];
        expect.assert_debug_eq(&clock_driver);
    }
}
