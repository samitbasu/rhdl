#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]
use rhdl::prelude::*;
use rhdl_core::{
    trace::db::{trace_init_db, trace_time},
    trace2::{TraceContainer, session::Session, svg::SvgFile, vcd::VcdFile},
};

#[test]
fn test_svg_ng_enum() {
    use rhdl_core::trace2::page::trace;
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
    let t0 = TimedSample { time: 0, value: () };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::None);
    trace("color", &b8(0b10101010));
    svg.record(&guard.release());
    let t0 = TimedSample {
        time: 1_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::A(bits(42), bits(1024)));
    trace("color", &b8(0b10101010));
    svg.record(&guard.release());
    let t0 = TimedSample {
        time: 2_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::B { name: bits(67) });
    trace("color", &b8(0b10111010));
    svg.record(&guard.release());
    let t0 = TimedSample {
        time: 3_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::C(true));
    svg.record(&guard.release());
    let t0 = TimedSample {
        time: 4_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::C(false));
    svg.record(&guard.release());
    let t0 = TimedSample {
        time: 5_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::B { name: bits(65) });
    svg.record(&guard.release());
    let t0 = TimedSample {
        time: 6_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::A(bits(21), bits(512)));
    svg.record(&guard.release());
    let t0 = TimedSample {
        time: 7_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::None);
    svg.record(&guard.release());
    let t0 = TimedSample {
        time: 8_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::None);
    svg.record(&guard.release());
    let mut buf = vec![];
    svg.finalize(
        &rhdl_core::trace2::svg::options::SvgOptions::default(),
        &mut buf,
    )
    .unwrap();
    expect_test::expect_file!["expect/svg_ng_enum_svg.expect"]
        .assert_eq(&String::from_utf8(buf).unwrap());
}

#[test]
fn test_vcd_ng_enum() {
    use rhdl_core::trace2::page::trace;
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
    let mut vcd = VcdFile::default();
    let trace_session = Session::default();
    let t0 = TimedSample { time: 0, value: () };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::None);
    trace("color", &b8(0b10101010));
    vcd.record(&guard.release());
    let t0 = TimedSample {
        time: 1_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::A(bits(42), bits(1024)));
    trace("color", &b8(0b10101010));
    vcd.record(&guard.release());
    let t0 = TimedSample {
        time: 2_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::B { name: bits(67) });
    trace("color", &b8(0b10111010));
    vcd.record(&guard.release());
    let t0 = TimedSample {
        time: 3_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::C(true));
    vcd.record(&guard.release());
    let t0 = TimedSample {
        time: 4_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::C(false));
    vcd.record(&guard.release());
    let t0 = TimedSample {
        time: 5_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::B { name: bits(65) });
    vcd.record(&guard.release());
    let t0 = TimedSample {
        time: 6_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::A(bits(21), bits(512)));
    vcd.record(&guard.release());
    let t0 = TimedSample {
        time: 7_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::None);
    vcd.record(&guard.release());
    let t0 = TimedSample {
        time: 8_000,
        value: (),
    };
    let p0 = trace_session.traced(t0);
    let guard = p0.guard();
    trace("enum", &Enum::None);
    vcd.record(&guard.release());
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

    let guard = trace_init_db();
    trace_time(0);
    trace("enum", &Enum::None);
    trace("color", &b8(0b10101010));
    trace_time(1_000);
    trace("enum", &Enum::A(bits(42), bits(1024)));
    trace_time(2_000);
    trace("enum", &Enum::B { name: bits(67) });
    trace_time(3_000);
    trace("enum", &Enum::C(true));
    trace_time(4_000);
    trace("enum", &Enum::C(false));
    trace_time(5_000);
    trace("enum", &Enum::B { name: bits(65) });
    trace_time(6_000);
    trace("enum", &Enum::A(bits(21), bits(512)));
    trace_time(7_000);
    trace("enum", &Enum::None);
    trace_time(8_000);
    trace("enum", &Enum::None);
    let mut vcd = vec![];
    guard.take().dump_vcd(&mut vcd, None).unwrap();
    expect_test::expect_file!["expect/vcd_enum_vcd.expect"]
        .assert_eq(&String::from_utf8(vcd).unwrap());
}

#[test]
fn test_vcd_basic() {
    #[derive(PartialEq, Clone, Copy, Digital)]
    pub struct Simple {
        a: bool,
        b: Bits<8>,
    }

    let simple = Simple {
        a: true,
        b: Bits::from(0b10101010),
    };
    let guard = trace_init_db();
    trace_time(0);
    trace("simple", &simple);
    trace_time(1_000);
    let simple = Simple {
        a: false,
        b: Bits::from(0b01010101),
    };
    let mut snapshot = vec![];
    with_trace_db(|db| db.dump_vcd(&mut snapshot, None).unwrap());
    expect_test::expect_file!["expect/vcd_basic_snapshot.expect"]
        .assert_eq(&String::from_utf8(snapshot).unwrap());
    trace("simple", &simple);
    trace_time(2_000);
    trace("simple", &simple);
    let mut vcd = vec![];
    guard.take().dump_vcd(&mut vcd, None).unwrap();
    expect_test::expect_file!["expect/vcd_basic.expect"]
        .assert_eq(&String::from_utf8(vcd).unwrap());
}
