#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]
use rhdl::prelude::*;
use rhdl_core::trace::{TraceContainer, session::Session, svg::SvgFile, vcd::Vcd};

#[test]
fn test_svg_ng_enum_discontinuous() {
    use rhdl_core::trace::page::trace;
    #[derive(PartialEq, Debug, Digital, Default, Clone, Copy)]
    enum Enum {
        #[default]
        None,
        A(b8, b16),
        B {
            name: b8,
        },
        C(bool),
    }
    let mut svg = SvgFile::default();
    let trace_session = Session::default();
    let sample = trace_session.traced_at_time(0, || {
        trace("enum", &Enum::None);
        trace("color", &b8(0b10101010));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(1_000, || {
        trace("enum", &Enum::A(bits(42), bits(1024)));
        trace("color", &b8(0b10101010));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(2_000, || {
        trace("enum", &Enum::B { name: bits(67) });
        trace("color", &b8(0b10111010));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(3_000, || {
        trace("enum", &Enum::C(true));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(4_000, || {
        trace("enum", &Enum::C(false));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(5_000, || {
        trace("enum", &Enum::B { name: bits(65) });
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(10_000, || {
        trace("enum", &Enum::A(bits(21), bits(512)));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(11_000, || {
        trace("enum", &Enum::None);
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(12_000, || {
        trace("enum", &Enum::None);
    });
    svg.record(&sample);
    let mut buf = vec![];
    svg.finalize(
        &rhdl_core::trace::svg::options::SvgOptions {
            pixels_per_time_unit: 0.1,
            ..Default::default()
        }
        .with_median_gap_detection(),
        &mut buf,
    )
    .unwrap();
    expect_test::expect_file!["expect/svg_ng_enum_discontinuous_median_gap_svg.expect"]
        .assert_eq(&String::from_utf8(buf).unwrap());
}

#[test]
fn test_svg_ng_enum() {
    use rhdl_core::trace::page::trace;
    #[derive(PartialEq, Debug, Digital, Default, Clone, Copy)]
    enum Enum {
        #[default]
        None,
        A(b8, b16),
        B {
            name: b8,
        },
        C(bool),
    }
    let mut svg = SvgFile::default();
    let trace_session = Session::default();
    let sample = trace_session.traced_at_time(0, || {
        trace("enum", &Enum::None);
        trace("color", &b8(0b10101010));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(1_000, || {
        trace("enum", &Enum::A(bits(42), bits(1024)));
        trace("color", &b8(0b10101010));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(2_000, || {
        trace("enum", &Enum::B { name: bits(67) });
        trace("color", &b8(0b10111010));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(3_000, || {
        trace("enum", &Enum::C(true));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(4_000, || {
        trace("enum", &Enum::C(false));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(5_000, || {
        trace("enum", &Enum::B { name: bits(65) });
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(6_000, || {
        trace("enum", &Enum::A(bits(21), bits(512)));
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(7_000, || {
        trace("enum", &Enum::None);
    });
    svg.record(&sample);
    let sample = trace_session.traced_at_time(8_000, || {
        trace("enum", &Enum::None);
    });
    svg.record(&sample);
    let mut buf = vec![];
    svg.finalize(
        &rhdl_core::trace::svg::options::SvgOptions {
            pixels_per_time_unit: 0.1,
            ..Default::default()
        },
        &mut buf,
    )
    .unwrap();
    expect_test::expect_file!["expect/svg_ng_enum_svg.expect"]
        .assert_eq(&String::from_utf8(buf).unwrap());
}

#[test]
fn test_vcd_ng_enum() {
    use rhdl_core::trace::page::trace;
    #[derive(PartialEq, Debug, Digital, Default, Clone, Copy)]
    enum Enum {
        #[default]
        None,
        A(b8, b16),
        B {
            name: b8,
        },
        C(bool),
    }
    let mut vcd = Vcd::default();
    let trace_session = Session::default();
    let sample = trace_session.traced_at_time(0, || {
        trace("enum", &Enum::None);
        trace("color", &b8(0b10101010));
    });
    vcd.record(&sample);
    let sample = trace_session.traced_at_time(1_000, || {
        trace("enum", &Enum::A(bits(42), bits(1024)));
        trace("color", &b8(0b10101010));
    });
    vcd.record(&sample);
    let sample = trace_session.traced_at_time(2_000, || {
        trace("enum", &Enum::B { name: bits(67) });
        trace("color", &b8(0b10111010));
    });
    vcd.record(&sample);
    let sample = trace_session.traced_at_time(3_000, || {
        trace("enum", &Enum::C(true));
    });
    vcd.record(&sample);
    let sample = trace_session.traced_at_time(4_000, || {
        trace("enum", &Enum::C(false));
    });
    vcd.record(&sample);
    let sample = trace_session.traced_at_time(5_000, || {
        trace("enum", &Enum::B { name: bits(65) });
    });
    vcd.record(&sample);
    let sample = trace_session.traced_at_time(10_000, || {
        trace("enum", &Enum::A(bits(21), bits(512)));
    });
    vcd.record(&sample);
    let sample = trace_session.traced_at_time(11_000, || {
        trace("enum", &Enum::None);
    });
    vcd.record(&sample);
    let sample = trace_session.traced_at_time(12_000, || {
        trace("enum", &Enum::None);
    });
    vcd.record(&sample);
    let mut buf = vec![];
    vcd.finalize(&mut buf).unwrap();
    expect_test::expect_file!["expect/vcd_ng_enum_vcd.expect"]
        .assert_eq(&String::from_utf8(buf).unwrap());
}

#[test]
fn test_vcd_enum() {
    #[derive(PartialEq, Debug, Digital, Default, Clone, Copy)]
    enum Enum {
        #[default]
        None,
        A(b8, b16),
        B {
            name: b8,
        },
        C(bool),
    }
    let session = Session::default();
    let mut vcd = Vcd::default();
    let sample = session.traced_at_time(0, || {
        trace("enum", &Enum::None);
        trace("color", &b8(0b10101010));
    });
    vcd.record(&sample).unwrap();
    let sample = session.traced_at_time(1_000, || {
        trace("enum", &Enum::A(bits(42), bits(1024)));
        trace("color", &b8(0b10101010));
    });
    vcd.record(&sample).unwrap();
    let sample = session.traced_at_time(2_000, || {
        trace("enum", &Enum::B { name: bits(67) });
        trace("color", &b8(0b10111010));
    });
    vcd.record(&sample).unwrap();
    let sample = session.traced_at_time(3_000, || {
        trace("enum", &Enum::C(true));
    });
    vcd.record(&sample).unwrap();
    let sample = session.traced_at_time(4_000, || {
        trace("enum", &Enum::C(false));
    });
    vcd.record(&sample).unwrap();
    let sample = session.traced_at_time(5_000, || {
        trace("enum", &Enum::B { name: bits(65) });
    });
    vcd.record(&sample).unwrap();
    let sample = session.traced_at_time(6_000, || {
        trace("enum", &Enum::A(bits(21), bits(512)));
    });
    vcd.record(&sample).unwrap();
    let sample = session.traced_at_time(7_000, || {
        trace("enum", &Enum::None);
    });
    vcd.record(&sample).unwrap();
    let sample = session.traced_at_time(8_000, || {
        trace("enum", &Enum::None);
    });
    vcd.record(&sample).unwrap();
    let mut buf = vec![];
    vcd.finalize(&mut buf).unwrap();
    expect_test::expect_file!["expect/vcd_enum_vcd.expect"]
        .assert_eq(&String::from_utf8(buf).unwrap());
}
