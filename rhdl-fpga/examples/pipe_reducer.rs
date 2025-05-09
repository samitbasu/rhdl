use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    gearbox::pipe_reducer::{In, PipeReducer},
    rng::xorshift::XorShift128,
};

fn mk_array<T, const N: usize>(t: &mut impl Iterator<Item = T>) -> [T; N]
where
    [T; N]: Default,
{
    let mut ret = <[T; N] as Default>::default();
    (0..N).for_each(|ndx| {
        ret[ndx] = t.next().unwrap();
    });
    ret
}

fn main() -> Result<(), RHDLError> {
    type Uut = PipeReducer<U2, b4, 4>;
    let uut = Uut::default();
    let mut need_reset = true;
    let mut source_rng = XorShift128::default().map(|x| bits((x & 0xF) as u128));
    let mut dest_rng = source_rng.clone();
    let mut latched_input: Option<[b4; 4]> = None;
    let vcd = uut
        .run_fn(
            move |out| {
                if need_reset {
                    need_reset = false;
                    return Some(rhdl::core::sim::ResetOrData::Reset);
                }
                let mut input = In::<b4, 4>::dont_care();
                // Downstream is likely to run
                let want_to_pause = rand::random::<u8>() > 200;
                input.ready = !want_to_pause;
                // Decide if the producer will generate a new data item
                let willing_to_send = rand::random::<u8>() < 200;
                if out.ready {
                    // The pipeline wants more data
                    if willing_to_send {
                        latched_input = Some(mk_array(&mut source_rng));
                    } else {
                        latched_input = None;
                    }
                }
                input.data = latched_input;
                if input.ready && out.data.is_some() {
                    assert_eq!(dest_rng.next(), out.data);
                }
                Some(rhdl::core::sim::ResetOrData::Data(input))
            },
            100,
        )
        .take_while(|t| t.time < 1_500)
        .collect::<Vcd>();
    let options = SvgOptions::default().with_io_filter();
    write_svg_as_markdown(vcd, "pipe_reducer.md", options)?;
    Ok(())
}
