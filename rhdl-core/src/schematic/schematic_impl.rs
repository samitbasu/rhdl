use std::collections::HashMap;

use crate::{
    ast::{ast_impl::FunctionId, source_location::SourceLocation, spanned_source::SpannedSource},
    types::path::Path,
    Kind,
};

use super::components::{BufferComponent, Component, ComponentKind};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PinIx(usize);

impl PinIx {
    pub fn offset(self, offset: usize) -> PinIx {
        PinIx(self.0 + offset)
    }
}

impl std::fmt::Debug for PinIx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "p{}", self.0)
    }
}

const NOPIN: PinIx = PinIx(!0);

impl Default for PinIx {
    fn default() -> Self {
        NOPIN
    }
}

const ORPHAN: ComponentIx = ComponentIx(!0);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentIx(usize);

impl ComponentIx {
    pub fn offset(self, offset: usize) -> ComponentIx {
        ComponentIx(self.0 + offset)
    }
}

impl From<ComponentIx> for usize {
    fn from(val: ComponentIx) -> Self {
        val.0
    }
}

impl std::fmt::Debug for ComponentIx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c{}", self.0)
    }
}

// Need some fixup mechanism for this so that
// pins can be created, then components, and then
// the pins can be updated to point to the correct
// component.

#[derive(Debug, Clone)]
pub struct Pin {
    pub kind: Kind,
    pub name: String,
    pub location: Option<SourceLocation>,
    pub parent: ComponentIx,
}

