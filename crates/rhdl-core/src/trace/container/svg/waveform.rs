use crate::{
    BitX,
    trace::container::svg::{
        bucket::Bucket,
        color::TraceColor,
        drawable::{Drawable, DrawableList, RegionKind},
        gap::GapList,
        label::format_as_label,
        options::SvgOptions,
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
    pub(crate) fn split_at_gaps(self, gaps: &GapList) -> Waveform {
        let mut regions = Vec::new();
        for region in self.data.iter() {
            let intervals = gaps.break_interval_at_gaps(&(region.start..=region.end));
            regions.extend(intervals.into_iter().map(|interval| Region {
                start: *interval.start(),
                end: *interval.end(),
                tag: region.tag.clone(),
                kind: region.kind,
                color: region.color,
            }));
        }
        Waveform {
            label: self.label,
            hint: self.hint,
            data: regions.into_boxed_slice(),
        }
    }
    pub(crate) fn render(self, options: &SvgOptions, gaps: &GapList) -> DrawableList {
        //let label_width = options.font_size_in_pixels as i32 * options.label_width;
        let label_width = (self.label.len() as f32 * options.font_size_in_pixels) as i32;
        let label_region = Drawable {
            start_x: 0,
            start_y: 0,
            width: label_width,
            tag: self.label.clone(),
            full_tag: self.hint.clone(),
            kind: RegionKind::Label,
            color: TraceColor::MultiColor,
        };
        let data = self.data.iter().filter_map(|r| {
            if r.start > r.end {
                return None;
            }
            let len = r.end - r.start;
            if let Some(min_time) = options.glitch_filter
                && len < min_time as u64
            {
                return None;
            }
            // Remap the times based on the gap
            let r_start = gaps.gap_time(r.start, options.gap_space);
            let r_end = gaps.gap_time(r.end - 1, options.gap_space) + 1;
            let len = r_end.saturating_sub(r_start);
            let width = (len as f32 * options.pixels_per_time_unit) as i32;
            let start_x = label_width + (r_start as f32 * options.pixels_per_time_unit) as i32;
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
            Some(Drawable {
                start_x,
                start_y: 0,
                full_tag,
                width,
                tag,
                kind,
                color: r.color,
            })
        });
        DrawableList(std::iter::once(label_region).chain(data).collect())
    }
}
