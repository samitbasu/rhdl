//! A single page of the trace data, i.e. all values
//! logged at a single instance in (simulated) time.

use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};

use crate::{
    Digital, TraceKey,
    trace2::{
        TraceId,
        meta::{TraceDetails, TraceMetadata},
        record::Record,
    },
};

/// A single page of the trace data, i.e. all values
/// logged at a single instance in (simulated) time.
pub struct TracePage {
    /// The meta data about the trace page.  Indicates
    /// which values are recorded where in the data.
    records: Vec<Record>,
    /// The current path active on the page.  Used to
    /// track changes in scope during execution.
    path: Vec<&'static str>,
    /// The shared details about the trace.  We hold these
    /// in a RwLock to allow sharing of the details across
    /// threads and across pages.
    pub(crate) details: Arc<RwLock<TraceMetadata>>,
}

impl std::fmt::Display for TracePage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TracePage:")?;
        for record in &self.records {
            let details = self.details.read().unwrap();
            if let Some(trace_details) = details.get_details(record.trace_id) {
                writeln!(
                    f,
                    "  {}/{} ({} bits)",
                    trace_details.path.join("/"),
                    trace_details.key,
                    record.data.bits()
                )?;
            } else {
                writeln!(f, "  Unknown trace ID {:?}", record.trace_id)?;
            }
        }
        Ok(())
    }
}

impl TracePage {
    /// Create a new empty trace page that references
    /// the given trace details DB
    pub fn new(details: Arc<RwLock<TraceMetadata>>) -> Self {
        TracePage {
            records: Vec::new(),
            path: Vec::new(),
            details,
        }
    }
    /// Reset the trace page to empty state.
    pub fn reset(&mut self) {
        self.records.clear();
        self.path.clear();
    }
    // Hash a key to get the ID for tracing.
    fn key_hash(&self, key: &impl TraceKey) -> TraceId {
        let mut hasher = fnv::FnvHasher::default();
        let key = (&self.path[..], key);
        key.hash(&mut hasher);
        TraceId(hasher.finish())
    }
    /// Record a traced value onto the page.  The key is combined
    /// with the current path to produce a unique trace ID.
    pub fn trace<T: Digital>(&mut self, key: impl TraceKey, value: &T) {
        let trace_id = self.key_hash(&key);
        // Check if we have already recorded this key.
        let has_key = self.details.read().unwrap().has_key(trace_id);
        if !has_key {
            // populate it.  This is expensive, but done rarely.
            let details = TraceDetails {
                trace_id,
                trace_type: value.trace_type(),
                path: self.path.clone(),
                key: key.as_string().to_string(),
                width: (T::TRACE_BITS as u32).max(1) as usize,
                kind: value.kind(),
            };
            self.details.write().unwrap().insert(trace_id, details);
        }
        self.records.push(Record {
            trace_id,
            data: Box::new(*value),
        });
    }
    pub fn records(&self) -> impl Iterator<Item = &Record> {
        self.records.iter()
    }
}

thread_local! {
    static PAGE: RefCell<Option<Box<TracePage>>> = const {RefCell::new(None)};
}

/// Set the current trace page for tracing in this thread
pub fn set_trace_page(page: Option<Box<TracePage>>) {
    PAGE.replace(page);
}

pub fn take_trace_page() -> Option<Box<TracePage>> {
    PAGE.take()
}

/// Push a name onto the current trace path
pub fn trace_push_path(name: &'static str) {
    PAGE.with_borrow_mut(|page_cell| {
        if let Some(page) = page_cell.as_mut() {
            page.path.push(name);
        }
    });
}

/// Pop a name from the current trace path
pub fn trace_pop_path() {
    PAGE.with_borrow_mut(|page_cell| {
        if let Some(page) = page_cell.as_mut() {
            page.path.pop();
        }
    });
}

/// Trace a signal value at the current time
pub fn trace(key: impl TraceKey, value: &impl Digital) {
    PAGE.with_borrow_mut(|page_cell| {
        if let Some(page) = page_cell.as_mut() {
            page.trace(key, value);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhdl_bits::alias::*;

    #[test]
    fn test_trace_page() {
        // Add test cases here
        let db = Arc::new(RwLock::new(TraceMetadata::default()));
        // Set the current trace page.
        PAGE.replace(Some(Box::new(TracePage::new(db.clone()))));
        trace_push_path("fn1");
        trace_push_path("fn2");
        trace("a", &true);
        trace_pop_path();
        trace("a", &b6(0x15));
        trace_pop_path();
        let page = PAGE.take();
        let page = page.expect("Trace page should be set");
        let expect = expect_test::expect![[r#"
            TracePage:
              fn1/fn2/a (1 bits)
              fn1/a (6 bits)
        "#]];
        expect.assert_eq(&format!("{}", page));
    }

    #[test]
    fn test_trace_page_benchmark_crude() {
        let db = Arc::new(RwLock::new(TraceMetadata::default()));
        let mut page_count = 0;
        let tic = std::time::Instant::now();
        let mut serialized_bits = 0;
        for i in 0..100_000 {
            PAGE.replace(Some(Box::new(TracePage::new(db.clone()))));
            // Set the current trace page.
            trace("sig1", &b32(i));
            trace("sig2", &b1((i % 2 == 0) as u128));
            trace_push_path("inner");
            trace("sig3", &b8(i * 3 % 256));
            trace_pop_path();
            PAGE.with_borrow_mut(|page_cell| {
                if let Some(page) = page_cell.as_mut() {
                    serialized_bits += page.records.iter().map(|r| r.data.bits()).sum::<usize>();
                    page_count += 1;
                }
            });
        }
        let elapsed = tic.elapsed();
        eprintln!(
            "Traced {} pages in {:?} ({:.2} pages/sec, {:.2} Mbits/sec)",
            page_count,
            elapsed,
            page_count as f64 / elapsed.as_secs_f64(),
            serialized_bits as f64 / 1_000_000.0 / elapsed.as_secs_f64()
        );
    }
}
