// Create an IBUFDS driver

use quote::format_ident;
use rhdl::prelude::*;

use crate::{
    constraints::{IOStandard, Location},
    utils::BoolParameter,
};

#[derive(Clone, Debug)]
pub struct Options {
    pub diff_term: bool,
    pub ibuf_low_pwr: bool,
    pub io_standard: IOStandard,
    pub pos_pin: Location,
    pub neg_pin: Location,
}

pub fn build<T: CircuitIO>(
    name: &str,
    path: &Path,
    options: &Options,
) -> Result<Driver<T>, RHDLError> {
    let mut driver = Driver::default();
    // We have two ports
    driver.input_port(&format!("{name}_p"), 1);
    driver.input_port(&format!("{name}_n"), 1);
    let output = driver.write_to_inner_input(path)?;
    let diff_term = BoolParameter(options.diff_term);
    let ibuf_low_pwr = BoolParameter(options.ibuf_low_pwr);
    let io_standard = options.io_standard;
    let buf_p = format_ident!("{name}_p");
    let buf_n = format_ident!("{name}_n");
    let instance_name = format_ident!("ibufds_{name}");
    driver.hdl = parse_quote_miette! {
        IBUFDS #(
           .DIFF_TERM(#diff_term),       // Differential Termination
           .IBUF_LOW_PWR(#ibuf_low_pwr),     // Low power="TRUE", Highest performance="FALSE"
           .IOSTANDARD(#io_standard)     // Specify the input I/O standard
        ) #instance_name (
           .O(#output),  // Buffer output
           .I(#buf_p),  // Diff_p buffer input (connect directly to top-level port)
           .IB(#buf_n) // Diff_n buffer input (connect directly to top-level port)
        );
    }?;
    let pos_pin = &options.pos_pin;
    let neg_pin = &options.neg_pin;
    driver.constraints = format!(
        r#"
# IBUFDS {name} ##########################################################
set_property IOSTANDARD {io_standard} [get_ports {{ {name}_p }}]
set_property PACKAGE_PIN {pos_pin} [get_ports {{ {name}_p }}]
set_property IOSTANDARD {io_standard} [get_ports {{ {name}_n }}]
set_property PACKAGE_PIN {neg_pin} [get_ports {{ {name}_n }}]
"#
    );
    Ok(driver)
}

#[cfg(test)]
mod tests {
    use crate::bga_pin;

    use super::*;

    #[test]
    fn test_ibufds() -> miette::Result<()> {
        let options = Options {
            diff_term: true,
            ibuf_low_pwr: false,
            io_standard: IOStandard::LowVoltageCMOS_1v5,
            pos_pin: bga_pin!(A, 5),
            neg_pin: bga_pin!(A, 6),
        };

        #[derive(Clone)]
        struct U;

        impl CircuitDQ for U {
            type D = ();
            type Q = ();
        }

        impl CircuitIO for U {
            type I = (Signal<Clock, Red>,);
            type O = Signal<b2, Red>;
            type Kernel = NoCircuitKernel<Self::I, (), (Self::O, ())>;
        }
        let i = <U as CircuitIO>::I::dont_care();
        let driver = build::<U>("clk", &path!(i.0.val()), &options)?;
        let hdl = expect_test::expect![[r#"
            IBUFDS #(.DIFF_TERM("TRUE"), .IBUF_LOW_PWR("FALSE"), .IOSTANDARD("LVCMOS15")) ibufds_clk(.O(inner_input[0:0]), .I(clk_p), .IB(clk_n));
        "#]];
        let xdc = expect_test::expect![[r#"

            # IBUFDS clk ##########################################################
            set_property IOSTANDARD LVCMOS15 [get_ports { clk_p }]
            set_property PACKAGE_PIN A5 [get_ports { clk_p }]
            set_property IOSTANDARD LVCMOS15 [get_ports { clk_n }]
            set_property PACKAGE_PIN A6 [get_ports { clk_n }]
        "#]];
        hdl.assert_eq(&driver.hdl.pretty());
        xdc.assert_eq(&driver.constraints);
        Ok(())
    }
}
