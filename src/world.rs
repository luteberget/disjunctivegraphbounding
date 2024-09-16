use tinyvec::TinyVec;

use crate::{
    problem::{DisjunctiveCliquesProblem, Edge},
    schedule::Schedule,
};

#[derive(Default)]
pub struct State {
    pub lb :i32,
    pub branching :Option<TinyVec<[Edge; 2]>>,
}

pub struct World {}

impl World {
    pub fn new(problem: &DisjunctiveCliquesProblem) -> Option<Self> {
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

        Some(Self {

        })
    }

    pub fn mk_state(&self, bound :i32) -> State {
        todo!()
    }

    pub fn push(&self, e: Edge) -> bool {
        todo!()
    }

    pub fn pop(&mut self) {
        todo!()
    }
}
