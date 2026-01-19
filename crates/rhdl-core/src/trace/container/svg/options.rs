//! Options for generating SVG traces

/// Gap detection options
pub enum GapDetectionOptions {
    /// Use the median clock period to determine if gaps are present
    Median,
    /// Use a manual threshold for the gap
    AtLeast(u64),
}

/// Options to control the SVG rendering
pub struct SvgOptions {
    /// Number of pixels per simulation time unit
    pub pixels_per_time_unit: f32,
    /// Font size in pixels
    pub font_size_in_pixels: f32,
    /// Vertical shim between traces
    pub shim: i32,
    /// Height of each trace region
    pub height: i32,
    /// Width of the label area
    pub label_width: Option<i32>,
    /// Minimum glitch filter duration
    pub glitch_filter: Option<u32>,
    /// Optional regex filter for trace names
    pub name_filters: Option<regex::Regex>,
    /// Set automatic gap detection
    pub auto_gap_detection: Option<GapDetectionOptions>,
    /// Amount of space to represent each gap
    pub gap_space: u64,
    /// Tail flush time (how long past the last event to continue drawing)
    pub tail_flush_time: u64,
}

// Generate an iterator that yields 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, etc.
//    This works by decomposing the number into
//  n       m    n / 3   n % 3
//  0       1      0       0
//  1       2      0       1
//  2       5      0       2
//  3      10      1       0
//  4      20      1       1
//  5      50      1       2
fn candidate_deltas() -> impl Iterator<Item = u64> {
    const SEQ: &[u64] = &[1, 2, 5];
    (0..).map(|n| {
        let m = n / 3;
        let n = n % 3;
        10u64.pow(m) * SEQ[n as usize]
    })
}

impl SvgOptions {
    /// Calculate the vertical spacing between traces   
    pub(crate) fn spacing(&self) -> i32 {
        self.height + self.shim * 2
    }
    /// Compute the candidate delta time for the current options
    pub(crate) fn select_time_delta(&self) -> u64 {
        // Need 10 characters * options.font_size_in_pixels < pixels_per_time_unit * time_delta
        candidate_deltas()
            .find(|x| (*x as f32 * self.pixels_per_time_unit) >= 10.0 * self.font_size_in_pixels)
            .unwrap()
    }
    /// Set the label width
    pub fn with_label_width(self, width: i32) -> Self {
        Self {
            label_width: Some(width),
            ..self
        }
    }
    /// Set a regex filter for trace names. Only names matching the regex will be included.
    pub fn with_filter(self, regex: &str) -> Self {
        Self {
            name_filters: Some(regex::Regex::new(regex).unwrap()),
            ..self
        }
    }
    /// Set a filter to include only top-level inputs and outputs, clock and reset
    pub fn with_io_filter(self) -> Self {
        Self {
            name_filters: Some(
                regex::Regex::new("(^top.input(.*))|(^top.outputs(.*))|(^top.reset)|(^top.clock)")
                    .unwrap(),
            ),
            ..self
        }
    }
    /// Enable automatic gap detection using the median clock period
    pub fn with_median_gap_detection(self) -> SvgOptions {
        Self {
            auto_gap_detection: Some(GapDetectionOptions::Median),
            ..self
        }
    }
}

impl Default for SvgOptions {
    fn default() -> Self {
        SvgOptions {
            pixels_per_time_unit: 1.0,
            font_size_in_pixels: 10.0,
            shim: 3,
            height: 14,
            label_width: None,
            glitch_filter: Some(2),
            name_filters: None,
            auto_gap_detection: None,
            gap_space: 100,
            tail_flush_time: 100,
        }
    }
}
