use std::{f64::consts::TAU, hash::DefaultHasher};

use rand::{
    seq::{self, SliceRandom},
    Rng,
};
use rand_distr::{Distribution, Normal};
use serde::Serialize;

#[derive(Serialize, Clone, Copy)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize)]
pub struct Node {
    pub location: Vec2,
}

#[derive(Serialize)]
pub struct Resource {
    pub length: f64,
    pub node_lo: usize,
    pub node_hi: usize,
    pub restype: ResourceType,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ResourceType {
    SingleTrack,
    DoubleTrack,
    Station { capacity: usize },
}

#[derive(Serialize)]
pub struct Infrastructure {
    pub nodes: Vec<Node>,
    pub resources: Vec<Resource>,
}

pub fn generate_infrastructure() -> (Infrastructure, Vec<Vec<Vec<usize>>>, usize) {
    let mut rng = rand::thread_rng();
    let mut infrastructure = Infrastructure {
        nodes: Default::default(),
        resources: Default::default(),
    };
    infrastructure.nodes.push(Node {
        location: Vec2 { x: 0., y: 0. },
    });
    infrastructure.nodes.push(Node {
        location: Vec2 { x: 1., y: 0. },
    });

    let main_station_west = 0;
    let main_station_east = 1;
    infrastructure.resources.push(Resource {
        length: 0.5,
        node_lo: main_station_west,
        node_hi: main_station_east,
        restype: ResourceType::Station { capacity: 20 },
    });

    // main double track line
    let mut main_line = add_simple_line(
        &mut infrastructure,
        main_station_west,
        10,
        15,
        4.0,
        true,
        false,
        Vec2 { x: -1.0, y: 0.0 },
    );
    main_line.push(main_station_east);

    println!("main line {:?}", main_line);

    let west_n_lines = 3;
    let east_n_lines = 4;

    // East lines are connected to the main_station_east
    let mut east_lines = (0..east_n_lines)
        .map(|i| {
            add_simple_line(
                &mut infrastructure,
                main_station_east,
                10,
                30,
                10.0,
                false,
                true,
                line_direction(0.0, i, east_n_lines),
            )
        })
        .collect::<Vec<_>>();

    for (i, l) in east_lines.iter().enumerate() {
        println!("east lien {} {:?}", i, l);
    }

    // West lines are connected to some stations along the main line
    let west_directions = (0..west_n_lines)
        .map(|i| line_direction(0.5, i, west_n_lines))
        .collect::<Vec<_>>();

    let is_node_lo_of_station = infrastructure
        .resources
        .iter()
        .filter(|r| matches!(r.restype, ResourceType::Station { .. }))
        .map(|r| r.node_lo)
        .collect::<std::collections::HashSet<_>>();

    let connection_points = main_line
        .iter()
        .skip(1)
        .take(main_line.len() - 3)
        .copied()
        .filter(|i| {
            // We want the node to be the node_lo of a station.
            is_node_lo_of_station.contains(i)
        })
        .collect::<Vec<_>>();

    let mut west_origin_station_idxs =
        rand::seq::index::sample(&mut rng, connection_points.len(), west_n_lines - 1).into_vec();
    west_origin_station_idxs.sort();

    let west_origin_stations = west_origin_station_idxs
        .into_iter()
        .rev()
        .map(|i| connection_points[i])
        .chain(std::iter::once(main_line[0]))
        .collect::<Vec<_>>();

    let mut dirs = west_directions;

    let west_lines = west_origin_stations
        .into_iter()
        .enumerate()
        .map(|(i, node_idx)| {
            let direction = if i % 2 == 0 {
                dirs.pop().unwrap()
            } else {
                dirs.remove(0)
            };

            add_simple_line(
                &mut infrastructure,
                node_idx,
                10,
                30,
                10.0,
                false,
                false,
                direction,
            )
        })
        .collect::<Vec<_>>();

    for (i, l) in west_lines.iter().enumerate() {
        println!("weast lien {} {:?}", i, l);
    }

    let n_west_lines = west_lines.len();
    let n_east_lines = east_lines.len();
    let groups = services_westeast_grouped(&mut rng, n_west_lines, n_east_lines);

    println!("east west {:?}", groups);

    // Expand the west/east references to sequences of nodes
    let mut grouped_services = vec![];
    for ews in groups.iter() {
        let mut group: Vec<Vec<usize>> = Vec::new();
        for (w, e) in ews {
            let west_connection = west_lines[*w].last().unwrap();
            assert!(main_line.contains(west_connection));

            let main_line_part = main_line
                .iter()
                .skip_while(|x| *x != west_connection)
                .skip(1);

            assert!(east_lines[*e][0] == *main_line.last().unwrap());

            let east_line_part = east_lines[*e].iter().skip(1);

            let service = west_lines[*w]
                .iter()
                .copied()
                .chain(main_line_part.copied())
                .chain(east_line_part.copied())
                .collect::<Vec<_>>();

            group.push(service);
        }
        grouped_services.push(group);
    }

    let bottleneck = infrastructure
        .resources
        .iter()
        .position(|r| r.node_hi == main_station_west)
        .unwrap();

    (infrastructure, grouped_services, bottleneck)
}

fn services_westeast_grouped(
    rng: &mut rand::prelude::ThreadRng,
    n_west_lines: usize,
    n_east_lines: usize,
) -> Vec<Vec<(usize, usize)>> {
    let sharing_groups_characteristic_fn: Box<dyn Fn(usize, usize) -> usize> =
        if n_east_lines >= n_west_lines {
            Box::new(|a, _| a)
        } else {
            Box::new(|_, b| b)
        };

    let mut groups: std::collections::HashMap<usize, Vec<(usize, usize)>> = Default::default();

    let west_shuffle = seq::index::sample(rng, n_west_lines, n_west_lines).into_vec();
    let east_shuffle = seq::index::sample(rng, n_east_lines, n_east_lines).into_vec();

    for i in 0..n_east_lines.max(n_west_lines) {
        let e = east_shuffle[i % east_shuffle.len()];
        let w = west_shuffle[i % west_shuffle.len()];
        groups
            .entry(sharing_groups_characteristic_fn(w, e))
            .or_default()
            .push((w, e));
    }

    groups.into_values().collect()
}

#[allow(clippy::too_many_arguments)]
pub fn add_simple_line(
    infrastructure: &mut Infrastructure,
    mut track_start_node: usize,
    min_stations: usize,
    max_stations: usize,
    avg_dist: f64,
    double_track: bool,
    forward: bool,
    direction: Vec2,
) -> Vec<usize> {
    let mut line = vec![track_start_node];
    let mut rng = rand::thread_rng();

    let station_tracks = if double_track {
        vec![4]
    } else {
        vec![1, 2, 2, 2, 2, 2, 2, 2, 3, 4, 5]
    };

    // let avg_dist = 10.0f64;
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
        line.push(track_end_node);
        infrastructure.nodes.push(Node {
            location: track_end_loc,
        });

        infrastructure.resources.push(Resource {
            node_lo: if forward {
                track_start_node
            } else {
                track_end_node
            },
            node_hi: if forward {
                track_end_node
            } else {
                track_start_node
            },
            length: track_length,
            restype: if double_track {
                ResourceType::DoubleTrack
            } else {
                ResourceType::SingleTrack
            },
        });

        let n_station_tracks = station_tracks[rng.gen_range(0..station_tracks.len())];

        let station_viz_length = 0.2;
        let station_end_loc = Vec2 {
            x: track_end_loc.x + station_viz_length * direction.x,
            y: track_end_loc.y + station_viz_length * direction.y,
        };

        let station_end_node = infrastructure.nodes.len();
        line.push(station_end_node);
        infrastructure.nodes.push(Node {
            location: station_end_loc,
        });

        infrastructure.resources.push(Resource {
            node_lo: if forward {
                track_end_node
            } else {
                station_end_node
            },
            node_hi: if forward {
                station_end_node
            } else {
                track_end_node
            },
            length: 0.2,
            restype: ResourceType::Station {
                capacity: n_station_tracks,
            },
        });

        track_start_node = station_end_node;
    }

    println!("  total length {total_length:.2}");

    if !forward {
        line.reverse();
    }
    line
}

fn line_direction(const_angle: f64, i: usize, n_lines: usize) -> Vec2 {
    let (i, n_lines) = if n_lines % 2 == 1 {
        (if i >= n_lines / 2 { i + 1 } else { i }, n_lines + 1)
    } else {
        (i, n_lines)
    };
    let range = 1. / 7.;
    let angle =
        (const_angle + -range + 2. * range * (i as f64 / (n_lines - 1).max(1) as f64)) * TAU;
    println!("angle1 {}", angle);
    let direction = Vec2 {
        x: angle.cos(),
        y: -angle.sin(),
    };
    direction
}
