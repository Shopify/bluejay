#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct NodeId(usize);

pub(super) struct Arena<T> {
    nodes: Vec<T>,
}

impl<T> Arena<T> {
    pub(super) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub(super) fn next_id(&self) -> NodeId {
        NodeId(self.nodes.len())
    }

    pub(super) fn add(&mut self, node: T) -> NodeId {
        let id = self.next_id();
        self.nodes.push(node);
        id
    }

    pub(super) fn get(&self, id: NodeId) -> Option<&T> {
        self.nodes.get(id.0)
    }

    pub(super) fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.nodes.get_mut(id.0)
    }
}
