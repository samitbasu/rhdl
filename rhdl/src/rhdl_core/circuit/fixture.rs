use super::{circuit_impl::Circuit, hdl_backend::maybe_decl_wire};
use crate::{
    prelude::{bit_range, CircuitIO, Direction, HDLKind, Kind, Module, Path, RHDLError, Timed},
    rhdl_core::{
        hdl::ast::id,
        hdl::ast::{component_instance, connection, Port, SignedWidth, Statement},
        types::path::leaf_paths,
    },
};
use miette::Diagnostic;
use serde::Serialize;
use thiserror::Error;
use tinytemplate::TinyTemplate;

#[derive(Error, Debug, Diagnostic)]
pub enum ExportError {
    #[error("Multiple drivers to circuit input")]
    MultipleDrivers,
    #[error("Inputs are not covered in exported core:\n{0}")]
    InputsNotCovered(String),
    #[error("Templating Error {0}")]
    TemplateError(#[from] tinytemplate::error::Error),
}

#[derive(Clone, Debug)]
pub enum MountPoint {
    Input(std::ops::Range<usize>),
    Output(std::ops::Range<usize>),
}

impl std::fmt::Display for MountPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MountPoint::Input(range) => {
                if range.is_empty() {
                    Err(std::fmt::Error)
                } else if range.len() == 1 {
                    write!(f, "inner_input[{}]", range.start)
                } else {
                    write!(
                        f,
                        "inner_input[{}:{}]",
                        range.end.saturating_sub(1),
                        range.start
                    )
                }
            }
            MountPoint::Output(range) => {
                if range.is_empty() {
                    Err(std::fmt::Error)
                } else if range.len() == 1 {
                    write!(f, "inner_output[{}]", range.start)
                } else {
                    write!(
                        f,
                        "inner_output[{}:{}]",
                        range.end.saturating_sub(1),
                        range.start
                    )
                }
            }
        }
    }
}

impl Serialize for MountPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct DriverPort {
    pub name: String,
    pub direction: Direction,
    pub width: usize,
}

