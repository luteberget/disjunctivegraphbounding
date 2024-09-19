use std::collections::HashMap;

use tinyvec::TinyVec;

use crate::{
    longestpaths::LongestPaths,
    problem::{DisjunctiveGraph, Edge},
};

#[derive(Default)]
pub struct State {
    pub lb: i32,
    pub branching: Option<TinyVec<[Edge; 2]>>,
}

type PartitionId = u32;

pub struct World {
    schedule: LongestPaths,
    nonunit_disjunctions: Vec<TinyVec<[Edge; 2]>>,

    n_partitions: usize,
    partitions: Vec<PartitionId>,
}

impl World {
    pub fn new(problem: &DisjunctiveGraph) -> Option<Self> {
        let mut schedule = LongestPaths::new();
        for node in problem.nodes.iter() {
            schedule.add_node(node);
        }

        let mut nonunit_disjunctions: Vec<TinyVec<[Edge; 2]>> = Default::default();
        let mut partitioning_uf = petgraph::unionfind::UnionFind::new(problem.nodes.len());

        // Add fixed edges
        for edge in problem.edge_sets.iter() {
            match edge.as_slice() {
                [] => return None,
                [x] => {
                    if !schedule.add_fixed_edge(*x) {
                        return None;
                    }
                    partitioning_uf.union(x.src, x.tgt);
                }
                xs => {
                    nonunit_disjunctions.push(xs.iter().copied().collect());
                }
            }
        }

        let mut partitioning_representatives: HashMap<u32, usize> = Default::default();
        let mut n_partitions = 0;
        let mut partitions = Vec::new();
        for (node_idx, _node) in problem.nodes.iter().enumerate() {
            let partition_idx = *partitioning_representatives
                .entry(partitioning_uf.find_mut(node_idx as u32))
                .or_insert_with(|| {
                    n_partitions += 1;
                    n_partitions - 1
                });
            partitions.push(partition_idx as u32);
        }

        Some(Self {
            schedule,
            nonunit_disjunctions,
            n_partitions,
            partitions,
        })
    }

    pub fn mk_state(&mut self, cost_ub: i32) -> State {
        let mut branching: Option<(i32, TinyVec<[Edge; 2]>)> = None;
        let mut lb: Option<i32> = None;

        let realized_cost = self.schedule.objective_value;

        // Strong branching + gather conflict bounding problem coefficients
        for es in self.nonunit_disjunctions.iter() {
            // Skip disjunctions that are satisfied by the relaxed schedule
            if es.iter().any(|e| {
                let t1 = self.schedule.nodes[e.src as usize].position;
                let t2 = self.schedule.nodes[e.tgt as usize].position;
                t1 + e.weight <= t2
            }) {
                continue;
            }

            let valid_edges: TinyVec<[(Edge, i32); 2]> = es
                .iter()
                .filter_map(|x| self.schedule.hypothetical_edge_lb(*x).map(|lb| (*x, lb - realized_cost)))
                .collect();

            // Short-circuit when there is a forced edge or infeasibility.
            if valid_edges.len() < 2 {
                branching = Some((i32::MAX, valid_edges.into_iter().map(|x| x.0).collect()));
                lb = Some(i32::MIN);
                break;
            }

            // Gather conflicts based on the partition
            if valid_edges.len() == 2 {
                let lb_edge1 = 
            }
        }

        let lb = lb.unwrap_or_else(|| 0);

        State {
            lb,
            branching: branching.map(|x| x.1),
        }
    }

    pub fn push(&mut self, e: Edge) -> bool {
        self.schedule.push_edge(e)
    }

    pub fn pop(&mut self) {
        self.schedule.pop(|_| {});
    }
}
