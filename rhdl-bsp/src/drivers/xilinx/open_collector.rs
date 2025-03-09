use rhdl::prelude::*;
use serde::Serialize;

use crate::drivers::{port, Direction, Driver};
use crate::utils::tt_render;
use crate::{
    constraints::{Location, MountPoint, SignalType},
    error::BspError,
};

#[derive(Clone, Debug, Serialize)]
pub struct Options {
    pub io_standard: SignalType,
    pub pins: Vec<Location>,
}

#[derive(Serialize)]
struct Context {
    name: String,
    options: Options,
    output: MountPoint,
}

static HDL: &str = r#"
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

pub fn build<T: CircuitIO>(name: &str, path: &Path, options: &Options) -> Result<Driver, BspError> {
    let (bits, _) = bit_range(<T::O as Timed>::static_kind(), path)?;
    let ports = vec![port(name, Direction::Output, bits.len())];
    let context = Context {
        name: name.into(),
        options: options.clone(),
        output: MountPoint::Output(bits),
    };
    Ok(Driver {
        ports,
        hdl: tt_render(HDL, &context)?,
        constraints: tt_render(XDC, &context)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::bga_pin;

    use super::*;

    #[test]
    fn test_open_collector() {
        let options = Options {
            io_standard: SignalType::LowVoltageCMOS_3v3,
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
        eprintln!("{:?}", driver);
    }
}
