use rhdl::prelude::*;

pub mod step_1 {
    use super::*;

    // ANCHOR: step_1
    #[kernel]
    fn kernel_2(a: (b4, b4), b: b4) -> b4 {
        trace("a", &a);
        trace("b", &b);
        let output = a.0 + b;
        trace("output", &output);
        output
    }
    // ANCHOR_END: step_1

    // ANCHOR: step_1_test
    #[test]
    fn test_kernel_2() {
        let a_set = [(0, 1), (4, 2), (15, 3), (7, 4), (4, 5), (3, 6)];
        let b_set = [0, 3, 5, 7, 9, 11];
        let session = Session::default();
        let mut svg = SvgFile::default();
        for (ndx, (a, b)) in a_set.iter().zip(b_set).enumerate() {
            let traced = session.traced_at_time(ndx as u64 * 100, || {
                kernel_2((b4::from(a.0), b4::from(a.1)), b4::from(b));
            });
            trace::container::TraceContainer::record(&mut svg, &traced).unwrap()
        }
        let mut file = std::fs::File::create("trace_complex.svg").unwrap();
        svg.finalize(&SvgOptions::default(), &mut file).unwrap();
    }
    // ANCHOR_END: step_1_test
}
