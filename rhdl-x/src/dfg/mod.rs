use std::collections::HashMap;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

use petgraph::stable_graph::DefaultIx;
use petgraph::stable_graph::NodeIndex;
use petgraph::Directed;
use petgraph::Graph;
use rhdl_core::ast::ast_impl::FunctionId;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::Kernel;
use rhdl_core::path::Path;
use rhdl_core::rhif::spec::Array;
use rhdl_core::rhif::spec::Assign;
use rhdl_core::rhif::spec::Case;
use rhdl_core::rhif::spec::Cast;
use rhdl_core::rhif::spec::Discriminant;
use rhdl_core::rhif::spec::Enum;
use rhdl_core::rhif::spec::Exec;
use rhdl_core::rhif::spec::ExternalFunctionCode;
use rhdl_core::rhif::spec::Index;
use rhdl_core::rhif::spec::Repeat;
use rhdl_core::rhif::spec::Select;
use rhdl_core::rhif::spec::Splice;
use rhdl_core::rhif::spec::Struct;
use rhdl_core::rhif::spec::Tuple;
use rhdl_core::rhif::spec::Unary;
use rhdl_core::Design;
use rhdl_core::KernelFnKind;
use rhdl_core::{
    rhif::{
        object::SourceLocation,
        spec::{AluBinary, AluUnary, Binary, OpCode, Slot},
        Object,
    },
    Kind,
};

type DFGType = Graph<Component, Link, Directed>;

#[derive(Clone, Debug)]
pub struct DFG {
    pub graph: DFGType,
    pub arguments: Vec<NodeIndex<u32>>,
    pub ret: NodeIndex<u32>,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub path: Path,
}

impl std::fmt::Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

// Todo
// Support for struct and enum
// Support for dynamic paths
// Support for non-kernel external calls

#[derive(Debug, Clone)]
pub struct Component {
    pub input: Kind,
    pub output: Kind,
    pub kind: ComponentKind,
    pub location: Option<SourceLocation>,
}

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Debug, Clone)]
pub enum ComponentKind {
    Buffer(String),
    Binary(AluBinary),
    Unary(AluUnary),
    Select,
    Index(Path),
    Splice,
    Repeat,
    Struct,
    Tuple,
    Case,
    Exec(String),
    Array,
    Discriminant,
    Enum,
    Cast,
    Constant,
}

impl std::fmt::Display for ComponentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentKind::Buffer(reason) => write!(f, "Buffer({})", reason),
            ComponentKind::Binary(op) => write!(f, "{:?}", op),
            ComponentKind::Unary(op) => write!(f, "{:?}", op),
            ComponentKind::Select => write!(f, "Select"),
            ComponentKind::Index(path) => write!(f, "Index({})", path),
            ComponentKind::Splice => write!(f, "Splice"),
            ComponentKind::Repeat => write!(f, "Repeat"),
            ComponentKind::Struct => write!(f, "Struct"),
            ComponentKind::Tuple => write!(f, "Tuple"),
            ComponentKind::Case => write!(f, "Case"),
            ComponentKind::Exec(name) => write!(f, "Exec({name})"),
            ComponentKind::Array => write!(f, "Array"),
            ComponentKind::Discriminant => write!(f, "Discriminant"),
            ComponentKind::Enum => write!(f, "Enum"),
            ComponentKind::Cast => write!(f, "Cast"),
            ComponentKind::Constant => write!(f, "Constant"),
        }
    }
}

// This is a macro by example that allows me to easily define ad hoc
// structs using a simple syntax.  So
//  digital_struct!( name {
//      field1: expr1,
//      field2: expr2,
//  })
// expands to
//  Kind::make_struct("name", vec![Kind::make_field("field1", expr1), Kind::make_field("field2", expr2)])
macro_rules! digital_struct {
    ($name:ident { $($field:ident: $value:expr),* }) => {
        Kind::make_struct(stringify!($name), vec![$(Kind::make_field(stringify!($field), $value)),*])
    };
}

#[derive(Debug)]
pub struct ObjectAnalyzer<'a> {
    design: &'a Design,
    object: &'a Object,
    graph: DFGType,
    slot_map: HashMap<Slot, NodeIndex<u32>>,
}

pub fn build_dfg(design: &Design, function: FunctionId) -> Result<DFG> {
    let object = design.objects.get(&function).ok_or(anyhow!(
        "Function {:?} not found in design {:?}",
        function,
        design
    ))?;
    let analyzer = ObjectAnalyzer::new(design, object);
    analyzer.build()
}

impl<'a> ObjectAnalyzer<'a> {
    pub fn new(design: &'a Design, object: &'a Object) -> Self {
        Self {
            design,
            object,
            graph: DFGType::new(),
            slot_map: HashMap::new(),
        }
    }

