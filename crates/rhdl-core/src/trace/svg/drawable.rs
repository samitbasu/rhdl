use crate::trace::svg::color::TraceColor;

#[derive(Clone, Debug)]
pub(crate) struct Drawable {
    pub(crate) start_x: i32,
    pub(crate) start_y: i32,
    pub(crate) width: i32,
    pub(crate) tag: String,
    pub(crate) full_tag: String,
    pub(crate) kind: RegionKind,
    pub(crate) color: TraceColor,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum RegionKind {
    True,
    False,
    Multibit,
    Label,
}

/// A waveform that has been converted into SVG regions for rendering.
pub(crate) struct DrawableList(pub Box<[Drawable]>);

impl DrawableList {
    pub(crate) fn set_start_y(&mut self, start_y: i32) {
        for region in self.0.iter_mut() {
            region.start_y = start_y;
        }
    }
    pub(crate) fn width(&self) -> i32 {
        self.0
            .iter()
            .map(|r| r.start_x + r.width)
            .max()
            .unwrap_or(0)
    }
    pub(crate) fn height(&self, spacing: i32) -> i32 {
        self.0
            .iter()
            .map(|r| r.start_y + spacing)
            .max()
            .unwrap_or_default()
    }
}
