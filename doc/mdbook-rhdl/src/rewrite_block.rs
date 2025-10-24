use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

pub struct BlockRewriter<I> {
    events: I,
    in_block: bool,
    block_text: String,
    spool: Vec<Event<'static>>,
    proc: fn(usize, &str, &str) -> Vec<Event<'static>>,
    source_path: Option<std::path::PathBuf>,
    tag: &'static str,
    captured_tag: String,
    block_counter: usize,
    cached: bool,
}

impl<I> BlockRewriter<I> {
    fn new(
        source_path: &Option<std::path::PathBuf>,
        events: I,
        proc: fn(usize, &str, &str) -> Vec<Event<'static>>,
        tag: &'static str,
        cached: bool,
    ) -> Self {
        Self {
            events,
            in_block: false,
            block_text: String::new(),
            spool: Vec::new(),
            proc,
            source_path: source_path.clone(),
            tag,
            captured_tag: String::new(),
            block_counter: 0,
            cached,
        }
    }
}

impl<'a, I> Iterator for BlockRewriter<I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.spool.is_empty() {
            return self.spool.remove(0).into();
        }
        loop {
            let event = self.events.next()?;

            match (&event, self.in_block) {
                // Start of rhdl-shell block
                (Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))), false)
                    if lang.as_ref().starts_with(self.tag) =>
                {
                    self.in_block = true;
                    self.block_text.clear();
                    self.captured_tag = lang.to_string();
                    self.block_counter += 1;
                    // Skip this event, don't return it
                    continue;
                }
                // Text inside the block
                (Event::Text(content), true) => {
                    self.block_text.push_str(content);
                    // Skip this event, don't return it
                    continue;
                }
                // End of block
                (Event::End(TagEnd::CodeBlock), true) => {
                    self.in_block = false;
                    let mut context = md5::Context::new();
                    let path = self
                        .source_path
                        .as_ref()
                        .map_or("".to_string(), |p| p.to_string_lossy().to_string());
                    context.consume(&path);
                    context.consume(&self.captured_tag);
                    context.consume(self.block_counter.to_le_bytes());
                    context.consume(&self.block_text);
                    let hash = context.finalize();
                    // Check to see if a cache file exists for this block already
                    let cache_key = format!("src/db/{:x}.json", hash);
                    let mut fetched = false;
                    if let Some(cached) = std::fs::read_to_string(&cache_key).ok()
                        && let Ok(cached) = serde_json::from_str::<Vec<Event<'_>>>(&cached)
                    {
                        self.spool = cached.into_iter().map(|e| e.into_static()).collect();
                        fetched = true;
                    }
                    if !fetched || !self.cached {
                        self.spool =
                            (self.proc)(self.block_counter, &self.captured_tag, &self.block_text);
                        if let Ok(serialized) = serde_json::to_string(&self.spool) {
                            let _ = std::fs::write(&cache_key, serialized);
                        }
                    }
                    if self.spool.is_empty() {
                        continue; // No events to return, continue the loop
                    }
                    return self.spool.remove(0).into();
                }
                // Any other event - pass through unchanged
                _ => return Some(event),
            }
        }
    }
}

pub trait BlockRewriterExt: Iterator {
    fn rewrite_blocks(
        self,
        source_path: &Option<std::path::PathBuf>,
        proc: fn(usize, &str, &str) -> Vec<Event<'static>>,
        tag: &'static str,
    ) -> BlockRewriter<Self>
    where
        Self: Sized,
    {
        BlockRewriter::new(source_path, self, proc, tag, true)
    }
    fn rewrite_blocks_uncached(
        self,
        source_path: &Option<std::path::PathBuf>,
        proc: fn(usize, &str, &str) -> Vec<Event<'static>>,
        tag: &'static str,
    ) -> BlockRewriter<Self>
    where
        Self: Sized,
    {
        BlockRewriter::new(source_path, self, proc, tag, false)
    }
}

// Implement the trait for all iterators that yield Event<'a>
impl<'a, I> BlockRewriterExt for I where I: Iterator<Item = Event<'a>> {}
