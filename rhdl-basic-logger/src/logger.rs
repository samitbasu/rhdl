use std::{
    fmt::{Display, Formatter},
    io::Write,
};

use indexmap::IndexMap;
use rhdl_core::{logger::LoggerImpl, ClockDetails, Digital, TagID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TimedValue<T: Clone + PartialEq + Eq> {
    pub(crate) time_in_fs: u64,
    pub(crate) value: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum LogValues<'a> {
    Bool(Vec<TimedValue<bool>>),
    Bits(Vec<TimedValue<u128>>),
    #[serde(borrow)]
    Enum(Vec<TimedValue<&'a str>>),
}

impl<'a> LogValues<'a> {
    pub(crate) fn len(&self) -> usize {
        match self {
            LogValues::Bool(v) => v.len(),
            LogValues::Bits(v) => v.len(),
            LogValues::Enum(v) => v.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LogSignal<'a> {
    pub(crate) name: String,
    pub(crate) width: usize,
    #[serde(borrow)]
    pub(crate) values: LogValues<'a>,
}

impl<'a> LogSignal<'a> {
    pub(crate) fn new(name: String, width: usize) -> LogSignal<'a> {
        LogSignal {
            name,
            width,
            values: if width == 0 {
                LogValues::Enum(vec![])
            } else if width == 1 {
                LogValues::Bool(vec![])
            } else {
                LogValues::Bits(vec![])
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TaggedSignal<'a> {
    pub(crate) tag: String,
    #[serde(borrow)]
    pub(crate) data: Vec<LogSignal<'a>>,
}

impl<'a> Display for TaggedSignal<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for signal in &self.data {
            writeln!(
                f,
                "{}::{} [{}] --> {}",
                self.tag,
                signal.name,
                signal.width,
                signal.values.len()
            )?;
        }
        Ok(())
    }
}

struct SignalPointer<'a> {
    signal: &'a LogSignal<'static>,
    index: usize,
    code: vcd::IdCode,
    code_as_bytes: Vec<u8>,
}

enum ScopeNode<'a> {
    Internal {
        children: IndexMap<String, ScopeNode<'a>>,
    },
    Leaf {
        width: usize,
        code: Option<vcd::IdCode>,
        signal: &'a LogSignal<'static>,
    },
}

impl<'a> ScopeNode<'a> {
    fn new_scope() -> Self {
        ScopeNode::Internal {
            children: IndexMap::new(),
        }
    }
    fn children(&mut self) -> &mut IndexMap<String, ScopeNode<'a>> {
        match self {
            ScopeNode::Internal { children } => children,
            ScopeNode::Leaf { .. } => panic!("Leaf node"),
        }
    }
    fn children_at(&mut self, path: &[&str]) -> &mut IndexMap<String, ScopeNode<'a>> {
        if let Some((&first, rest)) = path.split_first() {
            self.children()
                .entry(first.to_owned())
                .or_insert_with(ScopeNode::new_scope)
                .children_at(rest)
        } else {
            self.children()
        }
    }
}

fn build_signal_pointer_list<'a>(node: &ScopeNode<'a>) -> Vec<SignalPointer<'a>> {
    match node {
        ScopeNode::Internal { children } => children
            .iter()
            .flat_map(|(_, child)| build_signal_pointer_list(child))
            .collect(),
        ScopeNode::Leaf {
            width: _,
            code,
            signal,
        } => vec![SignalPointer {
            signal,
            index: 0,
            code: code.unwrap(),
            code_as_bytes: code.unwrap().to_string().into_bytes(),
        }],
    }
}

impl<'a> ScopeNode<'a> {
    fn dump(&self, indent_level: usize) {
        match self {
            ScopeNode::Internal { children } => {
                for (name, child) in children {
                    println!("{}{}", "  ".repeat(indent_level), name);
                    child.dump(indent_level + 1);
                }
            }
            ScopeNode::Leaf {
                width,
                code,
                signal: _,
            } => {
                println!("{}[{}] {:?}", "  ".repeat(indent_level), width, code);
            }
        }
    }
    fn register<W: Write>(&mut self, name: &str, v: &mut vcd::Writer<W>) -> anyhow::Result<()> {
        match self {
            ScopeNode::Internal { children } => {
                v.add_module(name)?;
                for (name, child) in children {
                    child.register(name.as_str(), v)?;
                }
                v.upscope()?
            }
            ScopeNode::Leaf {
                width,
                code,
                signal: _,
            } => *code = Some(v.add_wire(*width as u32, name)?),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScopeRecord<'a> {
    pub(crate) name: String,
    #[serde(borrow)]
    pub(crate) tags: Vec<TaggedSignal<'a>>,
}

impl<'a> Display for ScopeRecord<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for tag in &self.tags {
            for signal in &tag.data {
                writeln!(
                    f,
                    "<{}>::{}::{} [{}] --> {}",
                    self.name,
                    tag.tag,
                    signal.name,
                    signal.width,
                    signal.values.len()
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Logger<'a> {
    #[serde(borrow)]
    pub(crate) scopes: Vec<ScopeRecord<'a>>,
    pub(crate) clocks: Vec<ClockDetails>,
    pub(crate) field_index: usize,
    pub(crate) time_in_fs: u64,
}

impl<'a> Display for Logger<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for scope in &self.scopes {
            writeln!(f, "{}", scope)?;
        }
        Ok(())
    }
}

