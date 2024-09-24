use problem::DisjunctiveGraph;

pub mod bnb;
pub mod longestpaths;
pub mod problem;
pub mod wdg;
pub mod world;

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let problem: DisjunctiveGraph =
        serde_json::from_str(&std::fs::read_to_string(filename).unwrap()).unwrap();
    bnb::solve(&problem);
}
