// Create an IBUFDS driver

use rhdl::prelude::*;
use serde::Serialize;

use crate::constraints::{IOStandard, Location};

#[derive(Clone, Debug, Serialize)]
pub struct Options {
    pub diff_term: bool,
    pub ibuf_low_pwr: bool,
    pub io_standard: Option<IOStandard>,
    pub pos_pin: Location,
    pub neg_pin: Location,
}

#[derive(Serialize)]
struct Context {
    name: String,
    options: Options,
    output: MountPoint,
}

static HDL: &str = r#"
IBUFDS #(
   .DIFF_TERM("{options.diff_term}"),       // Differential Termination
   .IBUF_LOW_PWR("{options.ibuf_low_pwr}"),     // Low power="TRUE", Highest performance="FALSE"
   .IOSTANDARD("{options.io_standard}")     // Specify the input I/O standard
) ibufds_{name} (
   .O({output}),  // Buffer output
   .I({name}_p),  // Diff_p buffer input (connect directly to top-level port)
   .IB({name}_n) // Diff_n buffer input (connect directly to top-level port)
);
"#;

static XDC: &str = r#"
# IBUFDS {name} ##########################################################
set_property IOSTANDARD {options.io_standard} [get_ports \{ {name}_p }]
set_property PACKAGE_PIN {options.pos_pin} [get_ports \{ {name}_p }]
set_property IOSTANDARD {options.io_standard} [get_ports \{ {name}_n }]
set_property PACKAGE_PIN {options.neg_pin} [get_ports \{ {name}_n }]
"#;

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
    let context = Context {
        name: name.into(),
        options: options.clone(),
        output,
    };
    driver.render_hdl(HDL, &context)?;
    driver.render_constraints(XDC, &context)?;
    Ok(driver)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::{bga_pin, utils::tt_render};

    use super::*;

    #[test]
    fn test_context_serialized() {
        let context = Context {
            name: "sysclk".into(),
            output: MountPoint::Output(12..13),
            options: Options {
                diff_term: false,
                ibuf_low_pwr: true,
                io_standard: Some(IOStandard::LowVoltageDifferentialSignal_2v5),
                pos_pin: bga_pin!(K, 4),
                neg_pin: bga_pin!(J, 4),
            },
        };
        let res = serde_json::to_string_pretty(&context).unwrap();
        let expect = expect![[r#"
            {
              "name": "sysclk",
              "options": {
                "diff_term": false,
                "ibuf_low_pwr": true,
                "io_standard": "LVDS_25",
                "pos_pin": "K4",
                "neg_pin": "J4"
              },
              "output": "inner_output[12]"
            }"#]];
        expect.assert_eq(&res);
        let tt = tt_render(HDL, &context).unwrap();
        let expect = expect![[r#"

            IBUFDS #(
               .DIFF_TERM("false"),       // Differential Termination
               .IBUF_LOW_PWR("true"),     // Low power="TRUE", Highest performance="FALSE"
               .IOSTANDARD("LVDS_25")     // Specify the input I/O standard
            ) ibufds_sysclk (
               .O(inner_output[12]),  // Buffer output
               .I(sysclk_p),  // Diff_p buffer input (connect directly to top-level port)
               .IB(sysclk_n) // Diff_n buffer input (connect directly to top-level port)
            );
        "#]];
        expect.assert_eq(&tt);
        let tt = tt_render(XDC, &context).unwrap();
        let expect = expect![[r#"

            # IBUFDS sysclk ##########################################################
            set_property IOSTANDARD LVDS_25 [get_ports { sysclk_p }]
            set_property PACKAGE_PIN K4 [get_ports { sysclk_p }]
            set_property IOSTANDARD LVDS_25 [get_ports { sysclk_n }]
            set_property PACKAGE_PIN J4 [get_ports { sysclk_n }]
        "#]];
        expect.assert_eq(&tt);
    }
}
