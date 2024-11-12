type Instances = Vec<Instance>;

use serde::{Deserialize, Serialize};
use tinyvec::TinyVec;

#[derive(Deserialize,Serialize, Default, Clone, Copy, Debug)]
pub struct Edge {
    pub src: u32,
    pub tgt: u32,
    pub weight: i32,
}

#[derive(Deserialize, Serialize)]
pub struct Node {
    pub lb: i32,
    pub ub: i32,
    pub coeff: u32,
    pub threshold: i32,
}

#[derive(Deserialize,Serialize)]
pub struct DisjunctiveGraph {
    pub nodes: Vec<Node>,
    pub edge_sets: Vec<TinyVec<[Edge; 2]>>,
}

#[derive(serde::Deserialize, Debug)]
struct Instance {
    name: String,
    jobs: u32,
    machines: u32,
    optimum: Option<u32>,
    bounds: Option<Bounds>,
    path: String,
}

#[derive(serde::Deserialize, Debug)]
struct Bounds {
    upper: u32,
    lower: u32,
}

pub fn mk_disjunctive(name :&str, jobs: Vec<Vec<(usize, u32)>>) {
    let n_jobs = jobs.len();
    let n_machines = jobs[0].len();
    assert!(jobs.iter().all(|j| j.len() == n_machines));
    for job in &jobs {
        assert!(job.iter().all(|(m, _)| *m < n_machines));
    }

    let mut problem = DisjunctiveGraph {
        nodes: Default::default(),
        edge_sets: Default::default(),
    };

    let mut machine_usages: Vec<Vec<(usize, i32)>> =
        (0..n_machines).map(|_| Default::default()).collect();

    for job_ops in jobs.iter() {
        for (machine, duration) in job_ops.iter() {
            let node_idx = problem.nodes.len();
            problem.nodes.push(Node {
                lb: 0,
                ub: i32::MAX,
                coeff: 0,
                threshold: 0,
            });

            let next_node_idx = node_idx + 1;
            problem.edge_sets.push(
                std::iter::once(Edge {
                    src: node_idx as u32,
                    tgt: next_node_idx as u32,
                    weight: *duration as i32,
                })
                .collect(),
            );

            machine_usages[*machine].push((node_idx, *duration as i32));
        }

        // Finishing node
        problem.nodes.push(Node {
            lb: 0,
            ub: i32::MAX,
            coeff: 1,
            threshold: 0,
        });
    }

    // Disjunctive edges
    for usages in &machine_usages {
        for i in 0..usages.len() {
            for j in (i + 1)..usages.len() {
                let (a_in, a_dur) = usages[i];
                let (b_in, b_dur) = usages[j];

                problem.edge_sets.push(
                    vec![
                        Edge {
                            src: a_in as u32,
                            tgt: b_in as u32,
                            weight: a_dur,
                        },
                        Edge {
                            src: b_in as u32,
                            tgt: a_in as u32,
                            weight: b_dur,
                        },
                    ]
                    .into_iter()
                    .collect(),
                )
            }
        }
    }


    let filename = format!("instances/jsp_{}.json", name);
    println!("Writing {}", filename);
    std::fs::write(filename, serde_json::to_string(&problem).unwrap());

}

pub fn main() {
    let instances: Instances =
        serde_json::from_str(&std::fs::read_to_string("JSPLIB/instances.json").unwrap()).unwrap();

    for instance in instances {
        let contents = std::fs::read_to_string(&format!("JSPLIB/{}", instance.path)).unwrap();
        println!("contents {}", contents);
        let mut size: Option<(usize, usize)> = None;
        let mut jobs: Vec<Vec<(usize, u32)>> = Default::default();
        for line in contents.lines() {
            if line.trim_start().starts_with('#') {
                continue;
            }
            let split = line.split_ascii_whitespace().collect::<Vec<_>>();
            if size.is_none() {
                assert!(split.len() == 2);
                size = Some((
                    split[0].parse::<usize>().unwrap(),
                    split[1].parse::<usize>().unwrap(),
                ));
            } else {
                let (n_jobs, n_machines) = size.unwrap();
                let mut new_job = Vec::new();
                assert!(split.len() == n_machines * 2);
                for i in 0..n_machines {
                    let machine = split[2 * i].parse::<usize>().unwrap();
                    let duration = split[2 * i + 1].parse::<u32>().unwrap();

                    new_job.push((machine, duration));
                }

                jobs.push(new_job);
            }
        }

        assert!(size.unwrap().0 == jobs.len());

        // println!("jobs");
        // for j in &jobs {
        //     println!(" {:?}", j);
        // }

        mk_disjunctive(&instance.name, jobs);
    }
}
