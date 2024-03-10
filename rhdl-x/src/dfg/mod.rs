use std::collections::HashMap;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

use rhdl_core::ast::ast_impl::FunctionId;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::Kernel;
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
use rhdl_core::Module;
use rhdl_core::TypedBits;
use rhdl_core::{
    rhif::{
        spec::{Binary, OpCode, Slot},
        Object,
    },
    Kind,
};

use self::components::ArrayComponent;
use self::components::BinaryComponent;
use self::components::BlackBoxComponent;
use self::components::BufferComponent;
use self::components::CaseComponent;
use self::components::CastComponent;
use self::components::ComponentKind;
use self::components::ConstantComponent;
use self::components::DiscriminantComponent;
use self::components::EnumComponent;
use self::components::FieldPin;
use self::components::IndexComponent;
use self::components::KernelComponent;
use self::components::RepeatComponent;
use self::components::SelectComponent;
use self::components::SpliceComponent;
use self::components::StructComponent;
use self::components::TupleComponent;
use self::components::UnaryComponent;
use self::schematic::PinIx;
use self::schematic::Schematic;

pub mod components;
pub mod dot;
pub mod schematic;

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
            let kind = self.kind(*arg)?;
            let (ipin, opin) = self.make_buffer(format!("{}", arg), kind);
            self.schematic.inputs.push(ipin);
            self.bind(*arg, opin);
        }
        for (ndx, literal) in self.object.literals.iter().enumerate() {
            let kind = literal.kind.clone();
            let slot = Slot::Literal(ndx);
            let opin = self.make_constant(literal);
            self.bind(slot, opin);
        }
        for (op, location) in self
            .object
            .ops
            .iter()
            .cloned()
            .zip(self.object.opcode_map.iter().cloned())
        {
            match op {
                OpCode::Binary(binary) => self.make_binary(binary),
                OpCode::Unary(unary) => self.make_unary(unary),
                OpCode::Select(select) => self.make_select(select),
                OpCode::Index(index) => self.make_index(index),
                OpCode::Splice(splice) => self.make_splice(splice),
                OpCode::Repeat(repeat) => self.make_repeat(repeat),
                OpCode::Struct(structure) => self.make_struct(structure),
                OpCode::Tuple(tuple) => self.make_tuple(tuple),
                OpCode::Case(case) => self.make_case(case),
                OpCode::Array(array) => self.make_array(array),
                OpCode::Discriminant(discriminant) => self.make_discriminant(discriminant),
                OpCode::Enum(enumerate) => self.make_enum(enumerate),
                OpCode::AsBits(cast) | OpCode::AsSigned(cast) => self.make_cast(cast),
                OpCode::Assign(assign) => self.make_assign(assign),
                OpCode::Exec(exec) => self.make_exec(exec),
                OpCode::Noop | OpCode::Comment(_) => Ok(()),
            }?
        }
        self.schematic.output = self.lookup(self.object.return_slot)?;
        Ok(self.schematic)
    }

    fn kind(&self, slot: Slot) -> Result<Kind> {
        let Some(ty) = self.object.kind.get(&slot) else {
            bail!("Slot {:?} not found in object {:?}", slot, self.object)
        };
        Ok(ty.clone())
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

    fn make_output_pin(&mut self, slot: Slot) -> Result<PinIx> {
        let kind = self.kind(slot)?;
        let pin = self.schematic.make_pin(kind, format!("{}", slot));
        self.bind(slot, pin);
        Ok(pin)
    }

    fn make_buffer(&mut self, name: String, kind: Kind) -> (PinIx, PinIx) {
        let input = self
            .schematic
            .make_pin(kind.clone(), format!("{}_in", name));
        let output = self.schematic.make_pin(kind, format!("{}_out", name));
        let component = self
            .schematic
            .make_component(ComponentKind::Buffer(BufferComponent { input, output }));
        self.schematic.pin_mut(input).parent(component);
        self.schematic.pin_mut(output).parent(component);
        (input, output)
    }

    fn make_constant(&mut self, value: &TypedBits) -> PinIx {
        let output = self
            .schematic
            .make_pin(value.kind.clone(), "constant".to_string());
        let component = self
            .schematic
            .make_component(ComponentKind::Constant(ConstantComponent {
                value: value.clone(),
                output,
            }));
        self.schematic.pin_mut(output).parent(component);
        output
    }

    // Lookup the pin that drives the given slot, create a new
    // pin of the same type, and tie the two together with a wire.
    fn make_wired_pin(&mut self, slot: Slot) -> Result<PinIx> {
        let kind = self.kind(slot)?;
        let pin = self.schematic.make_pin(kind, format!("{}", slot));
        let output = self.lookup(slot)?;
        self.schematic.wire(output, pin);
        Ok(pin)
    }

    fn make_binary(&mut self, binary: Binary) -> Result<()> {
        let arg1 = self.make_wired_pin(binary.arg1)?;
        let arg2 = self.make_wired_pin(binary.arg2)?;
        let out = self.make_output_pin(binary.lhs)?;
        let component = self
            .schematic
            .make_component(ComponentKind::Binary(BinaryComponent {
                op: binary.op,
                input1: arg1,
                input2: arg2,
                output: out,
            }));
        self.schematic.pin_mut(arg1).parent(component);
        self.schematic.pin_mut(arg2).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_unary(&mut self, unary: Unary) -> Result<()> {
        let arg1 = self.make_wired_pin(unary.arg1)?;
        let out = self.make_output_pin(unary.lhs)?;
        let component = self
            .schematic
            .make_component(ComponentKind::Unary(UnaryComponent {
                op: unary.op,
                input: arg1,
                output: out,
            }));
        self.schematic.pin_mut(arg1).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_select(&mut self, select: Select) -> Result<()> {
        let cond = self.make_wired_pin(select.cond)?;
        let true_value = self.make_wired_pin(select.true_value)?;
        let false_value = self.make_wired_pin(select.false_value)?;
        let out = self.make_output_pin(select.lhs)?;
        let component = self
            .schematic
            .make_component(ComponentKind::Select(SelectComponent {
                cond,
                true_value,
                false_value,
                output: out,
            }));
        self.schematic.pin_mut(cond).parent(component);
        self.schematic.pin_mut(true_value).parent(component);
        self.schematic.pin_mut(false_value).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_index(&mut self, index: Index) -> Result<()> {
        let arg = self.make_wired_pin(index.arg)?;
        let out = self.make_output_pin(index.lhs)?;
        let dynamic = index
            .path
            .dynamic_slots()
            .map(|slot| self.make_wired_pin(*slot))
            .collect::<Result<Vec<_>>>()?;
        let component = self
            .schematic
            .make_component(ComponentKind::Index(IndexComponent {
                arg,
                path: index.path.clone(),
                output: out,
                dynamic: dynamic.clone(),
            }));
        self.schematic.pin_mut(arg).parent(component);
        self.schematic.pin_mut(out).parent(component);
        for pin in dynamic {
            self.schematic.pin_mut(pin).parent(component);
        }
        Ok(())
    }

    fn make_splice(&mut self, splice: Splice) -> Result<()> {
        let orig = self.make_wired_pin(splice.orig)?;
        let subst = self.make_wired_pin(splice.subst)?;
        let out = self.make_output_pin(splice.lhs)?;
        let dynamic = splice
            .path
            .dynamic_slots()
            .map(|slot| self.make_wired_pin(*slot))
            .collect::<Result<Vec<_>>>()?;
        let component = self
            .schematic
            .make_component(ComponentKind::Splice(SpliceComponent {
                orig,
                subst,
                output: out,
                path: splice.path.clone(),
                dynamic: dynamic.clone(),
            }));
        self.schematic.pin_mut(orig).parent(component);
        self.schematic.pin_mut(subst).parent(component);
        self.schematic.pin_mut(out).parent(component);
        for pin in dynamic {
            self.schematic.pin_mut(pin).parent(component);
        }
        Ok(())
    }

    fn make_repeat(&mut self, repeat: Repeat) -> Result<()> {
        let value = self.make_wired_pin(repeat.value)?;
        let out = self.make_output_pin(repeat.lhs)?;
        let component = self
            .schematic
            .make_component(ComponentKind::Repeat(RepeatComponent {
                value,
                output: out,
                len: repeat.len,
            }));
        self.schematic.pin_mut(value).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_struct(&mut self, structure: Struct) -> Result<()> {
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
        let component = self
            .schematic
            .make_component(ComponentKind::Struct(StructComponent {
                kind: structure.template.kind,
                fields: fields.clone(),
                output: out,
                rest,
            }));
        fields
            .iter()
            .for_each(|f| self.schematic.pin_mut(f.pin).parent(component));
        if let Some(pin) = rest {
            self.schematic.pin_mut(pin).parent(component);
        }
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_tuple(&mut self, tuple: Tuple) -> Result<()> {
        let fields = tuple
            .fields
            .into_iter()
            .map(|f| self.make_wired_pin(f))
            .collect::<Result<Vec<_>>>()?;
        let out = self.make_output_pin(tuple.lhs)?;
        let component = self
            .schematic
            .make_component(ComponentKind::Tuple(TupleComponent {
                fields: fields.clone(),
                output: out,
            }));
        fields
            .iter()
            .for_each(|f| self.schematic.pin_mut(*f).parent(component));
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_case(&mut self, case: Case) -> Result<()> {
        let discriminant = self.make_wired_pin(case.discriminant)?;
        let table = case
            .table
            .into_iter()
            .map(|(ndx, slot)| self.make_wired_pin(slot).map(|pin| (ndx, pin)))
            .collect::<Result<Vec<_>>>()?;
        let out = self.make_output_pin(case.lhs)?;
        let component = self
            .schematic
            .make_component(ComponentKind::Case(CaseComponent {
                discriminant,
                table: table.clone(),
                output: out,
            }));
        self.schematic.pin_mut(discriminant).parent(component);
        for (_, pin) in table {
            self.schematic.pin_mut(pin).parent(component);
        }
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_array(&mut self, array: Array) -> Result<()> {
        let elements = array
            .elements
            .into_iter()
            .map(|slot| self.make_wired_pin(slot))
            .collect::<Result<Vec<_>>>()?;
        let out = self.make_output_pin(array.lhs)?;
        let component = self
            .schematic
            .make_component(ComponentKind::Array(ArrayComponent {
                elements: elements.clone(),
                output: out,
            }));
        elements
            .iter()
            .for_each(|f| self.schematic.pin_mut(*f).parent(component));
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_discriminant(&mut self, discriminant: Discriminant) -> Result<()> {
        let arg = self.make_wired_pin(discriminant.arg)?;
        let out = self.make_output_pin(discriminant.lhs)?;
        let component =
            self.schematic
                .make_component(ComponentKind::Discriminant(DiscriminantComponent {
                    arg,
                    output: out,
                }));
        self.schematic.pin_mut(arg).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_enum(&mut self, enumerate: Enum) -> Result<()> {
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
        let component = self
            .schematic
            .make_component(ComponentKind::Enum(EnumComponent {
                fields: fields.clone(),
                output: out,
            }));
        fields
            .iter()
            .for_each(|f| self.schematic.pin_mut(f.pin).parent(component));
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_cast(&mut self, cast: Cast) -> Result<()> {
        let arg = self.make_wired_pin(cast.arg)?;
        let out = self.make_output_pin(cast.lhs)?;
        let component = self
            .schematic
            .make_component(ComponentKind::Cast(CastComponent {
                input: arg,
                output: out,
            }));
        self.schematic.pin_mut(arg).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_assign(&mut self, assign: Assign) -> Result<()> {
        let arg = self.make_wired_pin(assign.rhs)?;
        let out = self.make_output_pin(assign.lhs)?;
        let component = self
            .schematic
            .make_component(ComponentKind::Buffer(BufferComponent {
                input: arg,
                output: out,
            }));
        self.schematic.pin_mut(arg).parent(component);
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_exec(&mut self, exec: Exec) -> Result<()> {
        let code = &self.object.externals[exec.id.0].code;
        match code {
            ExternalFunctionCode::Kernel(kernel) => self.make_kernel(exec, kernel),
            ExternalFunctionCode::Extern(edef) => self.make_black_box(exec, edef),
        }
    }

    fn make_black_box(&mut self, exec: Exec, code: &ExternalKernelDef) -> Result<()> {
        let args = exec
            .args
            .iter()
            .map(|arg| self.make_wired_pin(*arg))
            .collect::<Result<Vec<_>>>()?;
        let out = self.make_output_pin(exec.lhs)?;
        let name = &self.object.externals[exec.id.0].path;
        let component = self
            .schematic
            .make_component(ComponentKind::BlackBox(BlackBoxComponent {
                name: name.clone(),
                args: args.clone(),
                output: out,
            }));
        args.iter()
            .for_each(|f| self.schematic.pin_mut(*f).parent(component));
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }

    fn make_kernel(&mut self, exec: Exec, kernel: &Kernel) -> Result<()> {
        let args = exec
            .args
            .iter()
            .map(|arg| self.make_wired_pin(*arg))
            .collect::<Result<Vec<_>>>()?;
        let out = self.make_output_pin(exec.lhs)?;
        let sub_schematic = build_schematic(self.module, kernel.inner().fn_id)?;
        let component = self
            .schematic
            .make_component(ComponentKind::Kernel(KernelComponent {
                name: kernel.inner().name.clone(),
                args: args.clone(),
                output: out,
                sub_schematic,
            }));
        args.iter()
            .for_each(|f| self.schematic.pin_mut(*f).parent(component));
        self.schematic.pin_mut(out).parent(component);
        Ok(())
    }
}