impl Pin {
    pub fn parent(&mut self, parent: ComponentIx) {
        self.parent = parent;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Wire {
    pub source: PinIx,
    pub dest: PinIx,
}

impl Wire {
    fn relocate(self, map: &HashMap<PinIx, PinIx>) -> Wire {
        Wire {
            source: *map.get(&self.source).unwrap_or(&self.source),
            dest: *map.get(&self.dest).unwrap_or(&self.dest),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schematic {
    pub pins: Vec<Pin>,
    pub components: Vec<Component>,
    pub wires: Vec<Wire>,
    pub inputs: Vec<PinIx>,
    pub output: PinIx,
    pub source: HashMap<FunctionId, SpannedSource>,
}

impl Schematic {
    pub fn make_pin(
        &mut self,
        kind: Kind,
        name: String,
        location: Option<SourceLocation>,
    ) -> PinIx {
        let ix = PinIx(self.pins.len());
        self.pins.push(Pin {
            kind,
            name,
            parent: ORPHAN,
            location,
        });
        ix
    }
    pub fn make_buffer(&mut self, kind: Kind, location: Option<SourceLocation>) -> (PinIx, PinIx) {
        let input = self.make_pin(kind.clone(), "in".into(), location);
        let output = self.make_pin(kind, "out".into(), location);
        let buf = self.make_component(
            ComponentKind::Buffer(BufferComponent { input, output }),
            location,
        );
        self.pin_mut(input).parent(buf);
        self.pin_mut(output).parent(buf);
        (input, output)
    }
    pub fn make_component(
        &mut self,
        kind: ComponentKind,
        location: Option<SourceLocation>,
    ) -> ComponentIx {
        let ix = ComponentIx(self.components.len());
        self.components.push(Component {
            path: vec![],
            kind,
            location,
        });
        ix
    }
    pub fn pin(&self, ix: PinIx) -> &Pin {
        &self.pins[ix.0]
    }
    pub fn pin_mut(&mut self, ix: PinIx) -> &mut Pin {
        &mut self.pins[ix.0]
    }
    pub fn wire(&mut self, source: PinIx, dest: PinIx) {
        self.wires.push(Wire { source, dest });
    }
    pub fn component(&self, ix: ComponentIx) -> &Component {
        &self.components[ix.0]
    }
    // Inline all of the Components that are Kernel invocations into
    // this schematic by replacing the KernelComponent with the
    // sub_schematic.  This can be done recursively, but when the
    // components are all merged together, the group numbers will need
    // to be sorted out so as to be unique.  Or something.
    pub fn inlined(self) -> Schematic {
        let mut output_schematic = Schematic {
            pins: self.pins,
            wires: self.wires,
            inputs: self.inputs,
            output: self.output,
            source: self.source,
            ..Default::default()
        };
        let mut relocation_offset = self.components.len();
        let mut extra_components = vec![];
        let mut kernel_calls: HashMap<String, i32> = HashMap::new();
        for component in self.components {
            match component.kind {
                ComponentKind::Kernel(kernel) => {
                    let sub_schematic = kernel.sub_schematic.inlined();
                    // Remap all of the sub schematic pins to our schematic, by
                    // offsetting them to adjust for their component indices and
                    // pin indices.
                    let pin_offset = output_schematic.pins.len();
                    let component_offset = relocation_offset;
                    relocation_offset += sub_schematic.components.len();
                    // Add the pins from the sub schematic to the output schematic
                    // and adjust their parent pointers to point to the new
                    // component index.
                    output_schematic
                        .pins
                        .extend(sub_schematic.pins.into_iter().map(|mut p| {
                            p.parent = p.parent.offset(component_offset);
                            p
                        }));
                    // Fix up the wires to point to the new pin indices.
                    output_schematic
                        .wires
                        .extend(sub_schematic.wires.into_iter().map(|w| Wire {
                            source: w.source.offset(pin_offset),
                            dest: w.dest.offset(pin_offset),
                        }));
                    // Add the components from the sub schematic to the output
                    // schematic and adjust their path to point to the new
                    // component index.
                    let unique_name = {
                        let call_count = kernel_calls.entry(kernel.name.clone()).or_insert(0);
                        *call_count += 1;
                        format!("{}_{}", kernel.name, call_count)
                    };
                    extra_components.extend(
                        sub_schematic
                            .components
                            .into_iter()
                            .map(|c| c.offset(&unique_name, pin_offset)),
                    );
                    output_schematic.components.push(Component {
                        path: vec![],
                        kind: ComponentKind::Noop,
                        location: None,
                    });
                    let relocations = kernel
                        .args
                        .iter()
                        .zip(sub_schematic.inputs)
                        .map(|(arg, pin)| (*arg, pin.offset(pin_offset)))
                        .chain(std::iter::once((
                            kernel.output,
                            sub_schematic.output.offset(pin_offset),
                        )))
                        .collect::<HashMap<PinIx, PinIx>>();
                    output_schematic.wires = output_schematic
                        .wires
                        .into_iter()
                        .map(|w| w.relocate(&relocations))
                        .collect();
                    output_schematic.source.extend(sub_schematic.source);
                }
                _ => {
                    output_schematic.components.push(component);
                }
            }
        }
        output_schematic.components.extend(extra_components);
        output_schematic
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PinPath {
    pub pin: PinIx,
    pub path: Path,
}

pub fn pin_path(pin: PinIx, path: Path) -> PinPath {
    PinPath { pin, path }
}

#[derive(Debug, Clone)]
pub struct WirePath {
    pub source: PinIx,
    pub dest: PinIx,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct Trace {
    pub source: PinPath,
    pub paths: Vec<WirePath>,
    pub sinks: Vec<PinPath>,
}

impl From<PinPath> for Trace {
    fn from(source: PinPath) -> Self {
        Trace {
            source,
            paths: vec![],
            sinks: vec![],
        }
    }
}

impl Trace {
    pub fn all_pin_paths(&self) -> impl Iterator<Item = PinPath> + '_ {
        self.paths
            .iter()
            .flat_map(|wp| {
                vec![
                    pin_path(wp.source, wp.path.clone()),
                    pin_path(wp.dest, wp.path.clone()),
                ]
            })
            .chain(std::iter::once(self.source.clone()))
            .chain(self.sinks.iter().cloned())
    }
}