    pub fn build(mut self) -> Result<DFG> {
        for arg in &self.object.arguments {
            let kind = self.kind(*arg)?;
            let location = self.location(*arg).ok();
            self.add_node(
                Component {
                    input: kind.clone(),
                    output: kind,
                    kind: ComponentKind::Buffer(format!("Arg{:?}", arg)),
                    location,
                },
                *arg,
            );
        }
        for (ndx, literal) in self.object.literals.iter().enumerate() {
            let kind = literal.kind.clone();
            let slot = Slot::Literal(ndx);
            let location = self.location(slot).ok();
            self.add_node(
                Component {
                    input: Kind::Empty,
                    output: kind,
                    kind: ComponentKind::Constant,
                    location,
                },
                slot,
            );
        }
        for (op, location) in self
            .object
            .ops
            .iter()
            .cloned()
            .zip(self.object.opcode_map.iter().cloned())
        {
            match op {
                OpCode::Binary(binary) => self.binary(binary, location),
                OpCode::Unary(unary) => self.unary(unary, location),
                OpCode::Select(select) => self.select(select, location),
                OpCode::Index(index) => self.index(index, location),
                OpCode::Splice(splice) => self.splice(splice, location),
                OpCode::Repeat(repeat) => self.repeat(repeat, location),
                OpCode::Struct(structure) => self.mk_struct(structure, location),
                OpCode::Tuple(tuple) => self.tuple(tuple, location),
                OpCode::Case(case) => self.case(case, location),
                OpCode::Array(array) => self.array(array, location),
                OpCode::Discriminant(discriminant) => self.discriminant(discriminant, location),
                OpCode::Enum(enumerate) => self.enumerate(enumerate, location),
                OpCode::AsBits(cast) => self.cast(cast, location),
                OpCode::AsSigned(cast) => self.cast(cast, location),
                OpCode::Assign(assign) => self.assign(assign, location),
                OpCode::Exec(exec) => self.exec(exec, location),
                OpCode::Noop | OpCode::Comment(_) => Ok(()),
            }?
        }
        let arguments = self
            .object
            .arguments
            .iter()
            .map(|arg| self.node(*arg).unwrap())
            .collect();
        let ret = self.node(self.object.return_slot)?;
        Ok(DFG {
            graph: self.graph,
            arguments,
            ret,
        })
    }

    fn exec(&mut self, exec: Exec, location: SourceLocation) -> Result<()> {
        let func = &self.object.externals[exec.id.0];
        match &func.code {
            ExternalFunctionCode::Kernel(kernel) => self.exec_kernel(exec, kernel),
            ExternalFunctionCode::Extern(extern_fn) => self.exec_extern(exec, extern_fn),
        }
    }

    fn exec_extern(&mut self, exec: Exec, extern_fn: &ExternalKernelDef) -> Result<()> {
        let input = Kind::make_tuple(
            exec.args
                .iter()
                .map(|arg| self.kind(*arg))
                .collect::<Result<Vec<_>>>()?,
        );
        let output = self.kind(exec.lhs)?;
        let node = self.add_node(
            Component {
                input,
                output,
                kind: ComponentKind::Exec(extern_fn.name.clone()),
                location: None,
            },
            exec.lhs,
        );
        for (ndx, slot) in exec.args.iter().enumerate() {
            self.add_edge(*slot, node, Path::default().index(ndx))?;
        }
        Ok(())
    }

    fn exec_kernel(&mut self, exec: Exec, kernel: &Kernel) -> Result<()> {
        // Build a DFG for the callee.
        let callee_dfg = build_dfg(self.design, kernel.inner().fn_id)?;
        // Insert the callee's DFG into the current DFG, keeping track of
        // the mapping for the node IDs from callee to caller.
        let mut node_map = HashMap::new();
        for nodes in callee_dfg.graph.node_indices() {
            let node = callee_dfg.graph.node_weight(nodes).unwrap();
            let new_index = self.graph.add_node(node.clone());
            node_map.insert(nodes, new_index);
        }
        // Iterate over the edges, and add them to the current DFG, using the
        // node_map to map the node indices from callee to caller.
        for edges in callee_dfg.graph.edge_indices() {
            let (source, target) = callee_dfg.graph.edge_endpoints(edges).unwrap();
            let source = node_map.get(&source).unwrap();
            let target = node_map.get(&target).unwrap();
            let link = callee_dfg.graph.edge_weight(edges).unwrap();
            self.graph.add_edge(*source, *target, link.clone());
        }
        // Link the arguments and the return
        for (callee_arg, caller_arg) in callee_dfg.arguments.iter().zip(exec.args.iter()) {
            let caller_arg_in_current_scope = self.node(*caller_arg)?;
            let callee_arg_remapped = node_map.get(callee_arg).unwrap();
            self.graph.add_edge(
                caller_arg_in_current_scope,
                *callee_arg_remapped,
                Link {
                    path: Path::default(),
                },
            );
        }
        let callee_ret_remapped = node_map.get(&callee_dfg.ret).unwrap();
        let caller_ret = self.buffer(exec.lhs, "Exec".to_string())?;
        self.graph.add_edge(
            *callee_ret_remapped,
            caller_ret,
            Link {
                path: Path::default(),
            },
        );
        Ok(())
    }

