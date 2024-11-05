use crate::problem::{self, Edge};

pub type Time = i32;

#[derive(Debug)]
pub struct Node {
    pub position: Time,
    pub delayed_after: Time,
    pub ub: Time,
    pub coeff: i32,
}

pub struct LongestPaths {
    pub nodes: Vec<Node>,
    outgoing: Vec<tinyvec::TinyVec<[(u32, Time); 4]>>,
    pub edge_undo_stack: Vec<Edge>,
    queue: Vec<u32>,
    trail: Vec<(u32, Time)>,
    pub trail_lim: Vec<u32>,
    pub objective_value: i32,
}

impl LongestPaths {
    pub fn new() -> LongestPaths {
        LongestPaths {
            nodes: Vec::new(),
            outgoing: Vec::new(),
            queue: Vec::new(),
            edge_undo_stack: Vec::new(),
            trail: Vec::new(),
            trail_lim: Vec::new(),
            objective_value: 0,
        }
    }

    pub fn add_node(&mut self, node: &problem::Node) {
        // println!("Add node lb{} ub{} del{}", lb, ub,delayed_after);
        // assert!(lb <= ub);
        let node = Node {
            ub: node.ub,
            position: node.lb,
            delayed_after: node.threshold,
            coeff: node.coeff as i32,
        };
        self.objective_value += Self::obj_component(&node);
        self.nodes.push(node);
        self.outgoing.push(Default::default());
    }

    pub fn obj_component(node: &Node) -> i32 {
        node.coeff * node.position.saturating_sub(node.delayed_after)
    }

    pub fn add_fixed_edge(&mut self, edge: Edge) -> bool {
        // trace!("fixed edge {:?}", edge);
        assert!(
            self.trail.is_empty() && self.trail_lim.is_empty() && self.edge_undo_stack.is_empty()
        );
        if !self.push_edge(edge, |_, _| {}) {
            return false;
        }

        self.trail.clear();
        self.trail_lim.clear();
        self.edge_undo_stack.clear();
        true
    }

    pub fn push_edge(&mut self, edge: Edge, mut bound_change: impl FnMut(u32, i32)) -> bool {
        // trace!("push {:?}", edge);
        // let _p = hprof::enter("push edge");

        self.outgoing[edge.src as usize].push((edge.tgt, edge.weight));
        self.edge_undo_stack.push(edge);
        self.trail_lim.push(self.trail.len() as u32);
        self.queue.clear();
        self.queue.push(edge.src);

        while let Some(node) = self.queue.pop() {
            for (next_node, dist) in self.outgoing[node as usize].iter().copied() {
                let target_position = self.nodes[node as usize].position + dist;
                let next_node_data = &mut self.nodes[next_node as usize];
                if next_node_data.position < target_position {
                    if next_node == edge.src || target_position > next_node_data.ub {
                        self.pop(|_| {});
                        return false;
                    }

                    self.trail.push((next_node, next_node_data.position));
                    let old_objective = Self::obj_component(next_node_data);
                    next_node_data.position = target_position;
                    let new_objective = Self::obj_component(next_node_data);

                    let delta_objective = new_objective - old_objective;
                    assert!(delta_objective >= 0);

                    self.objective_value += delta_objective;
                    if delta_objective > 0 {
                        bound_change(next_node, delta_objective);
                    }

                    let mut new_elem_idx = self.queue.len();
                    self.queue.push(next_node);
                    while new_elem_idx > 0
                        && self.nodes[self.queue[new_elem_idx] as usize].position
                            > self.nodes[self.queue[new_elem_idx - 1] as usize].position
                    {
                        self.queue.swap(new_elem_idx - 1, new_elem_idx);
                        new_elem_idx -= 1;
                    }
                }
            }
        }

        true
    }

    pub fn pop(&mut self, mut node_changed: impl FnMut(u32)) {
        // trace!("pop {:?}", self.edge_undo_stack[0]);
        // let _p = hprof::enter("pop edge");
        let edge = self.edge_undo_stack.pop().unwrap();

        // Remove from the outgoing list.
        let outgoing = &mut self.outgoing[edge.src as usize];
        // assert!(outgoing.last().unwrap() == &(edge.tgt, edge.dist));
        outgoing.pop();

        // Undo assignemnts
        for (n, p) in self
            .trail
            .drain((self.trail_lim.pop().unwrap() as usize)..)
            .rev()
        {
            let node_data = &mut self.nodes[n as usize];
            self.objective_value -= Self::obj_component(node_data);
            node_data.position = p;
            self.objective_value += Self::obj_component(node_data);
            node_changed(n);
        }
    }

    pub fn updated_since(&self, lim: usize) -> impl Iterator<Item = u32> + '_ {
        let start = if lim == 0 { 0 } else { self.trail_lim[lim - 1] };
        self.trail[(start as usize)..].iter().map(|(nd, _)| *nd)
    }

    pub fn hypothetical_edge_lb(&mut self, edge: Edge, bound_change: impl FnMut(u32, i32)) -> bool {
        self.push_edge(edge, bound_change) && {
            self.pop(|_| {});
            true
        }
    }
}

impl Default for LongestPaths {
    fn default() -> Self {
        Self::new()
    }
}
