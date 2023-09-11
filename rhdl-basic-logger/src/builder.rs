use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};

use rhdl_core::{ClockDetails, LogBuilder, Synthesizable, TagID};

use crate::{
    logger::{LogSignal, ScopeRecord, TaggedSignal},
    Logger,
};

#[derive(Clone, Debug, Default)]
struct BuilderInner {
    scopes: Vec<ScopeRecord<'static>>,
    clocks: Vec<ClockDetails>,
}

// I don't like the use of interior mutability here.
// I need to redesign the API so it is not required.
#[derive(Clone, Debug)]
pub struct Builder {
    inner: Rc<RefCell<BuilderInner>>,
    path: Vec<String>,
    my_scope: usize,
}

impl Display for Builder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for scope in self.inner.borrow().scopes.iter() {
            writeln!(f, "{}", scope)?;
        }
        Ok(())
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(BuilderInner {
                scopes: vec![ScopeRecord {
                    name: "root".to_string(),
                    tags: Vec::new(),
                }],
                ..Default::default()
            })),
            path: vec![],
            my_scope: 0,
        }
    }
}

impl LogBuilder for Builder {
    type SubBuilder = Self;
    fn scope(&self, name: &str) -> Self {
        let name = format!(
            "{}::{}",
            self.inner.borrow().scopes[self.my_scope].name,
            name
        );
        self.inner.borrow_mut().scopes.push(ScopeRecord {
            name,
            tags: Vec::new(),
        });
        Self {
            inner: self.inner.clone(),
            path: vec![],
            my_scope: self.inner.borrow().scopes.len() - 1,
        }
    }

    fn tag<T: Synthesizable>(&mut self, name: &str) -> TagID<T> {
        let context_id: usize = self.my_scope;
        let tag = {
            let scope = &mut self.inner.borrow_mut().scopes[context_id];
            scope.tags.push(TaggedSignal {
                tag: name.to_string(),
                data: Vec::new(),
            });
            TagID {
                context: context_id,
                id: scope.tags.len() - 1,
                _marker: Default::default(),
            }
        };
        T::allocate(tag, self);
        tag
    }

    fn allocate<T: Synthesizable>(&self, tag: TagID<T>, width: usize) {
        let name = self.path.join("$");
        let signal = LogSignal::new(name, width);
        let context_id: usize = tag.context;
        let scope = &mut self.inner.borrow_mut().scopes[context_id];
        let tag_id: usize = tag.id;
        let tag = &mut scope.tags[tag_id];
        tag.data.push(signal);
    }

    fn namespace(&self, name: &str) -> Self {
        let mut new_path = self.path.clone();
        new_path.push(name.to_string());
        Self {
            inner: self.inner.clone(),
            path: new_path,
            my_scope: self.my_scope,
        }
    }

    fn add_clock(&mut self, clock: ClockDetails) {
        self.inner.borrow_mut().clocks.push(clock);
    }
}

impl Builder {
    pub fn build(self) -> Logger<'static> {
        let inner = self.inner.take();
        Logger {
            scopes: inner.scopes,
            clocks: inner.clocks,
            field_index: 0,
            time_in_fs: 0,
        }
    }
}
