use std::collections::HashMap;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

use crate::ast::ast_impl::FunctionId;
use crate::kernel::Kernel;
use crate::rhif::object::LocatedOpCode;
use crate::rhif::object::SourceLocation;
use crate::rhif::spec::Retime;
use crate::rhif::spec::{
    Array, Assign, Case, Cast, Enum, Exec, Index, Repeat, Select, Splice, Struct, Tuple, Unary,
};
use crate::Module;
use crate::TypedBits;
use crate::{
    rhif::{
        spec::{Binary, OpCode, Slot},
        Object,
    },
    Kind,
};

use super::components::UnaryComponent;
use super::components::{
    ArrayComponent, BinaryComponent, BufferComponent, CaseComponent, CastComponent, ComponentKind,
    ConstantComponent, EnumComponent, FieldPin, IndexComponent, KernelComponent, RepeatComponent,
    SelectComponent, SpliceComponent, StructComponent, TupleComponent,
};
use super::schematic_impl::PinIx;
use super::schematic_impl::Schematic;

#[derive(Debug)]
pub struct SchematicBuilder<'a> {
    module: &'a Module,
    object: &'a Object,
    schematic: Schematic,
    slot_map: HashMap<Slot, PinIx>,
}

pub fn build_schematic(module: &Module, function: FunctionId) -> Result<Schematic> {
    let object = module.objects.get(&function).ok_or(anyhow!(
        "Function {:?} not found in module {:?}",
        function,
        module
    ))?;
    let analyzer = SchematicBuilder::new(module, object);
    analyzer.build()
}

impl<'a> SchematicBuilder<'a> {
    fn new(module: &'a Module, object: &'a Object) -> Self {
        Self {
            module,
            object,
            schematic: Schematic::default(),
            slot_map: HashMap::new(),
        }
    }

    fn build(mut self) -> Result<Schematic> {
        for arg in &self.object.arguments {
            let kind = self.kind(Slot::Register(*arg))?;
            let (ipin, opin) = self.make_buffer(
                format!("{:?}", arg),
                kind,
                self.slot_source(Slot::Register(*arg)),
            );
            self.schematic.inputs.push(ipin);
            self.bind(Slot::Register(*arg), opin);
        }
        for (&slot, literal) in self.object.literals.iter() {
            if literal.bits.is_empty() {
                continue;
            }
            let source = self.slot_source(slot.into());
            let opin = self.make_constant(literal, source);
            self.bind(slot.into(), opin);
        }
        for lop in self.object.ops.iter().cloned() {
            let LocatedOpCode { op, id } = lop;
            let location = SourceLocation {
                func: self.object.fn_id,
                node: id,
            };
            match op {
                OpCode::Binary(binary) => self.make_binary(binary, Some(location)),
                OpCode::Unary(unary) => self.make_unary(unary, Some(location)),
                OpCode::Select(select) => self.make_select(select, Some(location)),
                OpCode::Index(index) => self.make_index(index, Some(location)),
                OpCode::Splice(splice) => self.make_splice(splice, Some(location)),
                OpCode::Repeat(repeat) => self.make_repeat(repeat, Some(location)),
                OpCode::Struct(structure) => self.make_struct(structure, Some(location)),
                OpCode::Tuple(tuple) => self.make_tuple(tuple, Some(location)),
                OpCode::Case(case) => self.make_case(case, Some(location)),
                OpCode::Array(array) => self.make_array(array, Some(location)),
                OpCode::Enum(enumerate) => self.make_enum(enumerate, Some(location)),
                OpCode::AsBits(cast) | OpCode::AsSigned(cast) => {
                    self.make_cast(cast, Some(location))
                }
                OpCode::Retime(cast) => self.make_retimed(cast, Some(location)),
                OpCode::Assign(assign) => self.make_assign(assign, Some(location)),
                OpCode::Exec(exec) => self.make_exec(exec, Some(location)),
                OpCode::Noop | OpCode::Comment(_) => Ok(()),
            }?
        }
        self.schematic.output = self.lookup(self.object.return_slot)?;
        self.schematic.source = self
            .module
            .objects
            .iter()
            .map(|(k, v)| (*k, v.symbols.source.clone()))
            .collect();
        Ok(self.schematic)
    }

