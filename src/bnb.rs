use std::{cmp::Reverse, collections::BinaryHeap, rc::Rc};

use tinyvec::TinyVec;

use crate::{
    problem::{DisjunctiveCliquesProblem, Edge},
    world::{State, World},
};

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

pub fn solve(problem: &DisjunctiveCliquesProblem) -> Option<i32> {
    let mut world = World::new(problem)?;
    let mut world_state = Rc::new(Node {
        state: world.mk_state(i32::MAX),
        depth: 0,
        parent: None,
    });
    let mut target_state = world_state.clone();
    let mut queue_by_lb: BinaryHeap<Reverse<Rc<Node>>> = Default::default();
    let mut best_state: Option<Rc<Node>> = None;
    let mut node_buf: Vec<Rc<Node>> = Vec::new();

    loop {
        // Bring the world state to the target state:
        //
        // Pop constraints until we reach the common ancestor,
        // then push constraints until we get down to the `node`.
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

        // Generate new nodes based on the target node's precomputed branching choices.
        //
        let ub = best_state.as_ref().map(|n| n.state.lb).unwrap_or(i32::MAX);
        let mut new_nodes: TinyVec<[Rc<Node>; 2]> = match target_state.state.branching.as_ref() {
            None => {
                if target_state.state.lb < ub {
                    best_state = Some(target_state);
                }
                Default::default()
            }
            Some(bs) => {
                let mut new_nodes: TinyVec<[Rc<Node>; 2]> = Default::default();
                for b in bs.iter() {
                    assert!(world.push(*b));
                    let state = world.mk_state(ub);
                    if !state
                        .branching
                        .as_ref()
                        .map(|x| x.is_empty())
                        .unwrap_or(false)
                    {
                        let node = Rc::new(Node {
                            state,
                            depth: target_state.depth + 1,
                            parent: Some((target_state.clone(), *b)),
                        });
                        if node.state.branching.is_none() {
                            assert!(node.state.lb < ub);
                            best_state = Some(node);
                        } else {
                            new_nodes.push(node);
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
                None => break,
                Some(Reverse(n)) => target_state = n,
            }
        }
        queue_by_lb.extend(new_nodes.into_iter().map(Reverse));
    }

    best_state.map(|n| n.state.lb)
}
