use std::time::Duration;

use bnb::SolverSettings;

pub mod bnb;
pub mod longestpaths;
pub mod problem;
pub mod wdg;
pub mod world;

fn main() {
    env_logger::init();

    let mut filenames = std::fs::read_dir("./instances_easy")
        .unwrap()
        .map(|path| path.unwrap().path())
        .filter(|path| path.extension().filter(|e| *e == "json").is_some())
        .collect::<Vec<_>>();

    filenames.sort();

    let settings_set = [
        SolverSettings {
            use_strong_branching: false,
            use_wdg_bound: false,
            use_relaxed_wdg: false,
        },
        SolverSettings {
            use_strong_branching: true,
            use_wdg_bound: false,
            use_relaxed_wdg: false,
        },
        SolverSettings {
            use_strong_branching: true,
            use_wdg_bound: true,
            use_relaxed_wdg: false,
        },
        SolverSettings {
            use_strong_branching: true,
            use_wdg_bound: true,
            use_relaxed_wdg: true   ,
        },
    ];

    println!("[");
    for settings in settings_set {
        let settings_name = if !settings.use_strong_branching {
            "chronological"
        } else if !settings.use_wdg_bound {
            "strong"
        } else if !settings.use_relaxed_wdg{
            "strong+wdg"
        } else {
            "strong+wdg+relx"
        };
        for filename in filenames.iter() {

            if !filename.to_string_lossy().contains("jsp") {
                continue;
            }

            let problem: problem::DisjunctiveGraph =
                serde_json::from_str(&std::fs::read_to_string(filename).unwrap()).unwrap();

            // n_states_generated: usize,
            // n_nodes_generated: usize,
            // n_nodes_solved: usize,
            // max_depth: u32,
            // solution_depth: u32,
            // root_bound: i32,

            println!("# solving {} {} ...", filename.display(), settings_name);

            let (stats, obj) = bnb::solve(&problem, &settings, Duration::from_secs(120));
            let objective = obj.map(|x| format!("{}", x)).unwrap_or("-".to_string());
            println!(" {{ 'name': '{}', 'settings': '{}', 'objective': '{}', 'states': {}, 'nodes_generated': {}, 'nodes_solved': {}, 'max_depth': {}, 'solution_depth': {}, 'root_bound': {}, 'best_bound': {}, 'best_value': {}  }},", 
                filename.display(), 
                settings_name, 
                objective,
                stats.n_states_generated, 
                stats.n_nodes_generated, 
                stats.n_nodes_solved,
                stats.max_depth,
                stats.solution_depth,
                stats.root_bound,
                stats.best_bound,
                stats.best_value,
            );
        }
    }
    println!("]");
}
