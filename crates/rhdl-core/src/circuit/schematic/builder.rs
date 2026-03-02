use crate::{
    CircuitIO, ClockReset, Digital, Kind, RHDLError, SynchronousIO, TypedBits,
    ast::{SourceLocation, spanned_source::SpannedSourceSet},
    circuit::schematic::{
        self, CanonicalPath, Link, LinkKind, Port, PortId, Schematic, SchematicId, SchematicKind,
        canonicalize_path, error::SchematicICE, port,
    },
    common::{slot_vec::SlotKey, symtab::LiteralId},
    error::rhdl_error,
    rhif::{self, spec::Slot},
    types::path::{PathExt, sub_kind},
};

#[derive(Debug)]
pub enum QueryPortSet {
    Input { index: usize },
    Output,
    ChildInput { child_index: usize, index: usize },
    ChildOutput { child_index: usize },
}

pub struct SchematicBuilder {
    top: Schematic,
    id: u32,
    schematic_id: u32,
}

impl SchematicBuilder {
    pub fn circuit<C: CircuitIO>(name: &str) -> Result<Self, RHDLError> {
        let mut me = Self::new(SchematicKind::Circuit);
        me.with_circuit_io::<C>()?
            .with_type_name::<C>()
            .with_name(name);
        Ok(me)
    }
    pub fn synchronous<S: SynchronousIO>(name: &str) -> Result<Self, RHDLError> {
        let mut me = Self::new(SchematicKind::Synchronous);
        me.with_synchronous_io::<S>()?
            .with_type_name::<S>()
            .with_name(name);
        Ok(me)
    }
    pub fn kernel() -> Self {
        Self::new(SchematicKind::Kernel)
    }
    pub fn opcode(loc: SourceLocation) -> Self {
        let mut me = Self::new(SchematicKind::OpCode);
        me.top.location = Some(loc);
        me
    }
    pub fn literal(slot: Slot, val: &TypedBits) -> Self {
        let mut me = Self::new(SchematicKind::Literal);
        me.top.name = format!("{}: {:?}", slot, val);
        me
    }
    fn new(kind: SchematicKind) -> Self {
        Self {
            top: Schematic {
                id: SchematicId(0),
                name: String::default(),
                type_name: String::default(),
                filename: String::default(),
                kind,
                inputs: vec![],
                outputs: vec![],
                inner: vec![],
                links: vec![],
                sources: SpannedSourceSet::default().into(),
                location: None,
                debug_text: String::default(),
            },
            id: 0,
            schematic_id: 0,
        }
    }
    pub fn with_circuit_io<C: CircuitIO>(&mut self) -> Result<&mut Self, RHDLError> {
        let circuit_input = <C as CircuitIO>::I::static_kind();
        let circuit_output = <C as CircuitIO>::O::static_kind();
        Ok(self
            .add_input_ports(0, circuit_input, None)?
            .add_output_port(circuit_output, None)?
            .with_type_name::<C>())
    }
    pub fn with_synchronous_io<S: SynchronousIO>(&mut self) -> Result<&mut Self, RHDLError> {
        let clock_reset = ClockReset::static_kind();
        let sync_input = <S as SynchronousIO>::I::static_kind();
        let sync_output = <S as SynchronousIO>::O::static_kind();
        Ok(self
            .add_input_ports(0, clock_reset, None)?
            .add_input_ports(1, sync_input, None)?
            .add_output_port(sync_output, None)?
            .with_type_name::<S>())
    }
    pub fn with_name(&mut self, name: &str) -> &mut Self {
        self.top.name = name.to_string();
        self
    }
    pub fn with_type_name<C>(&mut self) -> &mut Self {
        self.top.type_name = std::any::type_name::<C>().to_string();
        self
    }
    pub fn with_debug_text(&mut self, text: &str) -> &mut Self {
        self.top.debug_text = text.to_string();
        self
    }
    pub fn with_filename(&mut self, filename: &str) -> &mut Self {
        self.top.filename = filename.to_string();
        self
    }
    pub fn top_mut(&mut self) -> &mut Schematic {
        &mut self.top
    }
    pub fn next_id(&mut self) -> PortId {
        let id = self.id;
        self.id += 1;
        PortId(id)
    }
    pub fn add_input_ports(
        &mut self,
        index: usize,
        kind: Kind,
        slot: Option<Slot>,
    ) -> Result<&mut Self, RHDLError> {
        self.top.inputs.resize(index + 1, vec![]);
        for path in kind.all_leafs() {
            let port_kind = sub_kind(kind, &path)?;
            if port_kind.is_empty() {
                continue;
            }
            let id = self.next_id();
            log::debug!(
                "Adding input port {path:?} of kind {port_kind:?} with id {id:?} and slot {slot:?}",
            );
            self.top.inputs[index].push(port(kind, &path, port_kind, id, slot)?);
        }
        Ok(self)
    }
    pub fn add_output_port(
        &mut self,
        kind: Kind,
        slot: Option<Slot>,
    ) -> Result<&mut Self, RHDLError> {
        for path in kind.all_leafs() {
            let port_kind = sub_kind(kind, &path)?;
            if port_kind.is_empty() {
                continue;
            }
            let id = self.next_id();
            self.top
                .outputs
                .push(port(kind, &path, port_kind, id, slot)?);
        }
        Ok(self)
    }
    pub fn allocate_input_port(&mut self, kind: Kind, slot: Option<Slot>) -> usize {
        let index = self.top.inputs.len();
        self.add_input_ports(index, kind, slot)
            .expect("Adding input ports should never fail");
        index
    }
    pub fn import(&mut self, mut schematic: Schematic) -> usize {
        // Get the next port number
        let offset = self.next_id();
        let max_port_id = schematic.max_port_id();
        let max_schematic_id = schematic.max_schematic_id();
        schematic.shift_ports(offset);
        schematic.shift_id((self.schematic_id + 1).into());
        self.id += max_port_id.0 + 1;
        self.schematic_id += max_schematic_id.0 + 1;
        self.top.inner.push(schematic);
        self.top.inner.len() - 1
    }
    pub fn build(self) -> Schematic {
        self.top
    }
    pub fn connect(&mut self, from: PortId, to: PortId) -> &mut Self {
        self.top.links.push(Link {
            from,
            to,
            kind: LinkKind::Strong,
        });
        self
    }
    fn query_port(&self, query: &QueryPortSet, path: &CanonicalPath) -> Port {
        match query {
            QueryPortSet::Input { index } => self.top.inputs[*index]
                .iter()
                .find(|p| p.path == *path)
                .unwrap()
                .clone(),
            QueryPortSet::Output => self
                .top
                .outputs
                .iter()
                .find(|p| p.path == *path)
                .unwrap()
                .clone(),
            QueryPortSet::ChildInput { child_index, index } => self.top.inner[*child_index].inputs
                [*index]
                .iter()
                .find(|p| p.path == *path)
                .unwrap()
                .clone(),
            QueryPortSet::ChildOutput { child_index } => self.top.inner[*child_index]
                .outputs
                .iter()
                .find(|p| p.path == *path)
                .unwrap()
                .clone(),
        }
    }
    pub fn select_and_link(
        &mut self,
        from: QueryPortSet,
        to: QueryPortSet,
        kind: Kind,
        from_path: &CanonicalPath,
        to_path: &CanonicalPath,
    ) -> Result<(), RHDLError> {
        log::debug!(
            "Linking from {:?} to {:?} of kind {:?} with paths {:?} -> {:?}",
            from,
            to,
            kind,
            from_path,
            to_path
        );
        if kind.is_empty() {
            return Ok(());
        }
        for path in kind.all_leafs() {
            let path = canonicalize_path(kind, &path)?;
            let from_port_path = from_path.join(&path);
            let to_port_path = to_path.join(&path);
            let from_port = self.query_port(&from, &from_port_path);
            let to_port = self.query_port(&to, &to_port_path);
            log::debug!(
                "Linking from port {:?} to port {:?} of kind {:?}",
                from_port,
                to_port,
                LinkKind::Strong
            );
            self.top.links.push(Link {
                from: from_port.id,
                to: to_port.id,
                kind: LinkKind::Strong,
            });
        }
        Ok(())
    }
    pub fn forward_input_to_child(
        &mut self,
        input: usize,
        child_index: usize,
        child_input: usize,
    ) -> &mut Self {
        for (input_port, child_input_port) in self.top.inputs[input]
            .iter()
            .zip(self.top.inner[child_index].inputs[child_input].iter())
        {
            self.top.links.push(Link {
                from: input_port.id,
                to: child_input_port.id,
                kind: LinkKind::Strong,
            });
        }
        self
    }
    pub fn forward_output_from_child(&mut self, child_index: usize) -> &mut Self {
        for (output_port, child_output_port) in self
            .top
            .outputs
            .iter()
            .zip(&self.top.inner[child_index].outputs)
        {
            self.top.links.push(Link {
                from: child_output_port.id,
                to: output_port.id,
                kind: LinkKind::Strong,
            });
        }
        self
    }
    pub fn copy_child_output_to_child_input(
        &mut self,
        from_child_index: usize,
        to_child_index: usize,
        to_child_input: usize,
    ) -> &mut Self {
        for (from_port, to_port) in self.top.inner[from_child_index]
            .outputs
            .iter()
            .zip(&self.top.inner[to_child_index].inputs[to_child_input])
        {
            self.top.links.push(Link {
                from: from_port.id,
                to: to_port.id,
                kind: LinkKind::Strong,
            });
        }
        self
    }
    pub fn splat_input_port_to_output(&mut self, from_index: usize) -> &mut Self {
        for port in &self.top.inputs[from_index] {
            for dest in &self.top.outputs {
                self.top.links.push(Link {
                    from: port.id,
                    to: dest.id,
                    kind: LinkKind::Weak,
                });
            }
        }
        self
    }
    pub fn assign_input_to_output(&mut self, from_index: usize) -> &mut Self {
        for (input_port, output_port) in self.top.inputs[from_index].iter().zip(&self.top.outputs) {
            self.top.links.push(Link {
                from: input_port.id,
                to: output_port.id,
                kind: LinkKind::Strong,
            });
        }
        self
    }
    pub fn assign_from_to_path(
        &mut self,
        from_index: usize,
        from_path: CanonicalPath,
        to_path: CanonicalPath,
    ) -> Result<&mut Self, RHDLError> {
        let from_port = self.top.inputs[from_index]
            .iter()
            .find(|p| p.path == from_path)
            .ok_or_else(|| {
                rhdl_error(SchematicICE::MissingPortInSchematic {
                    path: from_path.clone(),
                    avail: self.top.inputs[from_index].clone(),
                })
            })?;
        let to_port = self
            .top
            .outputs
            .iter()
            .find(|p| p.path == to_path)
            .ok_or_else(|| {
                rhdl_error(SchematicICE::MissingPortInSchematic {
                    path: to_path.clone(),
                    avail: self.top.outputs.clone(),
                })
            })?;
        self.top.links.push(Link {
            from: from_port.id,
            to: to_port.id,
            kind: LinkKind::Strong,
        });
        Ok(self)
    }

    pub(crate) fn next_child_index(&self) -> usize {
        self.top.inner.len()
    }
}
