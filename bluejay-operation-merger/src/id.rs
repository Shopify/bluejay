use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(usize);

#[derive(Default, Clone)]
pub struct IdGenerator {
    next_id: Rc<AtomicUsize>,
}

impl IdGenerator {
    pub fn next(&self) -> Id {
        Id(self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}
