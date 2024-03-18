use crate::{path::Path, rhif::spec::Member};

pub(crate) fn path_with_member(path: Path, member: &Member) -> Path {
    match member {
        Member::Unnamed(ix) => path.index(*ix as usize),
        Member::Named(f) => path.field(f),
    }
}
