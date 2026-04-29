use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::half_spi_master::{HalfSpiMaster, In, FSM_TRANSITIONS},
};

fn main() -> Result<(), RHDLError> {
    // Emit the FSM diagram first — required by CLAUDE.md §12 rule 14.
    write_fsm_diagram_as_markdown::<HalfSpiMaster<8, 4>>(
        FSM_TRANSITIONS,
        "half_spi_master_fsm.md",
    )?;

    let uut = HalfSpiMaster::<8, 4>::default();
    // Simulate writing 0xA5 (8 bits), turnaround for 4 cycles, then reading
    // 8 bits with the slave returning 0x55.
    let n_cycles = 1 + 2 * 8 + 4 + 2 * 8 + 4;
    let mut stream_in: Vec<In<8, 4>> = Vec::with_capacity(n_cycles);
    for cycle in 0..n_cycles {
        let mut inp = In {
            tx_data: bits(0),
            write_bits: bits(0),
            read_bits: bits(0),
            turnaround: bits(0),
            sdio_in: false,
            start: false,
        };
        if cycle == 0 {
            inp.tx_data = bits(0xA5);
            inp.write_bits = bits(8);
            inp.read_bits = bits(8);
            inp.turnaround = bits(4);
            inp.start = true;
        }
        // Simulated slave: present each bit of 0x55 MSB-first during the read phase.
        let read_start = 1 + 2 * 8 + 4;
        if cycle >= read_start {
            let read_offset = cycle - read_start;
            let bit_idx = read_offset / 2;
            if bit_idx < 8 {
                let bit_pos = 7 - bit_idx;
                inp.sdio_in = ((0x55u128 >> bit_pos) & 1) != 0;
            }
        }
        stream_in.push(inp);
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "half_spi_master.md", options)?;
    Ok(())
}
