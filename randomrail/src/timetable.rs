use std::collections::HashMap;

use ordered_float::OrderedFloat;
use rand::{thread_rng, Rng};
use serde::Serialize;

use crate::infrastructure::Infrastructure;

#[derive(Serialize)]
pub struct Timetable {
    trains: Vec<Train>,
}

#[derive(Serialize)]
pub struct Train {
    operations: Vec<Operation>,
}

#[derive(Serialize)]
pub struct Operation {
    resource: usize,
    forward: bool,
    min_duration: f64,
    time: f64,
}

const SPEED :f64 = 60.0 /* km/h */ / 3600.0 /* km/sec */;
const SPAN: f64 = 6. * 3600.; // 6 hours
const SLACK: f64 = 1.04; // 4% running time slack
const MAIN_PERIOD: f64 = 10.0*60.0; // bottleneck frequency

pub fn generate_timetable(
    infrastructure: &Infrastructure,
    services: &Vec<Vec<Vec<usize>>>,
    bottleneck: usize,
) -> Timetable {
    let mut trains = Vec::new();
    let mut rng = thread_rng();
    let (group_freq, ingroup_freq) = generate_grouped_frequencies(services, &mut rng);

    for g in services.iter() {
        println!("group:");
        for s in g.iter() {
            println!(" service {:?}", s);
        }
    }
    println!("freq {:?}", group_freq);
    println!("freq {:?}", ingroup_freq);

    let mut t = 0.;
    let mut group_counter = services.iter().map(|x| 0).collect::<Vec<_>>();
    let mut ingroup_counter = services
        .iter()
        .map(|x| x.iter().map(|_| 0).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    while t < SPAN {
        // Choose a service group.
        let g = (0..services.len())
            .min_by_key(|i| {
                (
                    OrderedFloat(group_counter[*i] as f64 / group_freq[*i] as f64),
                    -group_freq[*i],
                )
            })
            .unwrap();

        // Choose a service in the group
        let s = (0..services[g].len())
            .min_by_key(|i| {
                (
                    OrderedFloat(ingroup_counter[g][*i] as f64 / ingroup_freq[g][*i] as f64),
                    -ingroup_freq[g][*i],
                )
            })
            .unwrap();

        // dispatch
        for up in [true, false] {
            trains.push(generate_train(
                infrastructure,
                &services[g][s],
                SPEED,
                SLACK,
                up,
                bottleneck,
                t,
            ));
        }

        // increase the counters
        group_counter[g] += 1;
        ingroup_counter[g][s] += 1;

        t += MAIN_PERIOD;
    }

    Timetable { trains }
}

fn generate_grouped_frequencies(
    services: &Vec<Vec<Vec<usize>>>,
    rng: &mut rand::prelude::ThreadRng,
) -> (Vec<i32>, Vec<Vec<i32>>) {
    let group_freq = services
        .iter()
        .map(|_| rng.gen_range(3..=10))
        .collect::<Vec<_>>();

    let ingroup_freq = services
        .iter()
        .map(|ss| ss.iter().map(|_| rng.gen_range(3..=10)).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    (group_freq, ingroup_freq)
}

pub fn generate_train(
    infrastructure: &Infrastructure,
    line: &[usize],
    speed: f64,
    slack: f64,
    up: bool,
    ref_resource: usize,
    ref_time: f64,
) -> Train {
    let mut up_resources: HashMap<usize, Vec<usize>> = Default::default();
    let mut down_resources: HashMap<usize, Vec<usize>> = Default::default();

    for (res_idx, res) in infrastructure.resources.iter().enumerate() {
        up_resources.entry(res.node_lo).or_default().push(res_idx);
        down_resources.entry(res.node_hi).or_default().push(res_idx);
    }

    let line: Box<dyn Iterator<Item = _>> = if up {
        Box::new(line.iter())
    } else {
        Box::new(line.iter().rev())
    };

    let mut time = 0.0;
    let mut operations: Vec<Operation> = line
        .filter_map(|n| {
            if up { &up_resources } else { &down_resources }
                .get(n)
                .map(|x| x[0])
        })
        .map(|resource_idx| {
            let min_duration = infrastructure.resources[resource_idx].length / speed;
            let op = Operation {
                forward: up,
                resource: resource_idx,
                min_duration,
                time,
            };
            let dt = min_duration * slack;
            time += dt;
            op
        })
        .collect();

    let ref_op_time = operations
        .iter()
        .find(|x| x.resource == ref_resource)
        .unwrap()
        .time;
    for op in operations.iter_mut() {
        op.time += ref_time - ref_op_time;
    }

    let length = operations.iter().map(|x| infrastructure.resources[x.resource].length ).sum::<f64>();

    let realspeed = 
    length / (operations.last().unwrap().time - operations[0].time);

    println!("train len {} speed {} realspeed {} up {} reftime {} total time {}",length, speed,realspeed, up,operations
    .iter()
    .find(|x| x.resource == ref_resource)
    .unwrap()
    .time, operations.last().unwrap().time - operations[0].time);

    Train { operations }
}

pub fn generate_trains(
    infrastructure: &Infrastructure,
    line: &[usize],
    speed: f64,
    span: f64,
    slack: f64,
    normalized_frequency: f64,
) -> Vec<Train> {
    let mut trains = Vec::new();

    let mut up_resources: HashMap<usize, Vec<usize>> = Default::default();
    let mut down_resources: HashMap<usize, Vec<usize>> = Default::default();

    for (res_idx, res) in infrastructure.resources.iter().enumerate() {
        up_resources.entry(res.node_lo).or_default().push(res_idx);
        down_resources.entry(res.node_hi).or_default().push(res_idx);
    }

    let oneway_actual_minimum_running_time = line
        .iter()
        .enumerate()
        .filter_map(|(line_idx, node_idx)| {
            let ress = up_resources.get(node_idx);
            // Only the last node should not have an up-going resource.
            assert!(ress.is_some() || line_idx + 1 == line.len());
            ress.map(|ress| infrastructure.resources[ress[0]].length / speed)
        })
        .sum::<f64>();

    let oneway = oneway_actual_minimum_running_time * (1.0 + slack);

    let period = 2. * oneway / normalized_frequency;
    // when does the same train start again?
    let next_period = (2. * oneway / period).ceil() * period;
    for (start, up) in [(0.0, true), (0.5 * next_period, false)] {
        for i in 0..=((span / period).round() as i32) {
            let mut time = start + i as f64 * period;
            let line: Box<dyn Iterator<Item = _>> = if up {
                Box::new(line.iter())
            } else {
                Box::new(line.iter().rev())
            };
            let operations = line
                .filter_map(|n| {
                    if up { &up_resources } else { &down_resources }
                        .get(n)
                        .map(|x| x[0])
                })
                .map(|x| {
                    let min_duration = infrastructure.resources[x].length / speed;
                    let op = Operation {
                        forward: up,
                        resource: x,
                        min_duration,
                        time,
                    };
                    let dt = min_duration * (1.0 + slack);
                    time += dt;
                    op
                })
                .collect();
            trains.push(Train { operations });
        }
    }

    trains
}