    fn cast(&mut self, cast: Cast, location: SourceLocation) -> Result<()> {
        let input = self.kind(cast.arg)?;
        let output = self.kind(cast.lhs)?;
        let node = self.add_node(
            Component {
                input,
                output,
                kind: ComponentKind::Cast,
                location: Some(location),
            },
            cast.lhs,
        );
        self.add_edge(cast.arg, node, Path::default())
    }

    fn enumerate(&self, enumerate: Enum, location: SourceLocation) -> Result<()> {
        todo!()
    }

    fn discriminant(&mut self, discriminant: Discriminant, location: SourceLocation) -> Result<()> {
        let input = self.kind(discriminant.arg)?;
        let output = self.kind(discriminant.lhs)?;
        let node = self.add_node(
            Component {
                input,
                output,
                kind: ComponentKind::Discriminant,
                location: Some(location),
            },
            discriminant.lhs,
        );
        self.add_edge(discriminant.arg, node, Path::default())
    }

    fn array(&mut self, array: Array, location: SourceLocation) -> Result<()> {
        let input = Kind::make_tuple(
            array
                .elements
                .iter()
                .map(|arg| self.kind(*arg))
                .collect::<Result<Vec<_>>>()?,
        );
        let output = self.kind(array.lhs)?;
        let node = self.add_node(
            Component {
                input,
                output,
                kind: ComponentKind::Array,
                location: Some(location),
            },
            array.lhs,
        );
        for (ndx, slot) in array.elements.iter().enumerate() {
            self.add_edge(*slot, node, Path::default().index(ndx))?;
        }
        Ok(())
    }

    fn tuple(&mut self, tuple: Tuple, location: SourceLocation) -> Result<()> {
        let input = Kind::make_tuple(
            tuple
                .fields
                .iter()
                .map(|arg| self.kind(*arg))
                .collect::<Result<Vec<_>>>()?,
        );
        let output = self.kind(tuple.lhs)?;
        let node = self.add_node(
            Component {
                input,
                output,
                kind: ComponentKind::Tuple,
                location: Some(location),
            },
            tuple.lhs,
        );
        for (ndx, slot) in tuple.fields.iter().enumerate() {
            self.add_edge(*slot, node, Path::default().index(ndx))?;
        }
        Ok(())
    }

    fn mk_struct(&self, structure: Struct, location: SourceLocation) -> Result<()> {
        todo!()
    }

    fn case(&mut self, case: Case, location: SourceLocation) -> Result<()> {
        let input_kind = self.kind(case.lhs)?;
        let discriminant = self.kind(case.discriminant)?;
        let input = digital_struct!(case {
            discriminant: discriminant,
            table: Kind::make_array(input_kind, case.table.len())
        });
        let output = self.kind(case.lhs)?;
        let node = self.add_node(
            Component {
                input,
                output,
                kind: ComponentKind::Case,
                location: Some(location),
            },
            case.lhs,
        );
        self.add_edge(
            case.discriminant,
            node,
            Path::default().field("discriminant"),
        )?;
        for (ndx, (_, slot)) in case.table.iter().enumerate() {
            self.add_edge(*slot, node, Path::default().field("table").index(ndx))?;
        }
        Ok(())
    }

    fn repeat(&mut self, repeat: Repeat, location: SourceLocation) -> Result<()> {
        let input = self.kind(repeat.value)?;
        let output = self.kind(repeat.lhs)?;
        let node = self.add_node(
            Component {
                input,
                output,
                kind: ComponentKind::Repeat,
                location: Some(location),
            },
            repeat.lhs,
        );
        self.add_edge(repeat.value, node, Path::default())
    }
    fn splice(&mut self, splice: Splice, location: SourceLocation) -> Result<()> {
        let orig = self.kind(splice.orig)?;
        let subst = self.kind(splice.subst)?;
        let input = digital_struct!(splice {
            orig: orig,
            subst: subst
        });
        let output = self.kind(splice.lhs)?;
        let node = self.add_node(
            Component {
                input,
                output,
                kind: ComponentKind::Splice,
                location: Some(location),
            },
            splice.lhs,
        );
        for slot in splice.path.dynamic_slots() {
            self.add_edge(*slot, node, Path::default())?;
        }
        self.add_edge(splice.orig, node, Path::default().field("orig"))?;
        self.add_edge(splice.subst, node, Path::default().field("subst"))?;
        Ok(())
    }

