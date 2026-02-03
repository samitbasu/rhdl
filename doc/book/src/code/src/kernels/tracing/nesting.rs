use rhdl::prelude::*;

pub mod step_1 {
    use super::*;

    // ANCHOR: step_1
    #[kernel]
    fn sub_kernel(a: b6, b: b6) -> b6 {
        trace("a", &a);
        trace("b", &b);
        let c = a + b;
        trace("c", &c);
        c
    }

    #[kernel]
    fn kernel_5(arg0: b6, arg1: b6) -> b6 {
        trace("arg0", &arg0);
        trace("arg1", &arg1);
        let output = sub_kernel(arg0 + 1, arg1 + 1);
        trace("output", &output);
        output
    }
    // ANCHOR_END: step_1

    // ANCHOR: step_1_test
    #[test]
    fn test_kernel_5() {
        let a_set = [0, 20, 15, 23, 42, 56];
        let b_set = [0, 3, 5, 7, 9, 11];
        let session = Session::default();
        let mut svg = SvgFile::default();
        for (ndx, (a, b)) in a_set.iter().zip(b_set).enumerate() {
            let traced = session.traced_at_time(ndx as u64 * 100, || {
                kernel_5(b6::from(*a), b6::from(b));
            });
            trace::container::TraceContainer::record(&mut svg, &traced).unwrap()
        }
        if !std::path::PathBuf::from("trace_nesting.svg").exists() {
            let mut file = std::fs::File::create("trace_nesting.svg").unwrap();
            svg.finalize(&SvgOptions::default(), &mut file).unwrap();
        }
    }
    // ANCHOR_END: step_1_test
}
