use tinyvec::TinyVec;

#[derive(Default ,Clone, Copy)]
pub struct Edge {
    pub src :u32,
    pub tgt: u32,
    pub weight :i32,
}

pub struct Node {
    pub lb :i32,
    pub ub :i32,
    pub coeff :u32,
    pub threshold :i32,
}

pub struct DisjunctiveGraph {
    pub nodes :Vec<Node>,
    pub edge_sets: Vec<TinyVec<[Edge; 2]>>,
}
