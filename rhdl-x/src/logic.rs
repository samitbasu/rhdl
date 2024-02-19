use rhdl_core::{CircuitDescriptor, Digital, DigitalFn, HDLDescriptor, HDLKind};

pub trait Kernel<I: Digital, O: Digital>: DigitalFn {
    const FUNC: fn(I) -> O;
}

pub trait Logic<I: Digital, O: Digital> {
    type S: Default + PartialEq + Clone;

    fn sim(&self, input: I, state: &mut Self::S) -> O;

    fn init_state(&self) -> Self::S {
        Default::default()
    }

    fn name(&self) -> &'static str;

    fn descriptor(&self) -> CircuitDescriptor;

    fn as_hdl(&self, kind: HDLKind) -> anyhow::Result<HDLDescriptor>;
}

pub trait LogicGroup {
    type I: Digital;
    type O: Digital;
}

pub struct Parallel<
    I: Digital,
    Kin: Kernel<I, G::I>,
    G: LogicGroup,
    Kout: Kernel<G::O, O>,
    O: Digital,
> {
    input: std::marker::PhantomData<I>,
    kernel_in: std::marker::PhantomData<Kin>,
    group: G,
    kernel_out: std::marker::PhantomData<Kout>,
    output: std::marker::PhantomData<O>,
}

pub struct Series<
    I: Digital,
    C0: Logic<I, O0>,
    O0: Digital,
    K: Kernel<O0, I1>,
    I1: Digital,
    C1: Logic<I1, O>,
    O: Digital,
> {
    input: std::marker::PhantomData<I>,
    logic0: C0,
    o0: std::marker::PhantomData<O0>,
    kernel: std::marker::PhantomData<K>,
    i1: std::marker::PhantomData<I1>,
    logic1: C1,
    output: std::marker::PhantomData<O>,
}

impl<
        I: Digital,
        C0: Logic<I, O0>,
        O0: Digital,
        K: Kernel<O0, I1>,
        I1: Digital,
        C1: Logic<I1, O>,
        O: Digital,
    > Logic<I, O> for Series<I, C0, O0, K, I1, C1, O>
{
    type S = (C0::S, C1::S);

    fn sim(&self, input: I, state: &mut Self::S) -> O {
        let (s0, s1) = state;
        let o0 = self.logic0.sim(input, s0);
        let i1 = K::FUNC(o0);
        self.logic1.sim(i1, s1)
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<K>()
    }

    fn descriptor(&self) -> CircuitDescriptor {
        let mut desc = self.logic0.descriptor();
        //desc.add_child("c0", &self.logic0);
        desc
    }

    fn as_hdl(&self, kind: HDLKind) -> anyhow::Result<HDLDescriptor> {
        todo!()
    }
}

pub struct Combinatorial<I: Digital, K: Kernel<I, O>, O: Digital> {
    input: std::marker::PhantomData<I>,
    kernel: std::marker::PhantomData<K>,
    output: std::marker::PhantomData<O>,
}

impl<I: Digital, K: Kernel<I, O>, O: Digital> Logic<I, O> for (I, K, O) {
    type S = ();

    fn sim(&self, input: I, state: &mut Self::S) -> O {
        K::FUNC(input)
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<K>()
    }

    fn descriptor(&self) -> CircuitDescriptor {
        CircuitDescriptor {
            unique_name: self.name().into(),
            input_kind: I::static_kind(),
            output_kind: O::static_kind(),
            num_tristate: 0,
            tristate_offset_in_parent: 0,
            children: Default::default(),
        }
    }

    fn as_hdl(&self, kind: HDLKind) -> anyhow::Result<HDLDescriptor> {
        todo!()
    }
}