// Uses the names of the scopes (which are seperated by ::) to
// build a tree of scope names for writing to the VCD file
// in a hierarchical manner.

impl Logger<'static> {
    fn signal<T: Digital>(&mut self, tag_id: TagID<T>) -> &mut LogSignal<'static> {
        let scope = &mut self.scopes[tag_id.context];
        let tag = &mut scope.tags[tag_id.id];
        let len = tag.data.len();
        let ret = &mut tag.data[self.field_index];
        self.field_index = (self.field_index + 1) % len;
        ret
    }
    fn build_scope_tree(&self) -> ScopeNode {
        let mut root = ScopeNode::new_scope();
        for scope in &self.scopes {
            println!("scope name: {}", scope.name);
            let path: Vec<_> = scope.name.split("::").collect();
            for tag in &scope.tags {
                // There are two possibilities for tags.
                // One is a tag that stores a struct, in which case,
                // there are named elements beneath the tag.  In
                // the other case, the tag just holds a single data element.
                // We treat these differently - in the first case, we
                // treat the tag as a scope.  In the second, we treat it as a signal.
                if tag.data.len() == 1 {
                    let signal = &tag.data[0];
                    println!("signal name: {}", signal.name);
                    root.children_at(&path)
                        .entry(tag.tag.clone())
                        .or_insert_with(|| ScopeNode::Leaf {
                            width: signal.width,
                            code: None,
                            signal,
                        });
                } else {
                    println!("Structured tag {}", tag.tag);
                    let tag_root = root
                        .children_at(&path)
                        .entry(tag.tag.clone())
                        .or_insert_with(ScopeNode::new_scope);
                    for signal in &tag.data {
                        println!("signal name: {}", signal.name);
                        tag_root
                            .children()
                            .entry(signal.name.clone())
                            .or_insert_with(|| ScopeNode::Leaf {
                                width: signal.width,
                                code: None,
                                signal,
                            });
                    }
                }
            }
        }
        root
    }
    pub fn vcd<W: Write>(self, w: W) -> anyhow::Result<()> {
        let mut writer = vcd::Writer::new(w);
        writer.timescale(1, vcd::TimescaleUnit::FS)?;
        writer.add_module("top")?;
        let clocks = self
            .clocks
            .iter()
            .map(|c| {
                (
                    c,
                    writer
                        .add_wire(1, &c.name)
                        .unwrap()
                        .to_string()
                        .into_bytes(),
                )
            })
            .collect::<Vec<_>>();
        let mut tree = self.build_scope_tree();
        tree.register("", &mut writer)?;
        writer.upscope()?;
        writer.enddefinitions()?;
        writer.timestamp(0)?;
        let mut signal_pointers = build_signal_pointer_list(&tree);
        let mut current_time = 0;
        // Find the next sample time (if any), and log any values that have the current timestamp
        let mut keep_running = true;
        let mut sbuf = [0_u8; 256];
        while keep_running {
            keep_running = false;
            let mut next_time = !0;
            // Write states for each clock
            for (clock, code) in &clocks {
                if clock.pos_edge_at(current_time) {
                    writer.writer().write_all(b"1")?;
                    writer.writer().write_all(code)?;
                    writer.writer().write_all(b"\n")?;
                } else if clock.neg_edge_at(current_time) {
                    writer.writer().write_all(b"0")?;
                    writer.writer().write_all(code)?;
                    writer.writer().write_all(b"\n")?;
                }
                next_time = next_time.min(clock.next_edge_after(current_time));
            }
            let mut found_match = true;
            while found_match {
                found_match = false;
                for ptr in &mut signal_pointers {
                    match ptr.signal.values {
                        LogValues::Bool(ref values) => {
                            if let Some(value) = values.get(ptr.index) {
                                if value.time_in_fs == current_time {
                                    writer.writer().write_all(bool_to_vcd(value.value))?;
                                    writer.writer().write_all(&ptr.code_as_bytes)?;
                                    writer.writer().write_all(b"\n")?;
                                    ptr.index += 1;
                                    found_match = true;
                                } else {
                                    next_time = next_time.min(value.time_in_fs);
                                }
                                keep_running = true;
                            }
                        }
                        LogValues::Bits(ref values) => {
                            if let Some(value) = values.get(ptr.index) {
                                if value.time_in_fs == current_time {
                                    sbuf[0] = b'b';
                                    bits_to_vcd(value.value, ptr.signal.width, &mut sbuf[1..]);
                                    sbuf[ptr.signal.width + 1] = b' ';
                                    writer
                                        .writer()
                                        .write_all(&sbuf[0..(ptr.signal.width + 2)])?;
                                    writer.writer().write_all(&ptr.code_as_bytes)?;
                                    writer.writer().write_all(&[b'\n'])?;
                                    ptr.index += 1;
                                    found_match = true;
                                } else {
                                    next_time = next_time.min(value.time_in_fs);
                                }
                                keep_running = true;
                            }
                        }
                        LogValues::Enum(ref values) => {
                            if let Some(value) = values.get(ptr.index) {
                                if value.time_in_fs == current_time {
                                    writer.change_string(ptr.code, value.value.unwrap_or("X"))?;
                                    ptr.index += 1;
                                    found_match = true;
                                } else {
                                    next_time = next_time.min(value.time_in_fs);
                                }
                                keep_running = true;
                            }
                        }
                    }
                }
            }
            if next_time != !0 {
                current_time = next_time;
                writer.timestamp(current_time)?;
            }
        }
        Ok(())
    }
    pub fn dump(&self) {
        let tree = self.build_scope_tree();
        tree.dump(0);
    }
}

