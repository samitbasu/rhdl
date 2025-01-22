use rhdl::{
    core::{hdl::ast::unary, rtl::spec::AluUnary},
    prelude::*,
};

#[derive(PartialEq, Debug, Clone, Default)]
pub struct U<C: Domain> {
    _c: std::marker::PhantomData<C>,
}

impl<C: Domain> CircuitDQ for U<C> {
    type D = ();
    type Q = ();
}

impl<C: Domain> CircuitIO for U<C> {
    type I = Signal<ResetN, C>;
    type O = Signal<Reset, C>;
    type Kernel = NoKernel2<Self::I, (), (Self::O, ())>;
}

impl<C: Domain> Circuit for U<C> {
    type S = ();

    fn init(&self) -> Self::S {}

    fn description(&self) -> String {
        format!(
            "Reset inversion (active low to active high) in domain {:?}",
            C::color()
        )
    }

    fn sim(&self, input: Self::I, _state: &mut Self::S) -> Self::O {
        let out = if input.val().raw() {
            reset(false)
        } else {
            reset(true)
        };
        signal(out)
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        let mut flow_graph = FlowGraph::default();
        let hdl = self.hdl(&format!("{name}_inner"))?;
        let (input, output) = flow_graph.circuit_black_box::<Self>(hdl);
        flow_graph.inputs = vec![input];
        flow_graph.output = output;
        Ok(CircuitDescriptor {
            unique_name: name.into(),
            flow_graph,
            input_kind: <Self::I as Timed>::static_kind(),
            output_kind: <Self::O as Timed>::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            rtl: None,
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = name.to_owned();
        let mut module = Module {
            name: module_name.clone(),
            ..Default::default()
        };
        module.ports = vec![
            port("i", Direction::Input, HDLKind::Wire, unsigned_width(1)),
            port("o", Direction::Output, HDLKind::Wire, unsigned_width(1)),
        ];
        module
            .statements
            .push(continuous_assignment("o", unary(AluUnary::Not, id("i"))));
        Ok(HDLDescriptor {
            name: name.into(),
            body: module,
            children: Default::default(),
        })
    }
}
