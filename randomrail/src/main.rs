use std::f64::consts::TAU;

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
    node_a: usize,
    node_b: usize,
    line_segments: Vec<Vec2>,
}

#[derive(Serialize)]
pub struct Infrastructure {
    nodes: Vec<Node>,
    resources: Vec<Resource>,
}

pub fn add_simple_line(
    infrastructure: &mut Infrastructure,
    mut track_start_node: usize,
    direction: Vec2,
) -> usize {
    let mut rng = rand::thread_rng();
    let min_stations = 10;
    let max_stations = 30;

    let station_tracks = [1, 2, 2, 2, 2, 3, 4, 5];

    let avg_dist = 10.0f64;
    let min_dist = 1.0;
    let dist_stddev = 8.0;
    let dist = Normal::new(avg_dist, dist_stddev).unwrap();

    let station_length = 0.2;

    let num_stations = rng.gen_range(min_stations..=max_stations);
    for _ in 0..num_stations {
        let track_length = dist.sample(&mut rng).max(min_dist);
        let start_loc = infrastructure.nodes[track_start_node].location;
        let track_end_loc = Vec2 {
            x: start_loc.x + track_length * direction.x,
            y: start_loc.y + track_length * direction.y,
        };
        let station_end_loc = Vec2 {
            x: track_end_loc.x + station_length * direction.x,
            y: track_end_loc.y + station_length * direction.y,
        };

        let track_end_node = infrastructure.nodes.len();
        infrastructure.nodes.push(Node {
            location: track_end_loc,
        });

        infrastructure.resources.push(Resource {
            node_a: track_start_node,
            node_b: track_end_node,
            line_segments: vec![],
        });

        let station_end_node = infrastructure.nodes.len();
        infrastructure.nodes.push(Node {
            location: station_end_loc,
        });

        let n_station_tracks = station_tracks[rng.gen_range(0..station_tracks.len())];
        for track in 0..n_station_tracks {
            let corner_dist = track as f64 * 0.025;

            let corner1 = Vec2 {
                x: track_end_loc.x + corner_dist * direction.x - corner_dist * direction.y,
                y: track_end_loc.y + corner_dist * direction.y + corner_dist * direction.x,
            };

            let c2c = station_length - 2.0 * corner_dist;

            let corner2 = Vec2 {
                x: corner1.x + c2c * direction.x,
                y: corner1.y + c2c * direction.x,
            };

            infrastructure.resources.push(Resource {
                node_a: track_end_node,
                node_b: station_end_node,
                line_segments: vec![corner1, corner2],
            });
        }

        track_start_node = station_end_node;
    }

    track_start_node
}

fn main() {
    let mut infrastructure = Infrastructure {
        nodes: Default::default(),
        resources: Default::default(),
    };
    infrastructure.nodes.push(Node {
        location: Vec2 { x: 0., y: 0. },
    });

    let n_lines = 1;
    let mut lines: Vec<(usize, usize)> = Vec::new();
    let center_node = 0;

    for i in 0..n_lines {
        let angle: f64 = (i as f64 / n_lines as f64) * TAU;
        let direction = Vec2 {
            x: angle.cos(),
            y: -angle.sin(),
        };

        let line_end_node = add_simple_line(&mut infrastructure, 0, direction);
        lines.push((center_node, line_end_node));
    }

    std::fs::write("l1.json", serde_json::to_string(&infrastructure).unwrap()).unwrap();
}
