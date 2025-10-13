use rhdl_bits::{BitWidth, Bits};

use crate::{Digital, Kind, bitx::BitX, trace::bit::TraceBit};

use super::kind::Field;

#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub struct BitZ<const N: usize>
where
    rhdl_bits::W<N>: BitWidth,
{
    pub value: Bits<N>,
    pub mask: Bits<N>,
}

impl<const N: usize> Digital for BitZ<N>
where
    rhdl_bits::W<N>: BitWidth,
{
    const BITS: usize = 2 * N;
    const TRACE_BITS: usize = N;
    fn static_kind() -> Kind {
        Kind::make_struct(
            "BitZ",
            vec![
                Field {
                    name: "value".to_string().into(),
                    kind: <Bits<N> as Digital>::static_kind(),
                },
                Field {
                    name: "mask".to_string().into(),
                    kind: <Bits<N> as Digital>::static_kind(),
                },
            ]
            .into(),
        )
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rhdl_trace_type::TraceType::Bits(Self::TRACE_BITS)
    }
    fn bin(self) -> Box<[BitX]> {
        [self.value.bin(), self.mask.bin()].concat().into()
    }
    fn trace(self) -> Box<[TraceBit]> {
        self.value
            .bin()
            .into_iter()
            .zip(self.mask.bin())
            .map(|(v, m)| match (v, m) {
                (BitX::X, _) | (_, BitX::X) => TraceBit::X,
                (_, BitX::Zero) => TraceBit::Z,
                (BitX::Zero, BitX::One) => TraceBit::Zero,
                (BitX::One, BitX::One) => TraceBit::One,
            })
            .collect()
    }
    fn dont_care() -> Self {
        Self {
            value: Bits::dont_care(),
            mask: Bits::dont_care(),
        }
    }
}
