use tinyvec::TinyVec;

use crate::problem::{Edge, Node};

pub struct Schedule {
    t: Vec<(i32, i32)>,
    outgoing_edges: Vec<TinyVec<[(u32, i32); 6]>>,
}

impl Schedule {
    pub fn new<'a>(nodes: impl IntoIterator<Item = &'a Node> + 'a) -> Self {
        let mut t: Vec<(i32, i32)> = Vec::new();
        let mut outgoing_edges = Vec::new();
        for n in nodes.into_iter() {
            t.push((n.lb, n.ub));
            outgoing_edges.push(Default::default());
        }

        Self {
            t,
            outgoing_edges,
        }
    }

    pub fn objval(&self) -> i32 {todo!()}

    pub fn add_fixed_edge(&mut self, edge :&Edge) -> bool {
        todo!()
    }

    pub fn push(&mut self ,edge :Edge) -> bool {
        todo!()
    }

    pub fn pop(&mut self) -> Edge {
        todo!()
    }
}
