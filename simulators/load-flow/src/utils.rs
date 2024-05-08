use crate::graph::electric_graph::Graph;
use crate::graph::electric_graph::UndirectedGraph;
use crate::units::voltage::Voltage;
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
use std::f64::consts::PI;

pub(crate) fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    // Earth's mean radius in meters
    const EARTH_RADIUS: f64 = 6_371_000.0;

    // Convert degrees to radians
    let lat1_rad = lat1 * PI / 180.0;
    let lon1_rad = lon1 * PI / 180.0;
    let lat2_rad = lat2 * PI / 180.0;
    let lon2_rad = lon2 * PI / 180.0;

    // Calculate the differences in latitude and longitude
    let delta_lat = lat2_rad - lat1_rad;
    let delta_lon = lon2_rad - lon1_rad;

    // Haversine formula
    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    // Calculate the distance in meters
    EARTH_RADIUS * c.abs()
}

/// Check if the voltage of any node is NaN and replace it with a default value.
/// If the voltage is NaN, the algorithm will not converge.
pub fn check_convergence(graph: &mut UndirectedGraph) -> bool {
    let mut convergence = true;
    for node in graph.nodes() {
        if let Some(node) = graph.node(node) {
            if node.voltage().amplitude.is_nan()
                || node.voltage().angle.is_nan()
                || !(node.voltage().to_complex().re.is_finite()
                    && node.voltage().to_complex().im.is_finite())
            {
                convergence = false;
                let mut new_node = *node;
                new_node.set_voltage(Voltage::new(1.0, 0.0));
                graph.add_node(new_node.id(), new_node);
            }
        }
    }
    convergence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        let lat1 = 52.2296756;
        let lon1 = 21.0122287;
        let lat2 = 52.406374;
        let lon2 = 16.9251681;
        let distance = haversine_distance(lat1, lon1, lat2, lon2);
        assert_eq!(distance, 278458.1750754196);
    }
}
