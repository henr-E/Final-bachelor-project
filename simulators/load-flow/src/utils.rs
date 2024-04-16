use crate::graph::electric_graph::Graph;
use crate::graph::electric_graph::UndirectedGraph;
use nalgebra::{Complex, DMatrix};
use num_complex::ComplexFloat;

pub fn admittance_matrix(graph: &mut UndirectedGraph) -> DMatrix<Complex<f64>> {
    let size = graph.node_count();

    // Create an n x n matrix filled with zeros.
    let mut y_bus = DMatrix::from_element(size, size, Complex::new(0.0, 0.0));

    for from in graph.nodes() {
        for to in graph.neighbors(from) {
            if let Some(edge) = graph.edge(from, to) {
                let i = from % size;
                let j = to % size;
                let y_ij = edge.impedance(graph.z_base()).to_complex().recip();

                y_bus[(i, j)] = -y_ij;
                y_bus[(i, i)] += y_ij;
            }
        }
    }
    y_bus
}
pub fn print_matrix(y_bus: DMatrix<Complex<f64>>) {
    println!("Admittance Matrix:");
    for i in 0..y_bus.nrows() {
        for j in 0..y_bus.ncols() {
            print!("{:>10}", y_bus[(i, j)]);
        }
        println!();
    }
}