    fn kind(&self, slot: Slot) -> Result<Kind> {
        Ok(self.object.kind(slot))
    }

    fn bind(&mut self, slot: Slot, pin: PinIx) {
        self.slot_map.insert(slot, pin);
    }

    fn lookup(&self, slot: Slot) -> Result<PinIx> {
        let Some(pin) = self.slot_map.get(&slot) else {
            bail!("Slot {:?} not found in slot_map {:?}", slot, self.slot_map)
        };
        Ok(*pin)
    }

    fn slot_source(&self, slot: Slot) -> Option<SourceLocation> {
        self.object
            .symbols
            .slot_map
            .get(&slot)
            .cloned()
            .map(|x| (self.object.fn_id, x).into())
    }

    fn make_output_pin(&mut self, slot: Slot) -> Result<PinIx> {
        let kind = self.kind(slot)?;
        let location = self.slot_source(slot);
        let pin = self
            .schematic
            .make_pin(kind, format!("{:?}", slot), location);
        self.bind(slot, pin);
        Ok(pin)
    }

    fn make_buffer(
        &mut self,
        name: String,
        kind: Kind,
        location: Option<SourceLocation>,
    ) -> (PinIx, PinIx) {
        let input = self
            .schematic
            .make_pin(kind.clone(), format!("{}_in", name), location);
        let output = self
            .schematic
            .make_pin(kind, format!("{}_out", name), location);
        let component = self.schematic.make_component(
            ComponentKind::Buffer(BufferComponent { input, output }),
            location,
        );
        self.schematic.pin_mut(input).parent(component);
        self.schematic.pin_mut(output).parent(component);
        (input, output)
    }

    fn make_constant(&mut self, value: &TypedBits, location: Option<SourceLocation>) -> PinIx {
        let output = self
            .schematic
            .make_pin(value.kind.clone(), "constant".to_string(), location);
        let component = self.schematic.make_component(
            ComponentKind::Constant(ConstantComponent {
                value: value.clone(),
                output,
            }),
            location,
        );
        self.schematic.pin_mut(output).parent(component);
        output
    }

    // Lookup the pin that drives the given slot, create a new
    // pin of the same type, and tie the two together with a wire.
    fn make_wired_pin(&mut self, slot: Slot) -> Result<PinIx> {
        let kind = self.kind(slot)?;
        let source = self.slot_source(slot);
        let pin = self.schematic.make_pin(kind, format!("{:?}", slot), source);
        let output = self.lookup(slot)?;
        self.schematic.wire(output, pin);
        Ok(pin)
    }

