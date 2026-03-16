#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct LabelId(usize);

impl LabelId {
    pub fn next(self) -> Self {
        LabelId(self.0 + 1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LabelSide {
    East,
    West,
}

#[derive(Clone)]
pub struct Label {
    pub text: String,
    pub side: LabelSide,
    pub offset: f32,
    pub id: LabelId,
}
