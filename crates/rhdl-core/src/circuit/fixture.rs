use super::{circuit_impl::Circuit, hdl_backend::maybe_decl_wire};
use crate::{
    BitX, CircuitIO, Digital, Kind, RHDLError, Timed,
    hdl::ast::{
        Direction, HDLKind, Module, Port, SignedWidth, Statement, component_instance, connection,
        id,
    },
    types::path::{Path, bit_range, leaf_paths},
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
    #[error("Wrong constant type provided.  Expected {required:?}, and got {provided:?}")]
    WrongConstantType { provided: Kind, required: Kind },
    #[error("Path {0:?} on input is not a clock input")]
    NotAClockInput(Path),
    #[error(
        "Mismatch in signal width on input: expected {expected} bits, but got {actual} with path {path:?}"
    )]
    SignalWidthMismatchInput {
        expected: usize,
        actual: usize,
        path: Path,
    },
    #[error(
        "Mismatch in signal width on output: expected {expected} bits, but got {actual} with path {path:?}"
    )]
    SignalWidthMismatchOutput {
        expected: usize,
        actual: usize,
        path: Path,
    },
    #[error("Path {0:?} on input is not a clock output")]
    NotAClockOutput(Path),
    #[error("BSP Error {0}")]
    Custom(anyhow::Error),
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

#[derive(Clone)]
pub struct Driver<T> {
    marker: std::marker::PhantomData<T>,
    mounts: Vec<MountPoint>,
    pub ports: Vec<DriverPort>,
    pub hdl: String,
    pub constraints: String,
}

impl<T> std::fmt::Debug for Driver<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Driver")
            .field("mounts", &self.mounts)
            .field("ports", &self.ports)
            .field("hdl", &self.hdl)
            .field("constraints", &self.constraints)
            .finish()
    }
}

impl<T> Default for Driver<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData,
            mounts: Default::default(),
            ports: Default::default(),
            hdl: Default::default(),
            constraints: Default::default(),
        }
    }
}

fn render(template: &'static str, context: impl Serialize) -> Result<String, RHDLError> {
    let mut tt = TinyTemplate::new();
    tt.add_template("template", template)
        .map_err(|err| RHDLError::ExportError(ExportError::TemplateError(err)))?;
    tt.render("template", &context)
        .map_err(|err| RHDLError::ExportError(ExportError::TemplateError(err)))
}

impl<T: CircuitIO> Driver<T> {
    pub fn input_port(&mut self, name: &str, width: usize) {
        self.ports.push(DriverPort::input(name, width))
    }
    pub fn output_port(&mut self, name: &str, width: usize) {
        self.ports.push(DriverPort::output(name, width))
    }
    pub fn inout_port(&mut self, name: &str, width: usize) {
        self.ports.push(DriverPort::inout(name, width))
    }
    pub fn write_to_inner_input(&mut self, path: &Path) -> Result<MountPoint, RHDLError> {
        let (bits, _) = bit_range(<T::I as Timed>::static_kind(), path)?;
        let mount = MountPoint::Input(bits);
        self.mounts.push(mount.clone());
        Ok(mount)
    }
    pub fn read_from_inner_output(&mut self, path: &Path) -> Result<MountPoint, RHDLError> {
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

pub fn passthrough_output_driver<T: Circuit>(
    name: &str,
    path: &Path,
) -> Result<Driver<T>, RHDLError> {
    let (bits, _) = bit_range(<T::O as Timed>::static_kind(), path)?;
    let mut driver = Driver::default();
    driver.output_port(name, bits.len());
    let output = driver.read_from_inner_output(path)?;
    driver.hdl = format!("assign {name} = {output};");
    Ok(driver)
}

pub fn passthrough_input_driver<T: Circuit>(
    name: &str,
    path: &Path,
) -> Result<Driver<T>, RHDLError> {
    let (bits, _) = bit_range(<T::I as Timed>::static_kind(), path)?;
    let mut driver = Driver::default();
    driver.input_port(name, bits.len());
    let input = driver.write_to_inner_input(path)?;
    driver.hdl = format!("assign {input} = {name};");
    Ok(driver)
}

pub fn constant_driver<T: Circuit, S: Digital>(
    val: S,
    path: &Path,
) -> Result<Driver<T>, RHDLError> {
    let (_bits, sub_kind) = bit_range(<T::I as Timed>::static_kind(), path)?;
    if S::static_kind() != sub_kind {
        return Err(RHDLError::ExportError(ExportError::WrongConstantType {
            provided: S::static_kind(),
            required: sub_kind,
        }));
    }
    let mut driver = Driver::<T>::default();
    let input = driver.write_to_inner_input(path)?;
    let val = val.bin();
    let val_as_literal = val
        .into_iter()
        .map(|x| match x {
            BitX::One => '1',
            BitX::Zero => '0',
            BitX::X => 'x',
        })
        .collect::<String>();
    driver.hdl = format!(
        "assign {input} = {len}'b{literal};",
        len = val_as_literal.len(),
        literal = val_as_literal
    );
    Ok(driver)
}

pub struct Fixture<T> {
    name: String,
    drivers: Vec<Driver<T>>,
    circuit: T,
}

fn build_coverage_error(kind: Kind, coverage: &[bool]) -> String {
    let paths = leaf_paths(&kind, Path::default());
    let mut details = String::new();
    for path in paths {
        let (bits, _) = bit_range(kind, &path).unwrap();
        let covered = coverage[bits].iter().all(|b| *b);
        if !covered {
            details.push_str(&format!("Path {path:?} is not covered\n"));
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
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn add_driver(&mut self, driver: Driver<T>) {
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
    pub fn constant_input<S: Digital>(&mut self, val: S, path: &Path) -> Result<(), RHDLError> {
        self.add_driver(constant_driver::<T, S>(val, path)?);
        Ok(())
    }
    pub fn module(&self) -> Result<Module, RHDLError> {
        let ports = self
            .drivers
            .iter()
            .flat_map(|t| t.ports.iter())
            .map(|x| x.as_module_port())
            .collect();
        // Declare the mount points for the circuit
        let i_kind = <<T as CircuitIO>::I as Timed>::static_kind();
        let inputs_len = i_kind.bits();
        let outputs_len = <<T as CircuitIO>::O as Timed>::static_kind().bits();
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
            name: self.name.clone(),
            description: format!("Fixture for {}", self.circuit.description()),
            ports,
            declarations,
            statements,
            submodules: vec![verilog],
            ..Default::default()
        })
    }
    pub fn constraints(&self) -> String {
        let xdc = self
            .drivers
            .iter()
            .map(|x| x.constraints.clone())
            .collect::<Vec<_>>();
        xdc.join("\n")
    }
}
