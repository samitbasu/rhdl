use rhdl_bits::Bits;
use rhdl_core::{
    compile_design,
    schematic::{
        self, builder::build_schematic, components::ComponentKind, schematic_impl::Schematic,
    },
    Digital, DigitalFn, KernelFnKind,
};
use rhdl_macro::Digital;

pub(crate) fn get_schematic<T: DigitalFn>() -> Schematic {
    let Some(KernelFnKind::Kernel(kernel)) = T::kernel_fn() else {
        panic!("Kernel function not found");
    };
    let module = compile_design(kernel).unwrap();
    build_schematic(&module, module.top).unwrap()
}

pub(crate) fn trace_reached_no_inputs(
    schematic: &Schematic,
    trace: &schematic::schematic_impl::Trace,
) -> bool {
    eprintln!("{:?}", trace);
    trace.sinks.iter().all(|sink| {
        matches!(
            schematic.component(schematic.pin(sink.pin).parent).kind,
            ComponentKind::Constant(_) | ComponentKind::Enum(_)
        )
    })
}

pub(crate) fn trace_reached_inputs_or_constant(
    schematic: &Schematic,
    trace: &schematic::schematic_impl::Trace,
) -> bool {
    eprintln!("{:?}", trace);
    trace.sinks.iter().all(|sink| {
        schematic.inputs.contains(&sink.pin)
            || matches!(
                schematic.component(schematic.pin(sink.pin).parent).kind,
                ComponentKind::Constant(_) | ComponentKind::Enum(_)
            )
    })
}

pub(crate) fn trace_reached_output(
    schematic: &Schematic,
    trace: &schematic::schematic_impl::Trace,
) -> bool {
    eprintln!("{:?}", trace);
    trace.sinks.iter().any(|sink| schematic.output == sink.pin)
}

#[derive(Copy, Clone, Debug, PartialEq, Digital)]
pub(crate) enum Bar {
    A(Bits<4>),
    B(Bits<4>),
    C(bool),
    D,
}

#[derive(Copy, Clone, Debug, PartialEq, Digital, Default)]
pub(crate) struct Egg {
    pub(crate) a: [Bits<4>; 2],
    pub(crate) b: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Digital, Default)]
pub(crate) struct Nested {
    pub(crate) a: [Bits<4>; 2],
    pub(crate) b: bool,
    pub(crate) c: [Egg; 2],
}

#[derive(Copy, Clone, Debug, PartialEq, Digital)]
pub(crate) struct Foo {
    pub(crate) a: Bits<4>,
    pub(crate) b: Bits<4>,
    pub(crate) c: bool,
    pub(crate) d: Bar,
    pub(crate) e: Nested,
}

impl Default for Foo {
    fn default() -> Self {
        Self {
            a: Default::default(),
            b: Default::default(),
            c: false,
            d: Bar::D,
            e: Nested::default(),
        }
    }
}
