// Create a test fixture with a single register
// and a read and write controller for it

use crate::{
    axi4lite::{
        core::controller::blocking::{BlockReadWriteController, BlockRequest, BlockResponse},
        register::single::AxiRegister,
    },
    stream::Ready,
};
use rhdl::prelude::*;

#[derive(Clone, Synchronous, SynchronousDQ)]
pub struct TestFixture {
    controller: BlockReadWriteController,
    register: AxiRegister,
}

impl Default for TestFixture {
    fn default() -> Self {
        Self {
            controller: BlockReadWriteController::default(),
            register: AxiRegister::new(bits(0), bits(0)),
        }
    }
}

#[derive(PartialEq, Digital)]
pub struct In {
    pub cmd: Option<BlockRequest>,
}

#[derive(PartialEq, Digital)]
pub struct Out {
    pub reply: Option<BlockResponse>,
    pub cmd_ready: Ready<BlockRequest>,
}

impl SynchronousIO for TestFixture {
    type I = In;
    type O = Out;
    type Kernel = kernel;
}

#[kernel]
pub fn kernel(_cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    let mut o = Out::dont_care();
    d.controller.read_axi = q.register.read_axi;
    d.controller.write_axi = q.register.write_axi;
    d.register.read_axi = q.controller.read_axi;
    d.register.write_axi = q.controller.write_axi;
    d.register.data = None;
    d.controller.request = i.cmd;
    d.controller.resp_ready.raw = true;
    o.cmd_ready = q.controller.req_ready;
    o.reply = q.controller.response;
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use rhdl::core::sim::ResetOrData;

    use crate::axi4lite::types::{AXI4Error, StrobedData, WriteCommand};

    use super::*;

    fn write_cmd(strobe: u8, val: u32) -> BlockRequest {
        BlockRequest::Write(WriteCommand {
            addr: bits(0),
            strobed_data: StrobedData {
                data: bits(val as u128),
                strobe: bits(strobe as u128),
            },
        })
    }

    fn read_cmd() -> BlockRequest {
        BlockRequest::Read(bits(0))
    }

    fn test_stream() -> impl Iterator<Item = BlockRequest> {
        [
            write_cmd(0b1111, 42),
            read_cmd(),
            write_cmd(0b1111, 43),
            read_cmd(),
            write_cmd(0b1111, 45),
            write_cmd(0b1111, 42),
            read_cmd(),
            BlockRequest::Read(bits(4)),
            // Write DEADBEEF as 4 strobed writes
            write_cmd(0b0001, 0xAA55_AAEF),
            read_cmd(),
            write_cmd(0b0010, 0xAA55_BEAA),
            read_cmd(),
            write_cmd(0b0100, 0xAAAD_55AA),
            read_cmd(),
            write_cmd(0b1000, 0xDE55_AA55),
            read_cmd(),
        ]
        .into_iter()
    }

    #[test]
    fn test_synthesizable() -> miette::Result<()> {
        let uut = TestFixture::default();
        let _ = uut.hdl("top")?;
        Ok(())
    }

    #[test]
    fn test_register_trace() -> miette::Result<()> {
        let uut = TestFixture::default();
        let mut need_reset = true;
        let mut seq = test_stream().fuse();
        let mut tail = 0;
        let vcd = uut
            .run_fn(
                |o| {
                    if need_reset {
                        need_reset = false;
                        return Some(ResetOrData::Reset);
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
                    Some(ResetOrData::Data(input))
                },
                100,
            )
            .collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("axi4lite")
            .join("register");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["efe5aeb3b5dff66ea80200f244c8a5141727504545e60b9cb3843cadb5e70651"];
        let digest = vcd.dump_to_file(root.join("register.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_register_works() -> miette::Result<()> {
        let uut = TestFixture::default();
        let mut need_reset = true;
        let mut seq = test_stream().fuse();
        let mut tail = 0;
        let io = uut
            .run_fn(
                |o| {
                    if need_reset {
                        need_reset = false;
                        return Some(ResetOrData::Reset);
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
                    Some(ResetOrData::Data(input))
                },
                100,
            )
            .synchronous_sample();
        let io = io
            .filter_map(|x| x.value.2.reply)
            .filter_map(|x| match x {
                BlockResponse::Read(read) => Some(read),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            io,
            vec![
                Ok(bits(42)),
                Ok(bits(43)),
                Ok(bits(42)),
                Err(AXI4Error::DECERR),
                Ok(bits(0x00_00_00_EF)),
                Ok(bits(0x00_00_BE_EF)),
                Ok(bits(0x00_AD_BE_EF)),
                Ok(bits(0xDE_AD_BE_EF)),
            ]
        );
        Ok(())
    }
}
