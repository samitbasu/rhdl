// The MIG interface...
//
// Pins:
//   data_bus: InOut, 16,
//   address: Out, 15,
//   bank_select: Out, 3,
//   row_address_strobe_not: Out, 1
//   column_address_strobe_not: Out, 1
//   write_enable_not: Out, 1
//   on_die_termination: Out, 1
//   clock_enable: Out, 1
//   data_mask: Out, 2,
//   data_strobe_signal: InOut, 2
//   data_strobe_signal_neg: InOut, 2
//   dram_clock: Out, 1
//   dram_clock_neg: Out, 1
//   reset_not: Out, 1
//   raw_pos_clock: In, 1
//   raw_neg_clock: In, 1

// every pin has:
//   A location
//   A signal type
//   A set of timing constraints

// Create a raw interface (equivalent to a _sys thing)

/*
struct I {
    address: b29,
    command: b3,
    enable: bool,
    write_data_in: b128,
    write_data_end: bool,
    write_data_mask: b16,
    write_enable: bool,
    reset: bool,
}

struct O {
    reset_out: bool,
    clock: Clock,
    calib_done: bool,
    write_fifo_not_full: bool,
    ready: bool,
    read_data_out: b128,
    read_data_end: bool,
    read_data_valid: bool,
}
*/
// The top level needs to collect all pins from the module hierarchy
// The top module for export (to the synthesis tool) cannot have inputs or outputs
//
// Why the funky Module internal syntax?  I think it's to be language neutral...
pub mod drivers;
