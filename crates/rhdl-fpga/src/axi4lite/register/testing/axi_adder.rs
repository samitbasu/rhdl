//! Simple example of an adder on an AXI bus.
//!
use crate::{
    axi4lite::{
        core::controller::blocking::{BlockReadWriteController, BlockRequest, BlockResponse},
        register::bank::AxiRegBank,
    },
    stream::Ready,
};
use rhdl::prelude::*;
#[derive(Clone, Synchronous, SynchronousDQ)]
pub struct Adder {
    controller: BlockReadWriteController,
    bank: AxiRegBank<3>,
}

impl Default for Adder {
    fn default() -> Self {
        Self {
            controller: BlockReadWriteController::default(),
            bank: AxiRegBank::new(bits(0), [bits(0); 3]),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Digital)]
pub struct In {
    pub cmd: Option<BlockRequest>,
}

#[derive(PartialEq, Clone, Copy, Digital)]
pub struct Out {
    pub reply: Option<BlockResponse>,
    pub cmd_ready: Ready<BlockRequest>,
}

impl SynchronousIO for Adder {
    type I = In;
    type O = Out;
    type Kernel = kernel;
}

#[kernel]
pub fn kernel(_cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    let mut o = Out::dont_care();
    d.controller.read_axi = q.bank.read_axi;
    d.controller.write_axi = q.bank.write_axi;
    d.bank.read_axi = q.controller.read_axi;
    d.bank.write_axi = q.controller.write_axi;
    let arg_a = q.bank.data[0];
    let arg_b = q.bank.data[1];
    d.bank.data = Some((bits(2), arg_a + arg_b));
    d.controller.request = i.cmd;
    d.controller.resp_ready.raw = true;
    o.cmd_ready = q.controller.req_ready;
    o.reply = q.controller.response;
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::{
        axi4lite::types::{StrobedData, WriteCommand},
        rng::xorshift::XorShift128,
    };

    use super::*;
    fn write_cmd(addr: u32, val: u32) -> BlockRequest {
        BlockRequest::Write(WriteCommand {
            addr: bits(addr as u128),
            strobed_data: StrobedData {
                data: bits(val as u128),
                strobe: bits(0b1111),
            },
        })
    }

    fn read_cmd() -> BlockRequest {
        BlockRequest::Read(bits(8))
    }

    fn test_stream(
        problems: impl Iterator<Item = (u32, u32)>,
    ) -> impl Iterator<Item = BlockRequest> {
        problems.flat_map(|p| [write_cmd(0, p.0), write_cmd(4, p.1), read_cmd()])
    }

    #[test]
    fn test_synthesizable() -> miette::Result<()> {
        let uut = Adder::default();
        let descriptor = uut.descriptor("top".into())?;
        let _ = descriptor.hdl()?;
        Ok(())
    }

    #[test]
    fn test_adder_trace() -> miette::Result<()> {
        let uut = Adder::default();
        let mut need_reset = true;
        let mut rng = XorShift128::default();
        let problems = (0..10).map(|_| (rng.next().unwrap(), rng.next().unwrap()));
        let mut seq = test_stream(problems);
        let mut tail = 0;
        let vcd = uut
            .run_fn(
                |o| {
                    if need_reset {
                        need_reset = false;
                        return Some(rhdl::core::sim::ResetOrData::Reset);
                    }
                    let mut input = In::dont_care();
                    input.cmd = None;
                    if o.cmd_ready.raw {
                        input.cmd = seq.next();
                        if input.cmd.is_none() && tail > 100 {
                            return None;
                        }
                        tail += 1;
                    }
                    Some(rhdl::core::sim::ResetOrData::Data(input))
                },
                100,
            )
            .collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("axi4lite")
            .join("bank");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["28ae5dbc2b711c69f6dff0359d982691a1f4d9ca7c8ef5be737a919cf998ba46"];
        let digest = vcd.dump_to_file(root.join("adder.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_bank_works() -> miette::Result<()> {
        let uut = Adder::default();
        let mut need_reset = true;
        let mut rng = XorShift128::default();
        let problems = (0..1000).map(|_| (rng.next().unwrap(), rng.next().unwrap()));
        let mut rng2 = XorShift128::default();
        let mut answers = (0..1000).map(|_| {
            let a = rng2.next().unwrap();
            let b = rng2.next().unwrap();
            a.wrapping_add(b)
        });
        let mut seq = test_stream(problems);
        let mut tail = 0;
        uut.run_fn(
            |o| {
                if need_reset {
                    need_reset = false;
                    return Some(rhdl::core::sim::ResetOrData::Reset);
                }
                let mut input = In::dont_care();
                input.cmd = None;
                if o.cmd_ready.raw {
                    input.cmd = seq.next();
                    if input.cmd.is_none() && tail > 100 {
                        return None;
                    }
                    tail += 1;
                }
                if let Some(BlockResponse::Read(Ok(data))) = o.reply {
                    let expected = answers.next().unwrap();
                    assert_eq!(expected, data.raw() as u32);
                }
                Some(rhdl::core::sim::ResetOrData::Data(input))
            },
            100,
        )
        .for_each(drop);
        Ok(())
    }
}
