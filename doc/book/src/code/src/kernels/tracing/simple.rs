use rhdl::prelude::*;

pub mod step_1 {
    use super::*;

    // ANCHOR: step_1
    #[kernel]
    fn kernel_1(a: b8, b: b8) -> b8 {
        trace("a", &a);
        trace("b", &b);
        let output = a + b;
        trace("output", &output);
        output
    }
    // ANCHOR_END: step_1

    // ANCHOR: step_1_test

    #[test]
    fn test_kernel() {
        let a_set = [0, 4, 15, 23, 42, 56];
        let b_set = [5, 13, 42, 64, 85, 127];
        let session = Session::default();
        let mut svg = SvgFile::default();
        for (ndx, (a, b)) in a_set.iter().zip(b_set).enumerate() {
            let traced = session.traced_at_time(ndx as u64 * 100, || {
                kernel_1(b8::from(*a), b8::from(b));
            });
            trace::container::TraceContainer::record(&mut svg, &traced).unwrap()
        }
        if !std::path::PathBuf::from("trace_simple.svg").exists() {
            let mut file = std::fs::File::create("trace_simple.svg").unwrap();
            svg.finalize(&SvgOptions::default(), &mut file).unwrap();
        }
    }

    // ANCHOR_END: step_1_test
}
