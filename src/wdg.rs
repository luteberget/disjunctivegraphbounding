pub struct WdgEdge {
    pub partition: u32,
    pub d_cost: i32,
}

#[derive(Default)]
pub struct WdgSolverBinaryMIP {
    edge_pairs: Vec<(WdgEdge, WdgEdge)>,
}

impl WdgSolverBinaryMIP {
    pub fn clear(&mut self) {
        self.edge_pairs.clear();
    }

    pub fn add_edge_pair(&mut self, wdg_edge1: WdgEdge, wdg_edge2: WdgEdge) {
        self.edge_pairs.push((wdg_edge1, wdg_edge2));
    }

    pub fn solve(&mut self, n_partitions: usize) -> i32 {
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
        for (edge_pair_idx, (e1, e2)) in self.edge_pairs.iter().enumerate() {
            problem.add_row(
                0.0..,
                [
                    (partition_cost[e1.partition as usize], 1.0),
                    (edge_pair_vars[edge_pair_idx], -e1.d_cost as f64),
                ],
            );

            problem.add_row(
                e2.d_cost..,
                [
                    (partition_cost[e1.partition as usize], 1.0),
                    (edge_pair_vars[edge_pair_idx], e2.d_cost as f64),
                ],
            );
        }

        let solved = problem.optimise(highs::Sense::Minimise).solve();
        assert_eq!(solved.status(), highs::HighsModelStatus::Optimal);
        let solution = solved.get_solution();
        solution
            .columns()
            .iter()
            .take(partition_cost.len())
            .sum::<f64>()
            .round() as i32
    }
}
