use rhdl::prelude::*;
use rhdl_fpga::{
    audio::audio_pwm::{In, StereoAudioPwm},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    let uut = StereoAudioPwm::<4, 4>::new(bits(8));
    let mut stream_in: Vec<In<4>> = Vec::new();
    // Walk the duty values 0, 4, 8, 12, 15 on left; mirror right.
    let pattern = [0u128, 4, 8, 12, 15];
    for cycle in 0..80 {
        let mut inp = In {
            next_left: bits(0),
            next_right: bits(0),
            sample_valid: false,
        };
        // Provide a fresh sample every 16 cycles — host responds to sample_request
        // by presenting the next pattern element.
        let sample_idx = cycle / 16;
        if sample_idx < pattern.len() {
            inp.next_left = bits(pattern[sample_idx]);
            inp.next_right = bits(pattern[pattern.len() - 1 - sample_idx]);
            inp.sample_valid = true;
        }
        stream_in.push(inp);
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "audio_pwm.md", options)?;
    Ok(())
}
