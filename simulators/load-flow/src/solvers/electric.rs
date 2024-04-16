use crate::graph::graph::Graph;

pub trait Solver {
    fn solve(self, graph: Graph, max_iterations: usize, tolerance: f64)
        -> Result<(), &'static str>;
}

