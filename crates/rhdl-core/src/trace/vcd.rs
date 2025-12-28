//! A filter/inspector that writes trace pages to a VCD file.

use std::{
    io::Seek,
    io::Write,
    sync::{Arc, RwLock},
};

use tempfile::SpooledTempFile;
use vcd::IdCode;

use crate::{
    Digital, RHDLError, TraceBit,
    trace::{
        TraceContainer, TraceId, meta::TraceMetadata, trace_sample::TracedSample,
        trace_tree::TraceTree,
    },
};

pub struct Vcd {
    buffer: std::io::BufWriter<SpooledTempFile>,
    next_id_code: IdCode,
    id_code_map: fnv::FnvHashMap<TraceId, Box<[u8]>>,
    prev_values: fnv::FnvHashMap<TraceId, Box<[TraceBit]>>,
    db: Option<Arc<RwLock<TraceMetadata>>>,
}

impl Default for Vcd {
    fn default() -> Self {
        Vcd::new()
    }
}

impl<T: Digital, S: Digital> FromIterator<TracedSample<T, S>> for Vcd {
    fn from_iter<I: IntoIterator<Item = TracedSample<T, S>>>(iter: I) -> Self {
        let mut vcd = Vcd::new();
        for sample in iter {
            vcd.record(&sample)
                .expect("Failed to record sample into VCD");
        }
        vcd
    }
}

impl Vcd {
    pub fn new() -> Self {
        let buffer = std::io::BufWriter::new(SpooledTempFile::new(100 * 1024 * 1024));
        Self {
            buffer,
            next_id_code: IdCode::FIRST,
            id_code_map: fnv::FnvHashMap::default(),
            prev_values: fnv::FnvHashMap::default(),
            db: None,
        }
    }
    fn write_scope(
        &mut self,
        out: &mut impl std::io::Write,
        top: &str,
        tree: &TraceTree,
    ) -> std::io::Result<()> {
        out.write_fmt(format_args!("$scope module {} $end\n", top))?;
        for (name, subtree) in &tree.children {
            self.write_scope(out, name, subtree)?;
        }
        for (name, trace_id) in &tree.signals {
            let name_sanitized = name.replace("::", "__");
            let width = if let Some(db) = self.db.as_ref() {
                let details = db.read().unwrap();
                if let Some(trace_details) = details.get_details(*trace_id) {
                    trace_details.width
                } else {
                    1
                }
            } else {
                1
            };
            let id_code = &self.id_code(*trace_id);
            out.write_fmt(format_args!(
                "$var wire {} {} {} $end\n",
                width,
                std::str::from_utf8(id_code).unwrap(),
                name_sanitized
            ))?;
            self.next_id_code = self.next_id_code.next();
        }
        out.write_all(b"$upscope $end\n")?;
        Ok(())
    }
    fn write_id_code(&mut self, trace_id: TraceId) -> std::io::Result<()> {
        let id_code = self.id_code(trace_id);
        self.buffer.write_all(&id_code)
    }
    fn id_code(&mut self, trace_id: TraceId) -> Box<[u8]> {
        self.id_code_map
            .entry(trace_id)
            .or_insert_with(|| {
                let code = self.next_id_code;
                self.next_id_code = self.next_id_code.next();
                code.to_string().into_bytes().into_boxed_slice()
            })
            .clone()
    }
    pub fn finalize(mut self, mut out: impl std::io::Write) -> std::io::Result<()> {
        let Some(db) = self.db.as_ref() else {
            return Ok(());
        };
        self.buffer.flush()?;
        writeln!(out, "$timescale 1 ps $end")?;
        let rtt = db.read().unwrap().rtt();
        writeln!(out, "$comment")?;
        writeln!(out, "    {}", ron::ser::to_string(&rtt).unwrap())?;
        writeln!(out, "$end")?;
        let trace_tree = db.read().unwrap().build_trace_tree();
        self.write_scope(&mut out, "top", &trace_tree)?;
        writeln!(out, "$enddefinitions $end")?;
        let mut body = self.buffer.into_inner()?;
        body.seek(std::io::SeekFrom::Start(0))?;
        std::io::copy(&mut body, &mut out)?;
        Ok(())
    }
    pub fn dump_to_file(self, path: impl AsRef<std::path::Path>) -> std::io::Result<String> {
        use sha2::Digest;
        let mut file = std::fs::File::create(&path)?;
        self.finalize(&mut file)?;
        let mut file = std::fs::File::open(path)?;
        let mut hash = sha2::Sha256::default();
        std::io::copy(&mut file, &mut hash)?;
        Ok(format!("{:x}", hash.finalize()))
    }
    pub fn to_string(self) -> std::io::Result<String> {
        let mut buf = Vec::new();
        self.finalize(&mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }
}

impl TraceContainer for Vcd {
    fn record<T: Digital, S: Digital>(
        &mut self,
        sample: &TracedSample<T, S>,
    ) -> Result<(), RHDLError> {
        if let Some(page) = sample.page.as_ref() {
            if self.db.is_none() {
                self.db = Some(page.details.clone());
            }
            // Write the time stamp
            self.buffer.write_fmt(format_args!("#{}\n", sample.time))?;
            let mut sbuf = Vec::new();
            for record in page.records() {
                let value = record.data.trace();
                // Check to see if this value has changed since last time
                let changed = self
                    .prev_values
                    .get(&record.trace_id)
                    .map(|prev| *prev != value)
                    .unwrap_or(true);
                if !changed {
                    continue;
                }
                if value.is_empty() {
                    continue;
                }
                sbuf.clear();
                // Value has changed.  Write it out.  Get the VCD ID code for this trace ID.
                sbuf.push(b'b');
                sbuf.extend(value.iter().rev().map(|v| match v {
                    TraceBit::Zero => b'0',
                    TraceBit::One => b'1',
                    TraceBit::X => b'x',
                    TraceBit::Z => b'z',
                }));
                sbuf.push(b' ');
                self.buffer.write_all(&sbuf[..])?;
                self.write_id_code(record.trace_id)?;
                self.buffer.write_all(b"\n")?;
                // Update the previous values
                self.prev_values.insert(record.trace_id, value);
            }
        }
        Ok(())
    }
}
