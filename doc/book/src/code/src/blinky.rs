pub mod step_1 {
    // ANCHOR: step-1
    #![allow(dead_code)]
    use rhdl::prelude::*;
    use rhdl_fpga::core::dff::DFF;

    pub struct Count8 {
        dff: DFF<b8>,
    }
    // ANCHOR_END: step-1
}

pub mod step_2 {
    // ANCHOR: step-2
    #![allow(dead_code)]
    use rhdl::prelude::*;
    use rhdl_fpga::core::dff::DFF;

    #[derive(Synchronous, SynchronousDQ)]
    pub struct Count8 {
        dff: DFF<b8>,
    }
    // ANCHOR_END: step-2

    impl SynchronousIO for Count8 {
        type I = ();
        type O = b8;
        type Kernel = NoSynchronousKernel<ClockReset, (), Count8Q, (b8, Count8D)>;
    }

    // ANCHOR: step-3
    #[derive(Digital, Clone, Copy, PartialEq)]
    pub struct I {
        enable: bool,
    }

    #[derive(Digital, Clone, Copy, PartialEq)]
    pub struct O {
        count: b8,
        overflow: bool,
    }
    // ANCHOR_END: step-3
}

pub mod step_3 {
    #![allow(dead_code)]
    pub mod counter {
        use rhdl::prelude::*;
        use rhdl_fpga::core::dff::DFF;

        #[derive(Synchronous, SynchronousDQ)]
        pub struct Count8 {
            dff: DFF<b8>,
        }

        #[derive(Digital, Clone, Copy, PartialEq)]
        pub struct I {
            enable: bool,
        }

        #[derive(Digital, Clone, Copy, PartialEq)]
        pub struct O {
            count: b8,
            overflow: bool,
        }

        impl SynchronousIO for Count8 {
            type I = I;
            type O = O;
            type Kernel = NoSynchronousKernel<ClockReset, I, Count8Q, (O, Count8D)>;
        }
    }
}
