use ena::unify::UnifyKey;

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub(crate) struct EnaKey(u32);

impl UnifyKey for EnaKey {
    type Value = ();

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(u: u32) -> Self {
        Self(u)
    }

    fn tag() -> &'static str {
        "RegisterKey"
    }
}
