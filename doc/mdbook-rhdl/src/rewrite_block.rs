use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

pub struct BlockRewriter<I> {
    events: I,
    in_block: bool,
    block_text: String,
    spool: Vec<Event<'static>>,
    proc: fn(&str) -> Vec<Event<'static>>,
    tag: &'static str,
}

impl<I> BlockRewriter<I> {
    fn new(events: I, proc: fn(&str) -> Vec<Event<'static>>, tag: &'static str) -> Self {
        Self {
            events,
            in_block: false,
            block_text: String::new(),
            spool: Vec::new(),
            proc,
            tag,
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
                    if lang.as_ref() == self.tag =>
                {
                    self.in_block = true;
                    self.block_text.clear();
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
                    self.spool = (self.proc)(&self.block_text);
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
        proc: fn(&str) -> Vec<Event<'static>>,
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
