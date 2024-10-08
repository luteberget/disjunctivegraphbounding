use std::collections::HashMap;

use log::{debug, trace};
use tinyvec::TinyVec;

use crate::{
    bnb::SolverSettings,
    longestpaths::LongestPaths,
    problem::{DisjunctiveGraph, Edge},
    wdg::{WdgEdge, WdgSolverBinaryMIP},
};

#[derive(Default, Debug)]
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
    wdg_solver: WdgSolverBinaryMIP,
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
        trace!("Partitions: {:?}", partitions);

        Some(Self {
            schedule,
            nonunit_disjunctions,
            n_partitions,
            partitions,
            wdg_solver: WdgSolverBinaryMIP::default(),
        })
    }

    pub fn longestpaths_bound(&self) -> i32 {
        self.schedule.objective_value
    }

    pub fn mk_state(&mut self, settings: &SolverSettings, pre_lb :i32, cost_ub: i32) -> Option<State> {
        let mut branching: Option<(i32, TinyVec<[Edge; 2]>)> = None;
        let mut lb: Option<i32> = None;
        self.wdg_solver.clear();
        let realized_cost = self.schedule.objective_value;
        // debug!("realized cost {}", realized_cost);

        // Strong branching + gather conflict bounding problem coefficients
        for es in self.nonunit_disjunctions.iter() {
            // Skip disjunctions that are satisfied by the relaxed schedule
            if es.iter().any(|e| {
                let t1 = self.schedule.nodes[e.src as usize].position;
                let t2 = self.schedule.nodes[e.tgt as usize].position;
                t1 + e.weight <= t2
            }) {
                // trace!("skipping satsified constraint {:?}", es);
                continue;
            }

            let valid_edges: TinyVec<[(Edge, i32); 2]> = es
                .iter()
                .filter_map(|x| {
                    self.schedule
                        .hypothetical_edge_lb(*x, |_| true)
                        .map(|lb| (*x, lb - realized_cost))
                })
                .filter(|(_e, bound_increase)| realized_cost + bound_increase < cost_ub)
                .collect();

            // Short-circuit when there is a forced edge or infeasibility.
            if valid_edges.len() < 2 {
                lb = Some(pre_lb);
                branching = Some((i32::MAX, valid_edges.into_iter().map(|x| x.0).collect()));
                break;
            }

            // Gather conflicts based on the partition
            if settings.use_wdg_bound && valid_edges.len() == 2 {
                let wdg_edge1 = {
                    let edge = valid_edges[0].0;
                    let partition = self.partitions[edge.tgt as usize];
                    self.schedule
                        .hypothetical_edge_lb(edge, |other_node| {
                            self.partitions[other_node as usize] == partition
                        })
                        .map(|w| WdgEdge {
                            partition,
                            d_cost: w - realized_cost,
                        })
                }
                .unwrap();

                if wdg_edge1.d_cost > 0 {
                    let wdg_edge2 = {
                        let edge = valid_edges[1].0;
                        let partition = self.partitions[edge.tgt as usize];
                        self.schedule
                            .hypothetical_edge_lb(edge, |other_node| {
                                self.partitions[other_node as usize] == partition
                            })
                            .map(|w| WdgEdge {
                                partition,
                                d_cost: w - realized_cost,
                            })
                    }
                    .unwrap();

                    if wdg_edge2.d_cost > 0 {
                        self.wdg_solver.add_edge_pair(wdg_edge1, wdg_edge2);
                    }
                }
            }

            // Compute the strong-branching or chronology score.

            let score = if settings.use_strong_branching {
                let min_lb_incr = valid_edges.iter().map(|(_, d_lb)| d_lb).min().unwrap();
                let max_lb_incr = valid_edges.iter().map(|(_, d_lb)| d_lb).max().unwrap();

                5 * min_lb_incr + max_lb_incr
            } else {
                let earliest_time = es
                    .iter()
                    .flat_map(|e| {
                        let t1 = self.schedule.nodes[e.src as usize].position;
                        let t2 = self.schedule.nodes[e.tgt as usize].position;
                        [t1, t2]
                    })
                    .min()
                    .unwrap();

                -earliest_time
            };

            if score > branching.as_ref().map(|(s, _)| *s).unwrap_or(i32::MIN) {
                branching = Some((score, valid_edges.into_iter().map(|(e, _)| e).collect()));
            }
        }

        let lb = lb.unwrap_or_else(|| realized_cost + self.wdg_solver.solve(self.n_partitions));
        if lb >= cost_ub {
            return None;
        }
        
        trace!("mk_state ub={} cost={} lb={}", cost_ub, realized_cost, lb);

        Some(State {
            lb,
            branching: branching.map(|x| x.1),
        })
    }

    pub fn push(&mut self, e: Edge) -> bool {
        self.schedule.push_edge(e, |_| true)
    }

    pub fn pop(&mut self) {
        self.schedule.pop(|_| {});
    }
}
