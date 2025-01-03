use std::{collections::HashMap, f64::consts::TAU};

use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::Serialize;

#[derive(Serialize, Clone, Copy)]
pub struct Vec2 {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
pub struct Node {
    location: Vec2,
}

#[derive(Serialize)]
pub struct Resource {
    length: f64,
    node_a: usize,
    node_b: usize,
    line_segments: Vec<Vec2>,
}

#[derive(Serialize)]
pub struct Infrastructure {
    nodes: Vec<Node>,
    resources: Vec<Resource>,
}

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

pub fn add_simple_line(
    infrastructure: &mut Infrastructure,
    mut track_start_node: usize,
    direction: Vec2,
) -> usize {
    let mut rng = rand::thread_rng();
    let min_stations = 10;
    let max_stations = 30;

    let station_tracks = [1, 2, 2, 2, 2, 2, 2, 2, 3, 4, 5];

    let avg_dist = 10.0f64;
    let min_dist = 1.0;
    let dist_stddev = 8.0;
    let dist = Normal::new(avg_dist, dist_stddev).unwrap();

    let num_stations = rng.gen_range(min_stations..=max_stations);
    println!("Line: generating {} stations", num_stations);
    let mut total_length = 0.0;
    for _ in 0..num_stations {
        let track_length = dist.sample(&mut rng).max(min_dist);
        total_length += track_length;
        let start_loc = infrastructure.nodes[track_start_node].location;
        let track_end_loc = Vec2 {
            x: start_loc.x + direction.x,
            y: start_loc.y + direction.y,
        };

        let track_end_node = infrastructure.nodes.len();
        infrastructure.nodes.push(Node {
            location: track_end_loc,
        });

        infrastructure.resources.push(Resource {
            node_a: track_start_node,
            node_b: track_end_node,
            line_segments: vec![],
            length: track_length,
        });

        let n_station_tracks = station_tracks[rng.gen_range(0..station_tracks.len())];

        let station_end_node =
            add_station(infrastructure, n_station_tracks, track_end_node, direction);

        track_start_node = station_end_node;
    }

    println!("  total length {total_length:.2}");

    track_start_node
}

fn add_station(
    infrastructure: &mut Infrastructure,
    n_station_tracks: i32,
    start_node: usize,
    direction: Vec2,
) -> usize {
    const CORNER_DIST_PER_TRACK: f64 = 0.025;
    let station_viz_length = CORNER_DIST_PER_TRACK * 2.0 * (n_station_tracks as f64 + 2.0);

    let track_end_loc = infrastructure.nodes[start_node].location;
    let station_end_loc = Vec2 {
        x: track_end_loc.x + station_viz_length * direction.x,
        y: track_end_loc.y + station_viz_length * direction.y,
    };

    let station_end_node = infrastructure.nodes.len();
    infrastructure.nodes.push(Node {
        location: station_end_loc,
    });

    for track in 0..n_station_tracks {
        let corner_dist = track as f64 * CORNER_DIST_PER_TRACK;

        let corner1 = Vec2 {
            x: track_end_loc.x + corner_dist * direction.x - corner_dist * direction.y,
            y: track_end_loc.y + corner_dist * direction.y + corner_dist * direction.x,
        };

        let c2c = station_viz_length - 2.0 * corner_dist;

        let corner2 = Vec2 {
            x: corner1.x + c2c * direction.x,
            y: corner1.y + c2c * direction.y,
        };

        infrastructure.resources.push(Resource {
            node_a: start_node,
            node_b: station_end_node,
            line_segments: vec![corner1, corner2],
            length: 0.2,
        });
    }





    
    station_end_node
}


struct TrackEdge {
    source_node :StationNodeRef,
    target_node :StationNodeRef,
}

type StationNodeRef = usize;
struct TrackEdgeRef {
    index :usize,
    forward :bool,
}

struct LineSpec {
    node :LineSpecTreeNode,
    frequency :f64,
}

enum LineSpecTreeNode {
    Group(Vec<LineSpec>),
    LineSpec(Vec<TrackEdgeRef>)
}

fn round_robin(lines :&[LineSpec], span :f64) {
    let total_freq = lines.iter().map(|x| x.frequency).sum::<f64>();
    let period = 1.0 / total_freq;
    let mut t : f64 = 0.;

    let mut queue = (0..lines.len()).collect::<Vec<_>>();
    while t < span {
        // Sort the queue by (1) virtual time of next train
        //                   (2) absolute distance 
        //                   (3) frequency


        t += period;
    }
}

pub fn main_old() {
    let mut infrastructure = Infrastructure {
        nodes: Default::default(),
        resources: Default::default(),
    };
    infrastructure.nodes.push(Node {
        location: Vec2 { x: 0., y: 0. },
    });

    let n_lines = 3;
    let mut lines: Vec<Vec<usize>> = Vec::new();

    let main_direction = Vec2 { x: 1.0, y: 0.0 };
    let main_station_west = 0;
    let main_station_east = add_station(&mut infrastructure, 20, main_station_west, main_direction);

    for (const_angle, start_node) in [(0.0, main_station_east), (0.5, main_station_west)] {
        for i in 0..n_lines {
            let range = 1. / 7.;
            let angle =
                (const_angle + -range + 2. * range * (i as f64 / (n_lines - 1).max(1) as f64))
                    * TAU;
            println!("angle1 {}", angle);
            let direction = Vec2 {
                x: angle.cos(),
                y: -angle.sin(),
            };
            let start_idx = infrastructure.nodes.len();
            let line_end_node = add_simple_line(&mut infrastructure, start_node, direction);
            let mut line = Vec::new();
            line.push(start_node);
            line.extend(start_idx..=line_end_node);
            lines.push(line);
        }
    }

    std::fs::write("i1.json", serde_json::to_string(&infrastructure).unwrap()).unwrap();

    // TIMETABLE
    let mut timetable = Timetable { trains: Vec::new() };
    for line in &lines {
        const SPEED :f64 = 60.0 /* km/h */ / 3600.0 /* km/sec */;
        const SPAN: f64 = 6. * 3600.; // 6 hours
        const SLACK: f64 = 1.04; // 4% running time slack
        let freq = rand::thread_rng().gen_range(1.0..=4.0);

        println!("line freq {freq}");

        let line_trains = generate_trains(&infrastructure, line, SPEED, SPAN, SLACK, freq);
        timetable.trains.extend(line_trains.into_iter());
    }

    std::fs::write("tt1.json", serde_json::to_string(&timetable).unwrap()).unwrap();
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
        up_resources.entry(res.node_a).or_default().push(res_idx);
        down_resources.entry(res.node_b).or_default().push(res_idx);
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
