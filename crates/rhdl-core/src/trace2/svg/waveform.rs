use crate::{
    BitX,
    trace2::svg::{
        bucket::Bucket,
        color::TraceColor,
        label::format_as_label,
        options::SvgOptions,
        region::{RegionKind, SvgRegion, SvgWaveform},
    },
};

/// A region of a waveform representing a contiguous time span
/// where the value is the same.  The value has been converted
/// into a label for easier rendering.  If it is a binary value,
/// the kind indicates whether it is a 'True' or 'False' region.
#[derive(Clone, Debug)]
pub(crate) struct Region {
    start: u64,
    end: u64,
    tag: Option<String>,
    kind: RegionKind,
    color: TraceColor,
}

impl From<&Bucket> for Region {
    fn from(bucket: &Bucket) -> Self {
        let kind = match bucket.data.len() {
            1 => match bucket.data.bits()[0] {
                BitX::Zero => RegionKind::False,
                BitX::One => RegionKind::True,
                _ => RegionKind::Multibit,
            },
            _ => RegionKind::Multibit,
        };
        Self {
            start: bucket.start,
            end: bucket.end,
            tag: format_as_label(&bucket.data),
            kind,
            color: bucket.color,
        }
    }
}

/// A waveform to be rendered in the SVG output.
/// Consists of a label for the waveform, a hint (tooltip containing
/// additional information), and the actual data regions.  Each
/// data region represents a contiguous time span where the value
/// is the same.
pub(crate) struct Waveform {
    pub label: String,
    pub hint: String,
    pub data: Box<[Region]>,
}

impl Waveform {
    pub(crate) fn render(self, options: &SvgOptions) -> SvgWaveform {
        let data = self
            .data
            .iter()
            .filter_map(|r| {
                if r.start > r.end {
                    return None;
                }
                let len = r.end - r.start;
                if let Some(min_time) = options.glitch_filter
                    && len < min_time as u64
                {
                    return None;
                }
                let width = ((r.end - r.start) as f32 * options.pixels_per_time_unit) as i32;
                let start_x = (r.start as f32 * options.pixels_per_time_unit) as i32;
                let kind = r.kind;
                let width_in_characters = (width as f32 / options.font_size_in_pixels) as usize;
                let mut tag = r.tag.clone().unwrap_or_default();
                let full_tag = tag.clone();
                if tag.len() > width_in_characters {
                    while !tag.is_empty() && tag.len() + 3 > width_in_characters {
                        tag.pop();
                    }
                    if !tag.is_empty() {
                        tag += "...";
                    }
                }
                Some(SvgRegion {
                    start_x,
                    start_y: 0,
                    full_tag,
                    width,
                    tag,
                    kind,
                    color: r.color,
                })
            })
            .collect();
        SvgWaveform {
            label: self.label,
            hint: self.hint,
            data,
        }
    }
}
