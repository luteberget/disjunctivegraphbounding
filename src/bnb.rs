use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    rc::Rc,
    time::{Duration, Instant},
};

use log::{debug, info, trace};
use tinyvec::TinyVec;

use crate::{
    problem::{DisjunctiveGraph, Edge},
    world::{State, World},
};

pub struct SolverSettings {
    pub use_strong_branching: bool,
    pub use_wdg_bound: bool,
    pub use_relaxed_wdg :bool,
}

#[derive(Default)]
struct Node {
    state: State,
    depth: u32,
    parent: Option<(Rc<Node>, Edge)>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.state.lb == other.state.lb
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
        self.state.lb.cmp(&other.state.lb)
    }
}

#[derive(Debug)]
pub struct SolverStats {
    pub n_states_generated: usize,
    pub n_nodes_generated: usize,
    pub n_nodes_solved: usize,
    pub max_depth: u32,
    pub solution_depth: u32,
    pub root_bound: i32,
    pub best_bound: i32,
    pub best_value: i32,
}

pub fn solve(
    problem: &DisjunctiveGraph,
    settings: &SolverSettings,
    timeout: Duration,
) -> (SolverStats, Option<i32>) {
    let start_time = Instant::now();
    let mut stats = SolverStats {
        max_depth: 0,
        n_nodes_generated: 0,
        n_nodes_solved: 0,
        n_states_generated: 0,
        solution_depth: u32::MAX,
        root_bound: 0,
        best_bound: 0,
        best_value: i32::MAX,
    };
    let mut world = match World::new(problem) {
        None => {
            return (stats, None);
        }
        Some(w) => w,
    };

    // let root_longestpath_bound = world.longestpaths_bound();
    let mut world_state = Rc::new(Node {
        state: world
            .mk_state(settings, world.longestpaths_bound(), i32::MAX)
            .unwrap(),
        depth: 0,
        parent: None,
    });
    stats.root_bound = world_state.state.lb;
    debug!("Root node state {:?}", world_state.state);
    let mut target_state = world_state.clone();
    let mut queue_by_lb: BinaryHeap<Reverse<Rc<Node>>> = Default::default();
    let mut best_state: Option<Rc<Node>> = None;
    let mut node_buf: Vec<Rc<Node>> = Vec::new();

    loop {



        // Terminate timeout
        if start_time.elapsed() > timeout {
            return (stats, None);
        }

        // Terminate when lb >= ub
        if target_state.state.lb >= best_state.as_ref().map(|s| s.state.lb).unwrap_or(i32::MAX) {
            break;
        }

        stats.best_bound = stats.best_bound.max(target_state.state.lb);

        // Bring the world state to the target state:
        //
        // Pop constraints until we reach the common ancestor,
        // then push constraints until we get down to the `node`.
        trace!(
            "going to node d={} lb={}",
            target_state.depth,
            target_state.state.lb
        );
        {
            let mut common_ancestor = &target_state;
            while !Rc::ptr_eq(&world_state, common_ancestor) {
                if common_ancestor.depth > world_state.depth {
                    node_buf.push(common_ancestor.clone());
                    common_ancestor = &common_ancestor.parent.as_ref().unwrap().0;
                } else {
                    world.pop();
                    world_state = world_state.parent.as_ref().unwrap().0.clone();
                }
            }
            for n in node_buf.drain(..).rev() {
                assert!(world.push(n.parent.as_ref().unwrap().1));
                world_state = n;
            }
        }
        assert!(Rc::ptr_eq(&target_state, &world_state));
        stats.n_nodes_solved += 1;

        // Generate new nodes based on the target node's precomputed branching choices.
        //
        let ub = best_state.as_ref().map(|n| n.state.lb).unwrap_or(i32::MAX);
        let mut new_nodes: TinyVec<[Rc<Node>; 2]> = match target_state.state.branching.as_ref() {
            None => {
                if target_state.state.lb < ub {
                    info!("NEW BEST {}", target_state.state.lb);
                    stats.best_value = target_state.state.lb;
                    best_state = Some(target_state);
                }
                Default::default()
            }
            Some(bs) => {
                let mut new_nodes: TinyVec<[Rc<Node>; 2]> = Default::default();
                for b in bs.iter() {
                    assert!(world.push(*b));
                    let state = world.mk_state(settings, target_state.state.lb, ub);
                    stats.n_states_generated += 1;
                    if let Some(state) = state {
                        if !state
                            .branching
                            .as_ref()
                            .map(|x| x.is_empty())
                            .unwrap_or(false)
                        {
                            stats.n_nodes_generated += 1;
                            let node = Rc::new(Node {
                                state,
                                depth: target_state.depth + 1,
                                parent: Some((target_state.clone(), *b)),
                            });
                            stats.max_depth = stats.max_depth.max(node.depth);
                            if node.state.branching.is_none() {
                                assert!(node.state.lb < ub);
                                info!("NEW BEST {}", node.state.lb);
                                stats.best_value = target_state.state.lb;
                                best_state = Some(node);
                            } else {
                                new_nodes.push(node);
                            }
                        }
                    }
                    world.pop();
                }

                new_nodes
            }
        };

        // Find the next node to process, using the best-first queue only if necessary.
        new_nodes.sort_by_key(|n| n.state.lb);
        if !new_nodes.is_empty()
            && new_nodes.first().map(|n| n.state.lb).unwrap_or(i32::MAX)
                <= queue_by_lb.peek().map(|q| q.0.state.lb).unwrap_or(i32::MAX)
        {
            target_state = new_nodes.remove(0);
        } else {
            match queue_by_lb.pop() {
                None => {
                    debug!("queue empty");
                    break;
                }
                Some(Reverse(n)) => {
                    if n.state.lb >= ub {
                        debug!("ub reached");
                        break;
                    }
                    target_state = n;
                }
            }
        }
        queue_by_lb.extend(new_nodes.into_iter().map(Reverse));
    }

    stats.solution_depth = best_state.as_ref().map(|n| n.depth).unwrap_or(u32::MAX);
    
    if let Some(best) = best_state.as_ref() {
        stats.best_bound = stats.best_bound.max(best.state.lb);
        stats.best_value = best.state.lb;
    }

    (stats, best_state.map(|n| n.state.lb))
}
