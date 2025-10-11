use quote::{format_ident, quote};
use rhdl::prelude::*;

use crate::constraints::{IOStandard, Location};

#[derive(Clone, Debug)]
pub struct Options {
    pub io_standard: IOStandard,
    pub pins: Vec<Location>,
}

pub fn build<T: CircuitIO>(
    name: &str,
    path: &Path,
    options: &Options,
) -> Result<Driver<T>, RHDLError> {
    let (bits, _) = bit_range(<T::O as Digital>::static_kind(), path)?;
    let mut driver = Driver::default();
    driver.output_port(name, bits.len());
    let output = driver.read_from_inner_output(path)?;
    let drive_range: vlog::BitRange = (0..options.pins.len()).into();
    let name_ident = format_ident!("{name}");
    let drive_name = format_ident!("_drive_{name}");
    let pin_assignments = options.pins.iter().enumerate().map(|(index, location)| {
        let index = syn::Index::from(index);
        quote! {
            assign #name_ident[#index] = (#drive_name[#index] == 1'b1) ? (1'b0) : (1'bz);
        }
    });
    driver.hdl = parse_quote_miette! {
        wire [#drive_range] #drive_name;
        assign #drive_name = #output;
        #(#pin_assignments)*
    }?;
    driver.constraints = options
        .pins
        .iter()
        .enumerate()
        .map(|(index, location)| {
            let pin = location.to_string();
            format!(
                r#"
set_property IOSTANDARD {} [get_ports {{ {}[{}] }}]
set_property PACKAGE_PIN {} [get_ports {{ {}[{}] }}]
"#,
                options.io_standard, name, index, pin, name, index
            )
        })
        .collect();
    Ok(driver)
}

#[cfg(test)]
mod tests {
    use crate::bga_pin;

    use super::*;

    #[test]
    fn test_open_collector() {
        let options = Options {
            io_standard: IOStandard::LowVoltageCMOS_3v3,
            pins: vec![bga_pin!(A, 1), bga_pin!(B, 3)],
        };

        #[derive(Clone)]
        struct U;

        impl CircuitDQ for U {
            type D = ();
            type Q = ();
        }

        impl CircuitIO for U {
            type I = ();
            type O = (Signal<b2, Red>, Signal<b4, Blue>);
            type Kernel = NoKernel2<(), (), ((Signal<b2, Red>, Signal<b4, Blue>), ())>;
        }

        let driver = build::<U>("led", &path!(.0.val()), &options).unwrap();
        let hdl = expect_test::expect![[r#"
            wire [1:0] _drive_led;
            assign _drive_led = inner_output[1:0];
            assign led[0] = (_drive_led[0] == 1'b1) ? (1'b0) : (1'bz);
            assign led[1] = (_drive_led[1] == 1'b1) ? (1'b0) : (1'bz);
        "#]];
        let xdc = expect_test::expect![[r#"

            set_property IOSTANDARD LVCMOS33 [get_ports { led[0] }]
            set_property PACKAGE_PIN A1 [get_ports { led[0] }]

            set_property IOSTANDARD LVCMOS33 [get_ports { led[1] }]
            set_property PACKAGE_PIN B3 [get_ports { led[1] }]
        "#]];
        hdl.assert_eq(&driver.hdl.pretty());
        xdc.assert_eq(&driver.constraints);
    }
}
