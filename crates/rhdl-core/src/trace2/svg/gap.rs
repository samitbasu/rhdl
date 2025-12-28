//! Gap detection algorithm
use std::ops::RangeInclusive;

use crate::trace2::svg::options::{GapDetectionOptions, SvgOptions};

#[derive(Debug)]
pub struct GapList(Box<[RangeInclusive<u64>]>);

impl GapList {
    pub fn iter(&self) -> impl Iterator<Item = &RangeInclusive<u64>> {
        self.0.iter()
    }
    /// Given a time, calculates what (relative to 0) pixel value to use.
    /// This algorithm is O(|G|), and could probably be made faster.
    /// We also replace each gap interval with a fixed space of `gap_space`.
    pub fn gap_time(&self, time: u64, gap_space: u64) -> u64 {
        let mut eff_time = time;
        for gap in &self.0 {
            if time > *gap.start() {
                eff_time = eff_time.saturating_sub(*gap.end().min(&time) - gap.start());
            }
            if time >= *gap.end() {
                eff_time = eff_time.saturating_add(gap_space);
            }
        }
        eff_time
    }
    pub fn dropped_time(&self, time: u64) -> bool {
        self.0
            .iter()
            .any(|gap| gap.contains(&time) && time != *gap.end() && time != *gap.start())
    }
    pub fn is_gap_start(&self, time: u64) -> bool {
        self.0.iter().any(|gap| *gap.start() == time)
    }
    pub fn is_gap_end(&self, time: u64) -> bool {
        self.0.iter().any(|gap| *gap.end() == time)
    }
}

/// Given a list of time stamps, this function
/// scans them to determine if breaks are present using
/// the algorithm described in [SvgOptions].  It then
/// returns a list of contiguous segments to break
/// the time axis into.
pub fn segment_time(time: &[u64], options: &SvgOptions) -> GapList {
    let Some(method) = options.auto_gap_detection.as_ref() else {
        return GapList([].into());
    };
    match method {
        GapDetectionOptions::AtLeast(n) => manual_segmentation(time, *n),
        GapDetectionOptions::Median => {
            median_segmentation(time, u64::from(options.glitch_filter.unwrap_or(0)))
        }
    }
}

/// Use a manually specified gap time to identify breaks in the time axis.
fn manual_segmentation(time: &[u64], gap: u64) -> GapList {
    GapList(
        time.windows(2)
            .filter_map(|intervals| {
                let delta_t = intervals[1] - intervals[0];
                if delta_t > gap {
                    Some((intervals[0] + gap)..=(intervals[1]))
                } else {
                    None
                }
            })
            .collect(),
    )
}

/// Calculate the median delta_t, and use that as the segmentation gap
fn median_segmentation(time: &[u64], glitch_len: u64) -> GapList {
    let mut intervals = time
        .windows(2)
        .map(|x| x[1] - x[0])
        .filter(|x| *x > glitch_len)
        .collect::<Vec<_>>();
    intervals.sort();
    let median_time = intervals[intervals.len() / 2];
    manual_segmentation(time, median_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_selection() {
        let times: &[u64] = &[0, 1000, 2000, 3000, 4000, 5000, 10000, 11000, 12000];
        let gaps = segment_time(times, &SvgOptions::default().with_median_gap_detection());
        expect_test::expect![[r#"
            GapList(
                [
                    6000..=10000,
                ],
            )
        "#]]
        .assert_debug_eq(&gaps);
        let remapped_times = times
            .iter()
            .map(|t| gaps.gap_time(*t, 5))
            .collect::<Vec<_>>();
        expect_test::expect![[r#"
            [
                0,
                1000,
                2000,
                3000,
                4000,
                5000,
                6005,
                7005,
                8005,
            ]
        "#]]
        .assert_debug_eq(&remapped_times);
    }
}
