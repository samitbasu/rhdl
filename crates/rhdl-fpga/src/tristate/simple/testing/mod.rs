use super::*;

use crate::tristate::simple::receiver;
use crate::tristate::simple::sender;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
#[rhdl(dq_no_prefix)]
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
    use expect_test::{expect, expect_file};
    use rhdl::prelude::vlog::Pretty as _;

    use crate::tristate::simple::sender::Cmd;

    use super::*;
    use std::iter::once;

    #[test]
    fn test_basic_trace() -> miette::Result<()> {
        let input = std::iter::repeat_n(None, 2)
            .chain(once(Some(Cmd::Write(bits(0x15)))))
            .chain(std::iter::repeat_n(None, 2))
            .chain(once(Some(Cmd::Read)).chain(std::iter::repeat_n(None, 4)));
        let input = input.with_reset(1).clock_pos_edge(100);
        let uut = super::U::default();
        let vcd = uut.run(input).collect::<VcdFile>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("tristate");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["acb8eb9739400be1ff525d61b66c6ec1b38162dbf43e4fb460ebe00748b7f75b"];
        let digest = vcd.dump_to_file(root.join("basic.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_export() -> miette::Result<()> {
        type U = Adapter<sender::U, Red>;
        let uut = U::default();
        let mut top = Fixture::new("top", uut);
        top.pass_through_input("cr", &path!(.clock_reset))?;
        top.pass_through_input("bitz", &path!(.input.val().bitz))?;
        top.pass_through_input("cmd", &path!(.input.val().cmd))?;
        top.pass_through_output("bitz", &path!(.val().bitz))?;
        top.pass_through_output("control", &path!(.val().control))?;
        top.pass_through_output("data", &path!(.val().data))?;
        let module = top.module()?;
        let expect = expect_file!["tristate.expect"];
        expect.assert_eq(&module.pretty());
        Ok(())
    }
}
