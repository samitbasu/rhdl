use camino::Utf8PathBuf;

pub struct CreateProject {
    pub path: Utf8PathBuf,
    pub part: String,
    pub name: String,
    pub force: bool,
}

// For now, I'm not going to worry about supporting different versions of Vivado.

impl std::fmt::Display for CreateProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "create_project {} {} -part {} {}",
            self.name,
            self.path,
            self.part,
            if self.force { "-force" } else { "" }
        )
    }
}

pub struct CloseProject;

impl std::fmt::Display for CloseProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "close_project")
    }
}

pub struct CreateBlockDesign {
    pub name: String,
}

impl std::fmt::Display for CreateBlockDesign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "create_bd_design {}", self.name)
    }
}

pub enum FileType {
    Source,
    Constraint,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Source => write!(f, "-fileset sources_1"),
            FileType::Constraint => write!(f, "-fileset constrs_1"),
        }
    }
}

pub struct AddFiles {
    pub kind: FileType,
    pub paths: Vec<Utf8PathBuf>,
}

impl std::fmt::Display for AddFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "add_files {} {{", self.kind)?;
        for path in &self.paths {
            write!(f, "{} ", path)?;
        }
        write!(f, "}}")
    }
}

pub struct Prop {
    pub name: String,
    pub value: String,
}

impl Prop {
    pub fn new<S: Into<String>, T: Into<String>>(name: S, value: T) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone)]
pub struct CellName(pub String);

impl CellName {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self(name.into())
    }
}

impl std::fmt::Display for CellName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct PortName(pub String);

impl PortName {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self(name.into())
    }
}

impl std::fmt::Display for PortName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for PortName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

pub struct InterfaceName(pub String);

impl std::fmt::Display for InterfaceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for InterfaceName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

pub struct Connection<T> {
    pub local_port: T,
    pub remote_node: CellName,
    pub remote_port: T,
}

pub struct Cell {
    pub id: String,
    pub name: CellName,
    pub props: Vec<Prop>,
    pub pin_connections: Vec<Connection<PortName>>,
    pub interface_connections: Vec<Connection<InterfaceName>>,
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "set cell [create_bd_cell -type ip -vlnv {id} {name}]",
            id = self.id,
            name = self.name
        )?;
        for prop in &self.props {
            writeln!(
                f,
                "set_property  CONFIG.{name} {value} $cell",
                name = prop.name,
                value = prop.value
            )?;
        }
        for connection in &self.pin_connections {
            writeln!(f, "connect_bd_net [get_bd_pins {name}/{local_port}] [get_bd_pins {remote_node}/{remote_port}]",
                name = self.name,
                local_port = connection.local_port,
                remote_node = connection.remote_node,
                remote_port = connection.remote_port
            )?;
        }
        for connection in &self.interface_connections {
            writeln!(f, "connect_bd_intf_net [get_bd_intf_pins {name}/{local_port}] [get_bd_intf_pins {remote_node}/{remote_port}]",
                name = self.name,
                local_port = connection.local_port,
                remote_node = connection.remote_node,
                remote_port = connection.remote_port
            )?;
        }
        Ok(())
    }
}

pub struct ConfigEntry {
    pub name: String,
    pub value: String,
}

impl ConfigEntry {
    pub fn new<S: Into<String>, T: Into<String>>(name: S, value: T) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl From<(&str, &str)> for ConfigEntry {
    fn from((name, value): (&str, &str)) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

impl std::fmt::Display for ConfigEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.value)
    }
}

pub enum Reference {
    Cell(CellName),
    Interface(CellName, InterfaceName),
}

impl std::fmt::Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reference::Cell(cell) => write!(f, "[get_bd_cells {}]", cell),
            Reference::Interface(cell, interface) => {
                write!(f, "[get_bd_intf_pins {}/{}]", cell, interface)
            }
        }
    }
}

pub struct ApplyBlockDesignAutomation {
    pub rule: String,
    pub config: Vec<ConfigEntry>,
    pub target_cell: Reference,
}

impl std::fmt::Display for ApplyBlockDesignAutomation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "apply_bd_automation -rule {} -config {{", self.rule)?;
        for entry in &self.config {
            writeln!(f, "{entry}")?;
        }
        writeln!(f, "}} {}", self.target_cell)
    }
}

#[derive(Default, Copy, Clone)]
pub enum PortType {
    Clock,
    ClockEnable,
    Reset,
    Interrupt,
    #[default]
    Data,
}

impl std::fmt::Display for PortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortType::Clock => write!(f, "clk"),
            PortType::ClockEnable => write!(f, "ce"),
            PortType::Reset => write!(f, "rst"),
            PortType::Interrupt => write!(f, "intr"),
            PortType::Data => write!(f, "data"),
        }
    }
}

pub enum PortDirection {
    Input,
    Output,
    InOut,
}

impl std::fmt::Display for PortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortDirection::Input => write!(f, "I"),
            PortDirection::Output => write!(f, "O"),
            PortDirection::InOut => write!(f, "IO"),
        }
    }
}

pub struct CreateBlockDesignPort {
    pub name: PortName,
    pub direction: PortDirection,
    pub port_type: PortType,
    pub from: usize,
    pub to: usize,
}

impl std::fmt::Display for CreateBlockDesignPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "create_bd_port -dir {dir} -from {from} -to {to} -type {typ} {name}",
            dir = self.direction,
            from = self.from,
            to = self.to,
            typ = self.port_type,
            name = self.name
        )
    }
}

pub enum PortOrPin {
    Port(PortName),
    Pin(CellName, PortName),
}

impl std::fmt::Display for PortOrPin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortOrPin::Port(port) => write!(f, "[get_bd_ports {}]", port),
            PortOrPin::Pin(cell, port) => write!(f, "[get_bd_pins {}/{}]", cell, port),
        }
    }
}

pub struct ConnectBlockDesignNet {
    pub items: Vec<PortOrPin>,
}

impl std::fmt::Display for ConnectBlockDesignNet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "connect_bd_net")?;
        for item in &self.items {
            write!(f, " {}", item)?;
        }
        Ok(())
    }
}

pub struct GenerateAllTargets {
    pub path: Utf8PathBuf,
}

impl std::fmt::Display for GenerateAllTargets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "generate_target all [get_files {}]", self.path)
    }
}

pub struct MakeWrapper {
    pub path: Utf8PathBuf,
    pub is_top: bool,
}

impl std::fmt::Display for MakeWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "make_wrapper -files [get_files {}] {}",
            self.path,
            if self.is_top { "-top" } else { "" }
        )
    }
}

pub struct UpdateCompileOrder;

impl std::fmt::Display for UpdateCompileOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "update_compile_order -fileset sources_1")
    }
}

pub struct GenerateBitstream {
    pub compressed_bitstream: bool,
    pub bit_file: Utf8PathBuf,
}

impl std::fmt::Display for GenerateBitstream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "launch_runs impl_1 -to_step write_bitstream")?;
        writeln!(f, "wait_on_run impl_1")?;
        writeln!(f, "open_run [get_runs impl_1]")?;
        if self.compressed_bitstream {
            writeln!(
                f,
                "set_property BITSTREAM.GENERAL.COMPRESS TRUE [current_design]"
            )?;
        }
        writeln!(f, "write_bitstream -force -file {}", self.bit_file)?;

        Ok(())
    }
}

#[derive(Default)]
pub struct Script {
    pub commands: Vec<String>,
}

impl Script {
    pub fn add<T: std::fmt::Display>(&mut self, command: T) {
        self.commands.push(format!("{}", command));
    }
}
