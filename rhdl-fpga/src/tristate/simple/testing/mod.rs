use super::*;

use crate::tristate::simple::receiver;
use crate::tristate::simple::sender;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    pub snd: sender::U,
    pub rcv: receiver::U,
}

type I = Option<sender::Cmd>;
type O = Option<b8>;

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = fixture;
}

#[kernel]
pub fn fixture(_cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    d.rcv.bitz = q.snd.bitz;
    d.rcv.state = q.snd.control;
    d.snd.bitz = q.rcv;
    d.snd.cmd = i;
    (q.snd.data, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::tristate::simple::sender::Cmd;

    use super::*;
    use std::iter::{once, repeat};

    #[test]
    fn test_basic_trace() -> miette::Result<()> {
        let input = repeat(None)
            .take(2)
            .chain(once(Some(Cmd::Write(bits(0x15)))))
            .chain(repeat(None).take(2))
            .chain(once(Some(Cmd::Read)).chain(repeat(None).take(4)));
        let input = input.stream_after_reset(1).clock_pos_edge(100);
        let uut = super::U::default();
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("tristate");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["0f867cb3b4bbb022c836ff8ec7a4722a537256e93c7253902d3fa67bd7fe16ab"];
        let digest = vcd.dump_to_file(&root.join("basic.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_export() -> miette::Result<()> {
        type U = Adapter<sender::U, Red>;
        let uut = U::default();
        let i = <U as CircuitIO>::I::dont_care();
        let o = <U as CircuitIO>::O::dont_care();
        let binds = export![
            input cr => i.clock_reset,
            input bitz => i.input.val().bitz,
            input cmd => i.input.val().cmd,
            output bitz => o.val().bitz,
            output control => o.val().control,
            output data => o.val().data,
        ];
        let module = export_hdl_module(&uut, "tristate_sender", "TriState Sender Module", binds)?;
        std::fs::write("tristate.v", module.as_verilog()).unwrap();
        Ok(())
    }
}
