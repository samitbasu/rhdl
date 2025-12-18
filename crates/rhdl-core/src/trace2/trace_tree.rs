//! A trace tree is a hierarchical description of the trace elements
//! in a trace.  It is a folders/files model, in which the paths
//! of the trace elements are represented as a tree of scopes, with
//! individual trace signals as leaves.
//!
//! For example, if we had trace data that looked like this:
//!
//! ```text
//! root/baz/bar/a
//! root/baz/foo/b
//! root/c
//! ```
//!
//! Then the scope tree would look like this:
//!
//! ```text
//! root
//! ├── baz
//! │   ├── bar
//! │   │   └── a
//! │   └── foo
//! │       └── b
//! └── c
//! ```

use std::collections::BTreeMap;

use crate::trace2::{TraceId, meta::TraceDetails};

pub(crate) struct TSItem<'a> {
    pub(crate) path: &'a [&'static str],
    pub(crate) name: &'a str,
    pub(crate) id: TraceId,
}

#[derive(Default)]
pub(crate) struct TraceTree {
    pub(crate) children: BTreeMap<&'static str, Box<TraceTree>>,
    pub(crate) signals: BTreeMap<String, TraceId>,
}

impl TraceTree {
    pub(crate) fn build<'a>(paths: impl Iterator<Item = &'a TraceDetails>) -> TraceTree {
        hierarchical_walk(paths)
    }
}

fn hierarchical_walk<'a>(paths: impl Iterator<Item = &'a TraceDetails>) -> TraceTree {
    let mut root = TraceTree::default();
    for ts_item in paths {
        let mut folder = &mut root;
        for item in ts_item.path.iter() {
            if !folder.children.contains_key(item) {
                let new_folder = Box::new(TraceTree::default());
                folder.children.insert(item, new_folder);
            }
            folder = folder.children.get_mut(item).unwrap();
        }
        folder.signals.insert(ts_item.key.clone(), ts_item.trace_id);
    }
    root
}
