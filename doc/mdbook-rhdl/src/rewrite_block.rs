use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

pub struct BlockRewriter<I> {
    events: I,
    in_block: bool,
    block_text: String,
    spool: Vec<Event<'static>>,
    proc: fn(&str, &str) -> Vec<Event<'static>>,
    tag: &'static str,
    captured_tag: String,
    block_counter: usize,
}

impl<I> BlockRewriter<I> {
    fn new(events: I, proc: fn(&str, &str) -> Vec<Event<'static>>, tag: &'static str) -> Self {
        Self {
            events,
            in_block: false,
            block_text: String::new(),
            spool: Vec::new(),
            proc,
            tag,
            captured_tag: String::new(),
            block_counter: 0,
        }
    }
}

impl<'a, I> Iterator for BlockRewriter<I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.spool.len() > 0 {
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
                    // Check to see if a cache file exists for this block already
                    let cache_key = format!(
                        "src/db/{}.json",
                        heck::AsSnakeCase(format!(
                            "block_{}_{}",
                            self.captured_tag, self.block_counter
                        ))
                    );
                    let mut fetched = false;
                    if let Some(cached) = std::fs::read_to_string(&cache_key).ok() {
                        if let Ok(cached) = serde_json::from_str::<Vec<Event<'_>>>(&cached) {
                            self.spool = cached.into_iter().map(|e| e.into_static()).collect();
                            fetched = true;
                        }
                    }
                    if !fetched {
                        self.spool = (self.proc)(&self.captured_tag, &self.block_text);
                        if let Ok(serialized) = serde_json::to_string(&self.spool) {
                            let _ = std::fs::write(&cache_key, serialized);
                        }
                    }
                    if self.spool.len() == 0 {
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
        proc: fn(&str, &str) -> Vec<Event<'static>>,
        tag: &'static str,
    ) -> BlockRewriter<Self>
    where
        Self: Sized,
    {
        BlockRewriter::new(self, proc, tag)
    }
}

// Implement the trait for all iterators that yield Event<'a>
impl<'a, I> BlockRewriterExt for I where I: Iterator<Item = Event<'a>> {}
