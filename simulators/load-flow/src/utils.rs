use crate::graph::electric_graph::Graph;
use crate::graph::electric_graph::UndirectedGraph;
use crate::units::voltage::Voltage;
use nalgebra::{Complex, DMatrix};
use num_complex::ComplexFloat;

/// Constructs the admittance matrix (Y-Bus) for an undirected graph representing a power network.
///
/// This function computes the admittance matrix based on the network's nodes and edges, where each element represents
/// the admittance between two buses. The diagonal elements represent the sum of admittances for each node.
///
/// # Arguments
/// * `graph` - A mutable reference to an `UndirectedGraph` representing the power network.
///
/// # Returns
/// `DMatrix<Complex<f64>>` - A square matrix of complex numbers, where the size is the number of nodes in the graph.
/// Each element at (i, j) represents the admittance between nodes i and j.
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

/// Calculates the Haversine distance between two points on the Earth's surface.
///
/// This function uses the Haversine formula to determine the distance between two geographic locations,
/// given their latitude and longitude coordinates. The result is expressed in meters.
///
/// # Arguments
/// * `lat1` - Latitude of the first point in degrees.
/// * `lon1` - Longitude of the first point in degrees.
/// * `lat2` - Latitude of the second point in degrees.
/// * `lon2` - Longitude of the second point in degrees.
///
/// # Returns
/// `f64` - The distance between the two points in meters.
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

/// Ensures the convergence of a power network simulation by checking and correcting node voltages.
///
/// This function checks all nodes in the graph to see if any voltage values are NaN. If a NaN voltage is detected,
/// it replaces the node's voltage with a default voltage value, which helps ensure algorithm convergence.
///
/// # Arguments
/// * `graph` - A mutable reference to an `UndirectedGraph` representing the power network.
///
/// # Returns
/// `bool` - Returns `true` if all voltages are valid (non-NaN and finite), and `false` otherwise.
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
