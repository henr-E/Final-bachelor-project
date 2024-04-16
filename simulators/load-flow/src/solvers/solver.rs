use crate::graph::electric_graph::UndirectedGraph;

pub trait Solver {
    fn solve(
        self,
        graph: &mut UndirectedGraph,
        max_iterations: usize,
        tolerance: f64,
    ) -> Result<(), &'static str>;
}
