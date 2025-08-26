use rhdl::prelude::*;
use serde::Serialize;

use crate::constraints::{IOStandard, Location};

#[derive(Clone, Debug, Serialize)]
pub struct Options {
    pub io_standard: IOStandard,
    pub pins: Vec<Location>,
}

#[derive(Serialize)]
struct Context {
    name: String,
    pins_msb: usize,
    options: Options,
    output: MountPoint,
}

static HDL: &str = r#"
wire [{pins_msb}:0] _drive_{name};
assign _drive_{name} = {output};
{{ for pin in options.pins -}}
assign {name}[{@index}] = (_drive_{name}[{@index}] == 1'b1) ? (1'b0) : (1'bz);
{{ endfor }}
"#;

static XDC: &str = r#"
{{ for pin in options.pins -}}
set_property IOSTANDARD {options.io_standard} [get_ports {name}[{@index}]]
set_property PACKAGE_PIN {pin} [get_ports {name}[{@index}]]
{{ endfor}}
"#;

pub fn build<T: CircuitIO>(
    name: &str,
    path: &Path,
    options: &Options,
) -> Result<Driver<T>, RHDLError> {
    let (bits, _) = bit_range(<T::O as Timed>::static_kind(), path)?;
    let mut driver = Driver::default();
    driver.output_port(name, bits.len());
    let output = driver.read_from_inner_output(path)?;
    let context = Context {
        name: name.into(),
        pins_msb: options.pins.len() - 1,
        options: options.clone(),
        output,
    };
    driver.render_hdl(HDL, &context)?;
    driver.render_constraints(XDC, &context)?;
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
        eprintln!("{driver:?}");
    }
}