    fn make_binary(&mut self, binary: Binary, location: Option<SourceLocation>) -> Result<()> {
        let arg1 = self.make_wired_pin(binary.arg1)?;
        let arg2 = self.make_wired_pin(binary.arg2)?;
        let out = self.make_output_pin(binary.lhs)?;
        let component = self.schematic.make_component(
            ComponentKind::Binary(BinaryComponent {
                op: binary.op,
                input1: arg1,
                input2: arg2,
                output: out,
            }),
            location,
        );
        self.schematic.pin_mut(arg1).parent(component);
        self.schematic.pin_mut(arg2).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_unary(&mut self, unary: Unary, location: Option<SourceLocation>) -> Result<()> {
        let arg1 = self.make_wired_pin(unary.arg1)?;
        let out = self.make_output_pin(unary.lhs)?;
        let component = self.schematic.make_component(
            ComponentKind::Unary(UnaryComponent {
                op: unary.op,
                input: arg1,
                output: out,
            }),
            location,
        );
        self.schematic.pin_mut(arg1).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_select(&mut self, select: Select, location: Option<SourceLocation>) -> Result<()> {
        let cond = self.make_wired_pin(select.cond)?;
        let true_value = self.make_wired_pin(select.true_value)?;
        let false_value = self.make_wired_pin(select.false_value)?;
        let out = self.make_output_pin(select.lhs)?;
        let component = self.schematic.make_component(
            ComponentKind::Select(SelectComponent {
                cond,
                true_value,
                false_value,
                output: out,
            }),
            location,
        );
        self.schematic.pin_mut(cond).parent(component);
        self.schematic.pin_mut(true_value).parent(component);
        self.schematic.pin_mut(false_value).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_index(&mut self, index: Index, location: Option<SourceLocation>) -> Result<()> {
        let arg = self.make_wired_pin(index.arg)?;
        let out = self.make_output_pin(index.lhs)?;
        let dynamic = index
            .path
            .dynamic_slots()
            .map(|slot| self.make_wired_pin(*slot))
            .collect::<Result<Vec<_>>>()?;
        let component = self.schematic.make_component(
            ComponentKind::Index(IndexComponent {
                arg,
                path: index.path.clone(),
                output: out,
                dynamic: dynamic.clone(),
                kind: self.schematic.pin(arg).kind.clone(),
            }),
            location,
        );
        self.schematic.pin_mut(arg).parent(component);
        self.schematic.pin_mut(out).parent(component);
        for pin in dynamic {
            self.schematic.pin_mut(pin).parent(component);
        }
        Ok(())
    }

    fn make_splice(&mut self, splice: Splice, location: Option<SourceLocation>) -> Result<()> {
        let orig = self.make_wired_pin(splice.orig)?;
        let subst = self.make_wired_pin(splice.subst)?;
        let out = self.make_output_pin(splice.lhs)?;
        let dynamic = splice
            .path
            .dynamic_slots()
            .map(|slot| self.make_wired_pin(*slot))
            .collect::<Result<Vec<_>>>()?;
        let component = self.schematic.make_component(
            ComponentKind::Splice(SpliceComponent {
                orig,
                subst,
                output: out,
                path: splice.path.clone(),
                dynamic: dynamic.clone(),
                kind: self.schematic.pin(orig).kind.clone(),
            }),
            location,
        );
        self.schematic.pin_mut(orig).parent(component);
        self.schematic.pin_mut(subst).parent(component);
        self.schematic.pin_mut(out).parent(component);
        for pin in dynamic {
            self.schematic.pin_mut(pin).parent(component);
        }
        Ok(())
    }

    fn make_repeat(&mut self, repeat: Repeat, location: Option<SourceLocation>) -> Result<()> {
        let value = self.make_wired_pin(repeat.value)?;
        let out = self.make_output_pin(repeat.lhs)?;
        let repeat = repeat.len;
        let component = self.schematic.make_component(
            ComponentKind::Repeat(RepeatComponent {
                value,
                output: out,
                len: repeat,
            }),
            location,
        );
        self.schematic.pin_mut(value).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_struct(&mut self, structure: Struct, location: Option<SourceLocation>) -> Result<()> {
        let fields = structure
            .fields
            .into_iter()
            .map(|f| {
                self.make_wired_pin(f.value).map(|pin| FieldPin {
                    member: f.member,
                    pin,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let out = self.make_output_pin(structure.lhs)?;
        let rest = structure.rest.map(|r| self.make_wired_pin(r)).transpose()?;
        let component = self.schematic.make_component(
            ComponentKind::Struct(StructComponent {
                kind: structure.template.kind,
                fields: fields.clone(),
                output: out,
                rest,
            }),
            location,
        );
        fields
            .iter()
            .for_each(|f| self.schematic.pin_mut(f.pin).parent(component));
        if let Some(pin) = rest {
            self.schematic.pin_mut(pin).parent(component);
        }
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_tuple(&mut self, tuple: Tuple, location: Option<SourceLocation>) -> Result<()> {
        let fields = tuple
            .fields
            .into_iter()
            .map(|f| self.make_wired_pin(f))
            .collect::<Result<Vec<_>>>()?;
        let out = self.make_output_pin(tuple.lhs)?;
        let component = self.schematic.make_component(
            ComponentKind::Tuple(TupleComponent {
                fields: fields.clone(),
                output: out,
            }),
            location,
        );
        fields
            .iter()
            .for_each(|f| self.schematic.pin_mut(*f).parent(component));
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_case(&mut self, case: Case, location: Option<SourceLocation>) -> Result<()> {
        let discriminant = self.make_wired_pin(case.discriminant)?;
        let table = case
            .table
            .into_iter()
            .map(|(ndx, slot)| self.make_wired_pin(slot).map(|pin| (ndx, pin)))
            .collect::<Result<Vec<_>>>()?;
        let out = self.make_output_pin(case.lhs)?;
        let component = self.schematic.make_component(
            ComponentKind::Case(CaseComponent {
                discriminant,
                table: table.clone(),
                output: out,
            }),
            location,
        );
        self.schematic.pin_mut(discriminant).parent(component);
        for (_, pin) in table {
            self.schematic.pin_mut(pin).parent(component);
        }
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_array(&mut self, array: Array, location: Option<SourceLocation>) -> Result<()> {
        let elements = array
            .elements
            .into_iter()
            .map(|slot| self.make_wired_pin(slot))
            .collect::<Result<Vec<_>>>()?;
        let out = self.make_output_pin(array.lhs)?;
        let component = self.schematic.make_component(
            ComponentKind::Array(ArrayComponent {
                elements: elements.clone(),
                output: out,
            }),
            location,
        );
        elements
            .iter()
            .for_each(|f| self.schematic.pin_mut(*f).parent(component));
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_enum(&mut self, enumerate: Enum, location: Option<SourceLocation>) -> Result<()> {
        let fields = enumerate
            .fields
            .into_iter()
            .map(|f| {
                self.make_wired_pin(f.value).map(|pin| FieldPin {
                    member: f.member,
                    pin,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let out = self.make_output_pin(enumerate.lhs)?;
        let component = self.schematic.make_component(
            ComponentKind::Enum(EnumComponent {
                fields: fields.clone(),
                output: out,
                template: enumerate.template.clone(),
            }),
            location,
        );
        fields
            .iter()
            .for_each(|f| self.schematic.pin_mut(f.pin).parent(component));
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_cast(&mut self, cast: Cast, location: Option<SourceLocation>) -> Result<()> {
        let arg = self.make_wired_pin(cast.arg)?;
        let out = self.make_output_pin(cast.lhs)?;
        let component = self.schematic.make_component(
            ComponentKind::Cast(CastComponent {
                input: arg,
                output: out,
            }),
            location,
        );
        self.schematic.pin_mut(arg).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_retimed(&mut self, cast: Retime, location: Option<SourceLocation>) -> Result<()> {
        let arg = self.make_wired_pin(cast.arg)?;
        let out = self.make_output_pin(cast.lhs)?;
        let component = self.schematic.make_component(
            ComponentKind::Cast(CastComponent {
                input: arg,
                output: out,
            }),
            location,
        );
        self.schematic.pin_mut(arg).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_assign(&mut self, assign: Assign, location: Option<SourceLocation>) -> Result<()> {
        let arg = self.make_wired_pin(assign.rhs)?;
        let out = self.make_output_pin(assign.lhs)?;
        let component = self.schematic.make_component(
            ComponentKind::Buffer(BufferComponent {
                input: arg,
                output: out,
            }),
            location,
        );
        self.schematic.pin_mut(arg).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_exec(&mut self, exec: Exec, location: Option<SourceLocation>) -> Result<()> {
        let code = &self.object.externals[&exec.id].code;
        self.make_kernel(exec, code, location)
    }

    fn make_kernel(
        &mut self,
        exec: Exec,
        kernel: &Kernel,
        location: Option<SourceLocation>,
    ) -> Result<()> {
        let args = exec
            .args
            .iter()
            .map(|arg| self.make_wired_pin(*arg))
            .collect::<Result<Vec<_>>>()?;
        let out = self.make_output_pin(exec.lhs)?;
        let sub_schematic = build_schematic(self.module, kernel.inner().fn_id)?;
        let component = self.schematic.make_component(
            ComponentKind::Kernel(KernelComponent {
                name: kernel.inner().name.to_owned(),
                args: args.clone(),
                output: out,
                sub_schematic,
            }),
            location,
        );
        args.iter()
            .for_each(|f| self.schematic.pin_mut(*f).parent(component));
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }
}
