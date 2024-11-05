use log::trace;
use std::collections::{HashMap, HashSet};
use tinyvec::TinyVec;

#[derive(Debug, Default, Clone, Copy)]
pub struct WdgEdge {
    pub partition: u32,
    pub d_cost: i32,
}

type WdgEdgeSet = TinyVec<[WdgEdge; 8]>;

#[derive(Default)]
pub struct WdgSolverBinaryMIP {
    disjunctions: Vec<(WdgEdgeSet, WdgEdgeSet)>,
    simple_pair_disjunctions: HashMap<(u32, u32), Vec<usize>>,
    dominated_disjunctions: HashSet<usize>,
}

fn label_dominates(ds: &[(WdgEdgeSet, WdgEdgeSet)], a: usize, b: usize) -> bool {
    ds[a].0[0].d_cost >= ds[b].0[0].d_cost && ds[a].1[0].d_cost >= ds[b].1[0].d_cost
}

fn remove_dominated(
    ds: &[(WdgEdgeSet, WdgEdgeSet)],
    front: &mut Vec<usize>,
    new_elem: usize,
    mut set_dominated: impl FnMut(usize),
) -> bool {
    for (index, elem) in front.iter().enumerate() {
        if label_dominates(ds, *elem, new_elem) {
            if index > 0 {
                front.swap(index, index - 1);
            }
            return false;
        } else if label_dominates(ds, new_elem, *elem) {
            set_dominated(front.swap_remove(index));
            {
                let mut index = index;
                while index < front.len() {
                    if label_dominates(ds, new_elem, front[index]) {
                        set_dominated(front.swap_remove(index));
                    } else {
                        index += 1;
                    }
                }
            }

            return true;
        }
    }
    true
}

impl WdgSolverBinaryMIP {
    pub fn clear(&mut self) {
        self.disjunctions.clear();
        self.simple_pair_disjunctions.clear();
        self.dominated_disjunctions.clear();
    }

    pub fn add_disjunction(&mut self, alt1: &TinyVec<[WdgEdge; 8]>, alt2: &TinyVec<[WdgEdge; 8]>) {
        // Is it a simple pair?
        let simple_pair = alt1.len() == 1 && alt2.len() == 1;
        if simple_pair && alt1[0].partition > alt2[0].partition {
            self.add_disjunction(alt2, alt1);
            return;
        }

        let new_elem = self.disjunctions.len();
        self.disjunctions.push((alt1.clone(), alt2.clone()));

        if simple_pair {
            assert!(alt1[0].partition != alt2[0].partition);
            let (p1, p2) = (alt1[0].partition, alt2[0].partition);

            let disjunction_ref_list = self.simple_pair_disjunctions.entry((p1, p2)).or_default();

            let dom = |d| {
                self.dominated_disjunctions.insert(d);
            };
            if remove_dominated(&self.disjunctions, disjunction_ref_list, new_elem, dom) {
                disjunction_ref_list.push(new_elem);
            } else {
                // Undo the insertion -- this new disjunction was dominated.
                self.disjunctions.pop();
            }
        }
    }

    pub fn solve(&mut self, n_partitions: usize) -> i32 {
        if self.disjunctions.is_empty() {
            return 0;
        }

        trace!("wdg_solve:");
        for x in self.disjunctions.iter() {
            trace!("  -  {:?}", x);
        }

        let mut problem = highs::RowProblem::new();

        // Cost Vars
        let partition_cost: Vec<highs::Col> = (0..n_partitions)
            .map(|_| problem.add_column(1.0, 0..))
            .collect();

        // Constraints

        for (d_idx, (alt1, alt2)) in self.disjunctions.iter().enumerate() {
            if self.dominated_disjunctions.contains(&d_idx) {
                continue;
            }

            let var: highs::Col = problem.add_integer_column(0.0, 0..1);

            // alt1 constraints
            for WdgEdge { partition, d_cost } in alt1.iter() {
                problem.add_row(
                    0.0..,
                    [
                        (partition_cost[*partition as usize], 1.0),
                        (var, -*d_cost as f64),
                    ],
                );
            }

            // alt2 constraints
            for WdgEdge { partition, d_cost } in alt2.iter() {
                problem.add_row(
                    (*d_cost as f64)..,
                    [
                        (partition_cost[*partition as usize], 1.0),
                        (var, *d_cost as f64),
                    ],
                );
            }
        }

        let mut model = problem.optimise(highs::Sense::Minimise);
        model.set_option("output_flag", true);
        let solved = model.solve();
        assert_eq!(solved.status(), highs::HighsModelStatus::Optimal);
        let solution = solved.get_solution();
        let value = solution
            .columns()
            .iter()
            .take(partition_cost.len())
            .sum::<f64>()
            .round() as i32;

        trace!("HIGHS result {:?} value {}", solved.status(), value);

        value
    }
}
