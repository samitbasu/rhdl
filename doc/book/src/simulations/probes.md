# Probes

In RHDL, Probes are iterator adapters that can be used to process a stream of data from the _output_ of a simulation to extract certain data or events of interest.  

For example, when simulating a synchronous circuit, you may want to sample the output only on the rising or falling edge of the clock, and ignore all other samples in the time stream.  A Probe can be used to do this.

You may want to use probes to perform quality checking of the simulation output.  For example, in a synchronous design, the outputs should only change state on clock edges (to within a few time units as described in [Clock Pos Edge](../simulations/extensions/clock_pos_edge.md)).  A Probe can be used to check for glitches in the output that violate this property.

Finally, in a way similar to `.inspect` on iterators, probes can be used to tap into the output stream and log or record the output values for later analysis.  For example, a VCD (Value Change Dump) tap can be used to record the output values in a format that can be viewed with waveform viewers.

Each of these probes are simply convenient ways to continue using iterators in the context of RHDL simulation.  