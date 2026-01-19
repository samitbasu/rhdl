use rhdl::prelude::*;

pub mod step_1 {
    use super::*;

    // ANCHOR: step_1
    #[kernel]
    fn kernel_3(a: b6, b: b6) -> b6 {
        trace("a", &a);
        trace("b", &b);
        let mut c = a + b;
        for i in 0..4 {
            trace(("loop_index", i), &c);
            c += a;
        }
        let output = c;
        trace("output", &output);
        output
    }
    // ANCHOR_END: step_1

    // ANCHOR: step_1_test
    #[test]
    fn test_kernel_3() {
        let a_set = [0, 20, 15, 23, 42, 56];
        let b_set = [0, 3, 5, 7, 9, 11];
        let session = Session::default();
        let mut svg = SvgFile::default();
        for (ndx, (a, b)) in a_set.iter().zip(b_set).enumerate() {
            let traced = session.traced_at_time(ndx as u64 * 100, || {
                kernel_3(b6::from(*a), b6::from(b));
            });
            trace::container::TraceContainer::record(&mut svg, &traced).unwrap()
        }
        if !std::path::PathBuf::from("trace_keys.svg").exists() {
            let mut file = std::fs::File::create("trace_keys.svg").unwrap();
            svg.finalize(&SvgOptions::default(), &mut file).unwrap();
        }
    }
    // ANCHOR_END: step_1_test
}

pub mod step_2 {
    use super::*;

    // ANCHOR: step_2
    #[kernel]
    fn kernel_4(a: b6, b: b6) -> b6 {
        trace("a", &a);
        trace("b", &b);
        let c = a + b;
        trace(("kernel", "after_add"), &c);
        let output = c + c;
        trace("output", &output);
        output
    }
    // ANCHOR_END: step_2

    // ANCHOR: step_2_test
    #[test]
    fn test_kernel_4() {
        let a_set = [0, 20, 15, 23, 42, 56];
        let b_set = [0, 3, 5, 7, 9, 11];
        let session = Session::default();
        let mut svg = SvgFile::default();
        for (ndx, (a, b)) in a_set.iter().zip(b_set).enumerate() {
            let traced = session.traced_at_time(ndx as u64 * 100, || {
                kernel_4(b6::from(*a), b6::from(b));
            });
            trace::container::TraceContainer::record(&mut svg, &traced).unwrap()
        }
        if !std::path::PathBuf::from("trace_keys_2.svg").exists() {
            let mut file = std::fs::File::create("trace_keys_2.svg").unwrap();
            svg.finalize(&SvgOptions::default(), &mut file).unwrap();
        }
    }
    // ANCHOR_END: step_2_test
}
