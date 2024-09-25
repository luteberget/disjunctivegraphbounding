use std::collections::HashMap;
use log::trace;

#[derive(Debug)]
pub struct WdgEdge {
    pub partition: u32,
    pub d_cost: i32,
}

#[derive(Default)]
pub struct WdgSolverBinaryMIP {
    edge_pairs: HashMap<(u32, u32), Vec<(i32, i32)>>,
}

fn label_dominates(a: &(i32, i32), b: &(i32, i32)) -> bool {
    a.0 >= b.0 && a.1 >= b.1
}

fn remove_dominated(front: &mut Vec<(i32, i32)>, new_elem: &(i32, i32)) -> bool {
    for (index, elem) in front.iter().enumerate() {
        if label_dominates(elem, new_elem) {
            if index > 0 {
                front.swap(index, index - 1);
            }
            return false;
        } else if label_dominates(new_elem, elem) {
            front.swap_remove(index);
            {
                let mut index = index;
                while index < front.len() {
                    if label_dominates(new_elem, &front[index]) {
                        front.swap_remove(index);
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
        self.edge_pairs.clear();
    }

    pub fn add_edge_pair(&mut self, wdg_edge1: WdgEdge, wdg_edge2: WdgEdge) {
        if wdg_edge1.partition > wdg_edge2.partition {
            self.add_edge_pair(wdg_edge2, wdg_edge1);
        } else {
            let coeff_list = self
                .edge_pairs
                .entry((wdg_edge1.partition, wdg_edge2.partition))
                .or_default();
            let new_elem = (wdg_edge1.d_cost, wdg_edge2.d_cost);
            if remove_dominated(coeff_list, &new_elem) {
                coeff_list.push(new_elem);
            }
        }
    }

    pub fn solve(&mut self, n_partitions: usize) -> i32 {
        if self.edge_pairs.is_empty() {
            return 0;
        }
        trace!("wdg_solve:");
        for x in self.edge_pairs.iter() {
            trace!("  -  {:?}", x);
        }

        let mut problem = highs::RowProblem::new();

        // Vars
        let partition_cost: Vec<highs::Col> = (0..n_partitions)
            .map(|_| problem.add_column(1.0, 0..))
            .collect();
        let edge_pair_vars: Vec<highs::Col> = self
            .edge_pairs
            .iter()
            .map(|_| problem.add_integer_column(0.0, 0..1))
            .collect();

        // Constraints
        for (edge_pair_idx, ((p1, p2), coeffs)) in self.edge_pairs.iter().enumerate() {
            for (c1, c2) in coeffs {
                problem.add_row(
                    0.0..,
                    [
                        (partition_cost[*p1 as usize], 1.0),
                        (edge_pair_vars[edge_pair_idx], -*c1 as f64),
                    ],
                );

                problem.add_row(
                    (*c2 as f64)..,
                    [
                        (partition_cost[*p2 as usize], 1.0),
                        (edge_pair_vars[edge_pair_idx], *c2 as f64),
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