impl DriverPort {
    pub fn input(name: &str, width: usize) -> Self {
        Self {
            name: name.into(),
            direction: Direction::Input,
            width,
        }
    }
    pub fn output(name: &str, width: usize) -> Self {
        Self {
            name: name.into(),
            direction: Direction::Output,
            width,
        }
    }
    pub fn inout(name: &str, width: usize) -> Self {
        Self {
            name: name.into(),
            direction: Direction::Inout,
            width,
        }
    }
    fn as_module_port(&self) -> Port {
        Port {
            name: self.name.clone(),
            direction: self.direction,
            kind: HDLKind::Wire,
            width: SignedWidth::Unsigned(self.width),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Driver {
    mounts: Vec<MountPoint>,
    pub ports: Vec<DriverPort>,
    pub hdl: String,
    pub constraints: String,
}

fn render(template: &'static str, context: impl Serialize) -> Result<String, RHDLError> {
    let mut tt = TinyTemplate::new();
    tt.add_template("template", template)
        .map_err(|err| RHDLError::ExportError(ExportError::TemplateError(err)))?;
    tt.render("template", &context)
        .map_err(|err| RHDLError::ExportError(ExportError::TemplateError(err)))
}

impl Driver {
    pub fn input_port(&mut self, name: &str, width: usize) {
        self.ports.push(DriverPort::input(name, width))
    }
    pub fn output_port(&mut self, name: &str, width: usize) {
        self.ports.push(DriverPort::output(name, width))
    }
    pub fn inout_port(&mut self, name: &str, width: usize) {
        self.ports.push(DriverPort::inout(name, width))
    }
    pub fn write_to_inner_input<T: CircuitIO>(
        &mut self,
        path: &Path,
    ) -> Result<MountPoint, RHDLError> {
        let (bits, _) = bit_range(<T::I as Timed>::static_kind(), path)?;
        let mount = MountPoint::Input(bits);
        self.mounts.push(mount.clone());
        Ok(mount)
    }
    pub fn read_from_inner_output<T: CircuitIO>(
        &mut self,
        path: &Path,
    ) -> Result<MountPoint, RHDLError> {
        let (bits, _) = bit_range(<T::O as Timed>::static_kind(), path)?;
        let mount = MountPoint::Output(bits);
        self.mounts.push(mount.clone());
        Ok(mount)
    }
    pub fn render_hdl(
        &mut self,
        template: &'static str,
        context: impl Serialize,
    ) -> Result<(), RHDLError> {
        self.hdl = render(template, context)?;
        Ok(())
    }
    pub fn render_constraints(
        &mut self,
        template: &'static str,
        context: impl Serialize,
    ) -> Result<(), RHDLError> {
        self.constraints = render(template, context)?;
        Ok(())
    }
    pub fn set_hdl(&mut self, hdl: &str) {
        self.hdl = hdl.into();
    }
    pub fn set_constraints(&mut self, constraints: &str) {
        self.constraints = constraints.into();
    }
}

pub fn passthrough_output_driver<T: Circuit>(name: &str, path: &Path) -> Result<Driver, RHDLError> {
    let (bits, _) = bit_range(<T::O as Timed>::static_kind(), path)?;
    let mut driver = Driver::default();
    driver.output_port(name, bits.len());
    let output = driver.read_from_inner_output::<T>(path)?;
    driver.hdl = format!("assign {name} = {output};");
    Ok(driver)
}

pub fn passthrough_input_driver<T: Circuit>(name: &str, path: &Path) -> Result<Driver, RHDLError> {
    let (bits, _) = bit_range(<T::I as Timed>::static_kind(), path)?;
    let mut driver = Driver::default();
    driver.input_port(name, bits.len());
    let input = driver.write_to_inner_input::<T>(path)?;
    driver.hdl = format!("assign {input} = {name};");
    Ok(driver)
}

pub struct Fixture<T> {
    name: String,
    drivers: Vec<Driver>,
    circuit: T,
}

fn build_coverage_error(kind: Kind, coverage: &[bool]) -> String {
    let paths = leaf_paths(&kind, Path::default());
    let mut details = String::new();
    for path in paths {
        let (bits, _) = bit_range(kind, &path).unwrap();
        let covered = coverage[bits].iter().all(|b| *b);
        if !covered {
            details.push_str(&format!("Path {:?} is not covered\n", path));
        }
    }
    details
}

impl<T: Circuit> Fixture<T> {
    pub fn new(name: &str, t: T) -> Self {
        Self {
            name: name.into(),
            drivers: vec![],
            circuit: t,
        }
    }
    pub fn add_driver(&mut self, driver: Driver) {
        self.drivers.push(driver)
    }
    pub fn pass_through_input(&mut self, name: &str, path: &Path) -> Result<(), RHDLError> {
        self.add_driver(passthrough_input_driver::<T>(name, path)?);
        Ok(())
    }
    pub fn pass_through_output(&mut self, name: &str, path: &Path) -> Result<(), RHDLError> {
        self.add_driver(passthrough_output_driver::<T>(name, path)?);
        Ok(())
    }
    pub fn module(self) -> Result<Module, RHDLError> {
        let ports = self
            .drivers
            .iter()
            .flat_map(|t| t.ports.iter())
            .map(|x| x.as_module_port())
            .collect();
        // Declare the mount points for the circuit
        let i_kind = <T as CircuitIO>::I::static_kind();
        let inputs_len = i_kind.bits();
        let outputs_len = <T as CircuitIO>::O::static_kind().bits();
        let declarations = [
            maybe_decl_wire(inputs_len, "inner_input"),
            maybe_decl_wire(outputs_len, "inner_output"),
        ]
        .into_iter()
        .flatten()
        .collect();
        let mut i_cover = vec![false; inputs_len];
        self.drivers
            .iter()
            .flat_map(|x| x.mounts.iter())
            .flat_map(|m| match m {
                MountPoint::Input(range) => Some(range.clone()),
                _ => None,
            })
            .try_for_each(|range| {
                for bit in range {
                    if i_cover[bit] {
                        return Err::<(), RHDLError>(ExportError::MultipleDrivers.into());
                    }
                    i_cover[bit] = true;
                }
                Ok(())
            })?;
        if i_cover.iter().any(|b| !b) {
            let coverage = build_coverage_error(i_kind, &i_cover);
            return Err(ExportError::InputsNotCovered(coverage).into());
        }
        let mut statements = self
            .drivers
            .iter()
            .map(|x| Statement::Custom(x.hdl.clone()))
            .collect::<Vec<_>>();
        // Instantiate the thing
        let hdl = self.circuit.hdl("inner")?;
        let verilog = hdl.as_module();
        statements.push(component_instance(
            &verilog.name,
            "inner_inst",
            vec![
                connection("i", id("inner_input")),
                connection("o", id("inner_output")),
            ],
        ));
        Ok(Module {
            name: self.name,
            description: format!("Fixture for {}", self.circuit.description()),
            ports,
            declarations,
            statements,
            submodules: vec![verilog],
            ..Default::default()
        })
    }
}