impl rhdl_core::Logger for Logger<'static> {
    type Impl = Self;
    fn set_time_in_fs(&mut self, time: u64) {
        self.time_in_fs = time;
    }
    fn get_impl(&mut self) -> &mut Self::Impl {
        self
    }
}

fn bool_to_vcd(x: Option<bool>) -> &'static [u8] {
    match x {
        Some(true) => b"1",
        Some(false) => b"0",
        None => b"x",
    }
}

fn bits_to_vcd(x: Option<u128>, width: usize, buffer: &mut [u8]) {
    if let Some(x) = x {
        (0..width).for_each(|i| {
            buffer[i] = if x & (1 << (width - 1 - i)) != 0 {
                b'1'
            } else {
                b'0'
            };
        });
    } else {
        (0..width).for_each(|i| {
            buffer[i] = b'x';
        });
    }
}

impl LoggerImpl for Logger<'static> {
    fn write_bool<T: Digital>(&mut self, tag_id: TagID<T>, value: bool) {
        let time_in_fs = self.time_in_fs;
        if let LogValues::Bool(ref mut values) = self.signal(tag_id).values {
            values.push(TimedValue {
                time_in_fs,
                value: Some(value),
            });
        } else {
            panic!("Wrong type");
        }
    }
    fn write_bits<T: Digital>(&mut self, tag_id: TagID<T>, value: u128) {
        let time_in_fs = self.time_in_fs;
        if let LogValues::Bits(ref mut values) = self.signal(tag_id).values {
            values.push(TimedValue {
                time_in_fs,
                value: Some(value),
            });
        } else {
            panic!("Wrong type");
        }
    }
    fn write_string<T: Digital>(&mut self, tag_id: TagID<T>, val: &'static str) {
        let time_in_fs = self.time_in_fs;
        if let LogValues::Enum(ref mut values) = self.signal(tag_id).values {
            values.push(TimedValue {
                time_in_fs,
                value: Some(val),
            });
        } else {
            panic!("Wrong type");
        }
    }
    fn skip<T: Digital>(&mut self, tag_id: TagID<T>) {
        let time_in_fs = self.time_in_fs;
        match self.signal(tag_id).values {
            LogValues::Bool(ref mut values) => {
                values.push(TimedValue {
                    time_in_fs,
                    value: None,
                });
            }
            LogValues::Bits(ref mut values) => {
                values.push(TimedValue {
                    time_in_fs,
                    value: None,
                });
            }
            LogValues::Enum(ref mut values) => {
                values.push(TimedValue {
                    time_in_fs,
                    value: None,
                });
            }
        }
    }
}
