use rhdl::prelude::*;

use crate::axi4lite::channel::{ChannelMToS, ChannelSToM};

// This is a test harness that connects a random filler,
// a random drainer and an AXI channel into a single
// fixture.  It is easy to monitor the output - a single
// "full" bit that drops low if the channel ever yields
// an invalid value.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<const N: usize> {
    filler: crate::fifo::testing::filler::U<N>,
    sender: crate::axi4lite::channel::sender::U<Bits<N>>,
    delay_m_to_s: crate::core::delay::U<ChannelMToS<Bits<N>>, 4>,
    delay_s_to_m: crate::core::delay::U<ChannelSToM, 4>,
    receiver: crate::axi4lite::channel::receiver::U<Bits<N>>,
    drainer: crate::fifo::testing::drainer::U<N>,
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = ();
    type O = bool;
    type Kernel = fixture_kernel<N>;
}

#[kernel]
pub fn fixture_kernel<const N: usize>(_cr: ClockReset, _i: (), q: Q<N>) -> (bool, D<N>) {
    let mut d = D::<N>::init();
    // The filler needs access to the full signal of the sender
    d.filler = q.sender.full;
    // The sender input is connected to the filler output
    d.sender.to_send = q.filler;
    // The drainer is connected to the data output of the receiver
    d.drainer = q.receiver.data;
    // The advance signal of the sender comes from the drainer output
    d.receiver.next = q.drainer.next;
    // The delay lines are used to connect the sender and receiver
    d.delay_m_to_s = q.sender.bus;
    d.delay_s_to_m = q.receiver.bus;
    d.receiver.bus = q.delay_m_to_s;
    d.sender.bus = q.delay_s_to_m;
    (q.drainer.valid, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_trace() {
        let uut = U::<16>::default();
        let input = std::iter::repeat(())
            .take(1000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input).collect::<Vcd>();
        vcd.dump_to_file(&std::path::PathBuf::from("delay_channel.vcd"))
            .unwrap();
    }

    #[test]
    fn test_channel_is_valid() {
        let uut = U::<16>::default();
        let input = std::iter::repeat(())
            .take(100_000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let last = uut.run(input).last().unwrap();
        assert!(last.value.2);
    }
}
