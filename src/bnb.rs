use std::{cmp::Reverse, collections::BinaryHeap, rc::Rc};

use crate::{
    problem::{DisjunctiveCliquesProblem, Edge},
    schedule::Schedule,
};

struct Node {
    lb: i32,
    parent: Option<(Rc<Node>, Edge)>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.lb == other.lb
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.lb.cmp(&other.lb)
    }
}

pub fn solve(problem: &DisjunctiveCliquesProblem) -> Option<u32> {

    // Create the schedule with fixed edges
    let mut schedule = Schedule::new(&problem.disjunctive_graph.nodes);

    // Add fixed edges
    for edge in problem.disjunctive_graph.edge_sets.iter() {
        match edge.as_slice() {
            [] => return None,
            [x] => {
                if !schedule.add_fixed_edge(x) {
                    return None;
                }
            }
            xs => {}
        }
    }

    let root_node = Rc::new(Node {
        lb: todo!(),
        parent: None,
    });

    let mut queue: BinaryHeap<Reverse<Rc<Node>>> = BinaryHeap::new();
    queue.push(Reverse(root_node));

    todo!()
}
