use serde::Deserialize;
use tinyvec::TinyVec;

#[derive(Deserialize)]
#[derive(Default ,Clone, Copy)]
pub struct Edge {
    pub src :u32,
    pub tgt: u32,
    pub weight :i32,
}

#[derive(Deserialize)]
pub struct Node {
    pub lb :i32,
    pub ub :i32,
    pub coeff :u32,
    pub threshold :i32,
}

#[derive(Deserialize)]
pub struct DisjunctiveGraph {
    pub nodes :Vec<Node>,
    pub edge_sets: Vec<TinyVec<[Edge; 2]>>,
}
