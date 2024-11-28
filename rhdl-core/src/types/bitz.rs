use rhdl_bits::Bits;

use crate::{trace::bit::TraceBit, Digital, Kind};

use super::kind::Field;

#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub struct BitZ<const N: usize> {
    pub value: Bits<N>,
    pub mask: Bits<N>,
}

impl<const N: usize> Digital for BitZ<N> {
    const BITS: usize = 2 * N;
    const TRACE_BITS: usize = N;
    fn static_kind() -> Kind {
        Kind::make_struct(
            "BitZ",
            vec![
                Field {
                    name: "value".into(),
                    kind: <Bits<N> as Digital>::static_kind(),
                },
                Field {
                    name: "mask".into(),
                    kind: <Bits<N> as Digital>::static_kind(),
                },
            ],
        )
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rhdl_trace_type::TraceType::Bits(Self::TRACE_BITS)
    }
    fn bin(self) -> Vec<bool> {
        [self.value.bin().as_slice(), self.mask.bin().as_slice()].concat()
    }
    fn trace(self) -> Vec<TraceBit> {
        self.value
            .bin()
            .into_iter()
            .zip(self.mask.bin())
            .map(|(v, m)| match (v, m) {
                (_, false) => TraceBit::Z,
                (false, true) => TraceBit::Zero,
                (true, true) => TraceBit::One,
            })
            .collect()
    }
    fn maybe_init() -> Self {
        Self {
            value: Bits::maybe_init(),
            mask: Bits::maybe_init(),
        }
    }
}