    fn buffer(&mut self, value: Slot, reason: String) -> Result<NodeIndex<u32>> {
        let kind = self.kind(value)?;
        let node = self.add_node(
            Component {
                input: kind.clone(),
                output: kind,
                kind: ComponentKind::Buffer(reason),
                location: None,
            },
            value,
        );
        Ok(node)
    }

    fn assign(&mut self, assign: Assign, location: SourceLocation) -> Result<()> {
        let input = self.kind(assign.rhs)?;
        let output = self.kind(assign.lhs)?;
        let node = self.add_node(
            Component {
                input,
                output,
                kind: ComponentKind::Buffer(format!("Assign{:?}", assign.lhs)),
                location: Some(location),
            },
            assign.lhs,
        );
        self.add_edge(assign.rhs, node, Path::default())
    }

    fn index(&mut self, index: Index, location: SourceLocation) -> Result<()> {
        let input = self.kind(index.arg)?;
        let output = self.kind(index.lhs)?;
        let node = self.add_node(
            Component {
                input,
                output,
                kind: ComponentKind::Index(index.path.clone()),
                location: Some(location),
            },
            index.lhs,
        );
        for slot in index.path.dynamic_slots() {
            self.add_edge(*slot, node, Path::default())?;
        }
        self.add_edge(index.arg, node, Path::default())
    }

    fn select(&mut self, select: Select, location: SourceLocation) -> Result<()> {
        let cond = self.kind(select.cond)?;
        let true_value = self.kind(select.true_value)?;
        let false_value = self.kind(select.false_value)?;
        let input = digital_struct!(select {
            cond: cond,
            true_value: true_value,
            false_value: false_value
        });
        let output = self.kind(select.lhs)?;
        let node = self.add_node(
            Component {
                input,
                output,
                kind: ComponentKind::Select,
                location: Some(location),
            },
            select.lhs,
        );
        self.add_edge(select.cond, node, Path::default().field("cond"))?;
        self.add_edge(select.true_value, node, Path::default().field("true_value"))?;
        self.add_edge(
            select.false_value,
            node,
            Path::default().field("false_value"),
        )?;
        Ok(())
    }

    fn binary(&mut self, binary: Binary, location: SourceLocation) -> Result<()> {
        let arg1 = self.kind(binary.arg1)?;
        let arg2 = self.kind(binary.arg2)?;
        let input = Kind::make_tuple(vec![arg1, arg2]);
        let output = self.kind(binary.lhs)?;
        let kind = ComponentKind::Binary(binary.op);
        let node = self.add_node(
            Component {
                input,
                output,
                kind,
                location: Some(location),
            },
            binary.lhs,
        );
        self.add_edge(binary.arg1, node, Path::default().index(0))?;
        self.add_edge(binary.arg2, node, Path::default().index(1))?;
        Ok(())
    }

    fn unary(&mut self, unary: Unary, location: SourceLocation) -> Result<()> {
        let input = self.kind(unary.arg1)?;
        let output = self.kind(unary.lhs)?;
        let kind = ComponentKind::Unary(unary.op);
        let node = self.add_node(
            Component {
                input,
                output,
                kind,
                location: Some(location),
            },
            unary.lhs,
        );
        self.add_edge(unary.arg1, node, Path::default())?;
        Ok(())
    }

    fn kind(&self, slot: Slot) -> Result<Kind> {
        let Some(ty) = self.object.kind.get(&slot) else {
            bail!("Slot {:?} not found in object {:?}", slot, self.object)
        };
        Ok(ty.clone())
    }

    fn node(&self, slot: Slot) -> Result<NodeIndex<u32>> {
        let Some(ix) = self.slot_map.get(&slot) else {
            bail!("Slot {:?} not found in slot_map {:?}", slot, self.slot_map)
        };
        Ok(*ix)
    }

    fn add_edge(&mut self, slot: Slot, node: NodeIndex<u32>, path: Path) -> Result<()> {
        let ix = self.node(slot)?;
        self.graph.add_edge(ix, node, Link { path });
        Ok(())
    }

    fn add_node(&mut self, node: Component, slot: Slot) -> NodeIndex<u32> {
        let ix = self.graph.add_node(node);
        self.slot_map.insert(slot, ix);
        ix
    }

    fn location(&self, slot: Slot) -> Result<SourceLocation> {
        let Some(location) = self.object.slot_map.get(&slot) else {
            bail!(
                "Slot {:?} not found in slot_map {:?}",
                slot,
                self.object.slot_map
            )
        };
        Ok(*location)
    }
}
