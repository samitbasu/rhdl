//! Given a [Kind], translate it into a RHDL Trace Type
use crate::types::kind;
use rhdl_trace_type as rtt;

impl From<crate::Color> for rtt::Color {
    fn from(color: crate::Color) -> Self {
        match color {
            crate::Color::Red => rtt::Color::Red,
            crate::Color::Orange => rtt::Color::Orange,
            crate::Color::Yellow => rtt::Color::Yellow,
            crate::Color::Green => rtt::Color::Green,
            crate::Color::Blue => rtt::Color::Blue,
            crate::Color::Indigo => rtt::Color::Indigo,
            crate::Color::Violet => rtt::Color::Violet,
        }
    }
}

impl From<kind::DiscriminantAlignment> for rtt::DiscriminantAlignment {
    fn from(da: kind::DiscriminantAlignment) -> Self {
        match da {
            kind::DiscriminantAlignment::Msb => rtt::DiscriminantAlignment::Msb,
            kind::DiscriminantAlignment::Lsb => rtt::DiscriminantAlignment::Lsb,
        }
    }
}

impl From<kind::DiscriminantType> for rtt::DiscriminantType {
    fn from(dt: kind::DiscriminantType) -> Self {
        match dt {
            kind::DiscriminantType::Unsigned => rtt::DiscriminantType::Unsigned,
            kind::DiscriminantType::Signed => rtt::DiscriminantType::Signed,
        }
    }
}

impl From<kind::DiscriminantLayout> for rtt::DiscriminantLayout {
    fn from(dl: kind::DiscriminantLayout) -> Self {
        rtt::DiscriminantLayout {
            width: dl.width,
            alignment: dl.alignment.into(),
            ty: dl.ty.into(),
        }
    }
}
