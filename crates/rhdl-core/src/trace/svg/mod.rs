use std::sync::{Arc, RwLock};

use crate::{
    Digital, Kind, TypedBits,
    trace::{
        TraceContainer, TraceId,
        meta::TraceMetadata,
        svg::{
            bucket::bucketize,
            color::compute_trace_color_from_path,
            label::rewrite_trace_names_into_tree,
            layout::make_svg_document,
            options::SvgOptions,
            paths::{pretty_leaf_paths, try_path},
            waveform::{Region, Waveform},
        },
        trace_sample::TracedSample,
        trace_tree::TraceTree,
    },
    types::path::Path,
};

pub(crate) mod bucket;
pub(crate) mod color;
pub(crate) mod drawable;
pub(crate) mod gap;
pub(crate) mod label;
pub(crate) mod layout;
pub mod options;
pub(crate) mod paths;
pub(crate) mod waveform;

type TimeAndSample = (u64, TypedBits);

#[derive(Default)]
pub struct SvgFile {
    db: Option<Arc<RwLock<TraceMetadata>>>,
    inner: fnv::FnvHashMap<TraceId, Vec<TimeAndSample>>,
    times: Vec<u64>,
}

impl TraceContainer for SvgFile {
    fn record<T: Digital, S: Digital>(
        &mut self,
        sample: &TracedSample<T, S>,
    ) -> Result<(), crate::RHDLError> {
        if let Some(page) = sample.page.as_ref() {
            if self.db.is_none() {
                self.db = Some(page.details.clone());
            }
            let time = sample.time;
            self.times.push(time);
            for record in page.records() {
                let value = record.data.typed_bits();
                let id = record.trace_id;
                self.inner.entry(id).or_default().push((time, value));
            }
        }
        Ok(())
    }
}

impl<T: Digital, S: Digital> FromIterator<TracedSample<T, S>> for SvgFile {
    fn from_iter<I: IntoIterator<Item = TracedSample<T, S>>>(iter: I) -> Self {
        let mut svg = SvgFile::default();
        for sample in iter {
            svg.record(&sample)
                .expect("Failed to record sample into SVG");
        }
        svg
    }
}

impl SvgFile {
    fn build_time_trace(&self, trace_id: TraceId, kind: Kind, path: &Path) -> Box<[Region]> {
        let trace_color = compute_trace_color_from_path(kind, path).unwrap_or_default();
        let sliced = self
            .inner
            .get(&trace_id)
            .unwrap()
            .iter()
            .map(|(time, value)| (*time, try_path(value, path)));
        bucketize(sliced, trace_color)
            .iter()
            .map(|bucket| bucket.into())
            .collect()
    }
    fn trace_out(&self, name: &str, trace_id: TraceId, waves: &mut Vec<Waveform>) {
        let Some(db) = self.db.as_ref() else {
            return;
        };
        let db = db.read().unwrap();
        let Some(details) = db.get_details(trace_id) else {
            return;
        };
        let kind = details.kind;
        waves.extend(
            pretty_leaf_paths(kind, Path::default())
                .into_iter()
                .map(|path| {
                    let data = self.build_time_trace(trace_id, kind, &path);
                    Waveform {
                        label: format!("{name}{path:?}"),
                        hint: Default::default(),
                        data,
                    }
                }),
        )
    }
    fn write(&self, top: &str, tree: &TraceTree, waves: &mut Vec<Waveform>) {
        for (name, subtree) in &tree.children {
            self.write(name, subtree, waves);
        }
        for (name, trace_id) in &tree.signals {
            let name_sanitized = name.replace("::", "__");
            self.trace_out(&name_sanitized, *trace_id, waves);
        }
    }
    pub fn finalize(
        self,
        options: &SvgOptions,
        mut out: impl std::io::Write,
    ) -> std::io::Result<()> {
        let Some(db) = self.db.as_ref() else {
            return Ok(());
        };
        let trace_tree = db.read().unwrap().build_trace_tree();
        let mut waves = Vec::new();
        self.write("top", &trace_tree, &mut waves);
        let gaps = gap::segment_time(&self.times, options);
        rewrite_trace_names_into_tree(waves.as_mut_slice());
        let mut svg_waves = waves
            .into_iter()
            .filter(|w| {
                options
                    .name_filters
                    .as_ref()
                    .map(|f| f.is_match(&w.hint))
                    .unwrap_or(true)
            })
            .map(|w| w.render(options, &gaps))
            .collect::<Vec<_>>();
        let spacing = options.spacing();
        // Space the waveforms, and leave one space for the header
        for (i, wave) in svg_waves.iter_mut().enumerate() {
            wave.set_start_y((i + 1) as i32 * spacing);
        }
        let doc = make_svg_document(&svg_waves, &self.times, &gaps, options);
        svg::write(&mut out, &doc)?;
        Ok(())
    }
    pub fn to_string(self, options: &SvgOptions) -> std::io::Result<String> {
        let mut buf = Vec::new();
        self.finalize(options, &mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }
}
