use crate::graph::electric_graph::{Graph, UndirectedGraph};
use crate::graph::node::{BusNode, BusType};
use crate::solvers::solver::Solver;
use crate::units::voltage::Voltage;
use crate::utils::{admittance_matrix, check_convergence};
use nalgebra::{Complex, ComplexField, DMatrix};
use std::collections::HashMap;

pub struct GaussSeidel;
#[allow(dead_code)]
impl GaussSeidel {
    pub fn new() -> Self {
        GaussSeidel {}
    }
}

/// Checks for the presence of zero or invalid (NaN) diagonal elements in a given complex matrix.
///
/// This function iterates over the diagonal elements of the `y_bus` matrix, checking if any of the elements
/// are zero (both real and imaginary parts), contain NaN in either part, or have a modulus (magnitude) of zero.
///
/// # Arguments
/// * `y_bus` - A reference to a `DMatrix<Complex<f64>>` representing the admittance matrix, where each entry is a complex number.
///
/// # Returns
/// * `bool` - Returns `true` if any diagonal element is zero, contains NaN, or has a modulus of zero. Otherwise, returns `false`.
fn zero_diagonal_elements(y_bus: &DMatrix<Complex<f64>>) -> bool {
    for i in 0..y_bus.nrows() {
        let z = y_bus[(i, i)];
        // Check if the complex number is zero (both real and imaginary parts are zero)
        let is_zero = z == Complex::new(0.0, 0.0);
        // Check if either the real or imaginary part is NaN
        let has_nan = z.re.is_nan() || z.im.is_nan();
        // Check if the modulus (magnitude) of the complex number is zero
        let is_modulus_zero = z.norm() == 0.0;
        // Return true if the complex number is zero, has NaN, or has zero modulus
        if is_zero || has_nan || is_modulus_zero {
            return true;
        }
    }
    false
}

/// Captures the initial voltages of all nodes in an undirected graph, typically used for setting initial conditions before simulations.
///
/// This function traverses all the nodes in the given graph and stores their current voltages in a `HashMap` indexed by node ID.
///
/// # Arguments
/// * `graph` - A mutable reference to an `UndirectedGraph` where each node represents an electrical component or connection point.
///
/// # Returns
/// * `HashMap<usize, Voltage>` - A hashmap where the key is the node ID and the value is the voltage at that node.
fn initial_voltages(graph: &mut UndirectedGraph) -> HashMap<usize, Voltage> {
    let mut voltages: HashMap<usize, Voltage> = HashMap::new();
    for id in graph.nodes() {
        if let Some(node) = graph.node(id) {
            voltages.insert(id, node.voltage());
        }
    }
    voltages
}

/// Sets the voltage of generator nodes in the graph based on a provided mapping of node IDs to voltages.
///
/// This function iterates through all nodes in the `graph` and updates the voltage of generator nodes to the corresponding values in the `voltages` hashmap.
/// Other node types are unaffected. This is generally used after a simulation or an algorithm iteration to reset the node voltages to initial or desired states.
///
/// # Arguments
/// * `graph` - A mutable reference to an `UndirectedGraph` representing a network of electrical nodes.
/// * `voltages` - A `HashMap<usize, Voltage>` mapping node IDs to their respective voltages to be applied.
fn set_voltages(graph: &mut UndirectedGraph, voltages: HashMap<usize, Voltage>) {
    for id in graph.nodes() {
        if let Some(node) = graph.node(id) {
            let mut node_update = *node;
            if let Some(voltage) = voltages.get(&id) {
                if node.bus_type() == BusType::Generator {
                    node_update.set_voltage(Voltage::new(voltage.amplitude, node.voltage().angle));
                    graph.reset_node(node_update);
                }
            }
        }
    }
}

impl GaussSeidel {
    /// Calculates the total current flowing into a node from its connected neighbors.
    ///
    /// This function computes the sum of currents entering a node from all its neighboring nodes.
    /// The current from each neighbor is calculated as the product of the neighbor's voltage and the admittance
    /// between this node and the neighbor. The admittance values are obtained from the `y_bus` matrix, which should
    /// represent the entire network's admittance matrix.
    ///
    /// # Arguments
    /// * `graph` - A reference to an `UndirectedGraph` representing the electrical network.
    /// * `id` - The node ID for which the total current is being calculated.
    /// * `y_bus` - A reference to a `DMatrix<Complex<f64>>` containing the admittance values between network nodes.
    /// * `i` - The index corresponding to the node ID `id` in the `y_bus` matrix.
    ///
    /// # Returns
    /// * `Complex<f64>` - The total current as a complex number, where the real and imaginary parts represent
    ///   the real and reactive components of the current, respectively.
    fn total_current(
        &self,
        graph: &UndirectedGraph,
        id: usize,
        y_bus: &DMatrix<Complex<f64>>,
        i: usize,
    ) -> Complex<f64> {
        let mut vy_sum = Complex::new(0.0, 0.0); // Sum of currents
        for neighbor_id in graph.neighbors(id) {
            if let Some(neighbor) = graph.node(neighbor_id) {
                let k = neighbor_id % graph.node_count(); // Index for y_bus matrix.
                let y_ik = y_bus[(i, k)]; // Admittance between bus i and bus k
                let v_k = neighbor.voltage().to_complex();
                vy_sum += v_k * y_ik; // V*Y = I
            }
        }
        vy_sum
    }

    /// Calculates the new voltage for a node using the Gauss-Seidel iteration method.
    ///
    /// This function updates the voltage of a node based on the Gauss-Seidel formula. The new voltage is calculated
    /// using the node's own power, its self-admittance, and the total current computed from its connected neighbors.
    /// This method is typically used in iterative solvers to converge to a steady-state solution for the voltages in a power network.
    ///
    /// # Arguments
    /// * `graph` - A reference to an `UndirectedGraph` representing the electrical network.
    /// * `id` - The node ID for which the voltage is to be updated.
    /// * `node` - A reference to a `BusNode` representing the node whose voltage is being updated.
    /// * `y_bus` - A reference to a `DMatrix<Complex<f64>>` containing the admittance values between network nodes.
    ///
    /// # Returns
    /// * `Complex<f64>` - The updated voltage as a complex number, where the real and imaginary parts represent
    ///   the real and reactive voltage components, respectively.
    fn calculate_node_voltage(
        &self,
        graph: &UndirectedGraph,
        id: usize,
        node: &BusNode,
        y_bus: &DMatrix<Complex<f64>>,
    ) -> Complex<f64> {
        let i = id % graph.node_count(); // Ensure we are within the bounds of our y_bus matrix.
        let v_old = node.voltage().to_complex();
        let vy_sum = self.total_current(graph, id, y_bus, i);
        let y_ii = y_bus[(i, i)]; // Self admittance.
        let s_i = node.power().to_complex();

        // Compute the new voltage using the Gauss-Seidel update formula.
        // Conjugate of complex number: a + bi -> a - bi
        y_ii.recip() * ((s_i.conj() / v_old.conj()) - vy_sum)
    }
}

impl Solver for GaussSeidel {
    fn solve(
        self,
        graph: &mut UndirectedGraph,
        max_iterations: usize,
        tolerance: f64,
    ) -> Result<(), &'static str> {
        let mut iteration: usize = 0;
        let mut converged: bool = false;
        let y_bus = admittance_matrix(graph);
        if zero_diagonal_elements(&y_bus) {
            return Err("Zero diagonal elements in admittance matrix. This means a node is not connected to any other node in the graph.");
        }
        let initial_voltages = initial_voltages(graph);
        while !converged && iteration < max_iterations {
            let mut max_voltage_change = 0.0;

            for id in graph.nodes() {
                if let Some(node) = graph.node(id) {
                    if node.is_slack() {
                        // Slack stays constant during Gauss-Seidel
                        continue;
                    }

                    let v_old = node.voltage().to_complex();

                    // Update the voltage with the current values
                    let v_new = self.calculate_node_voltage(graph, id, node, &y_bus);

                    // Calculate the magnitude of voltage change; complex numbers lack order, so we use norm for comparison.
                    let voltage_change = (v_new - v_old).norm();

                    // Update node based on its type
                    let mut node_update = *node;
                    match node.bus_type() {
                        BusType::Generator => {
                            node_update.set_voltage(Voltage::new(
                                Voltage::from_complex(v_old).amplitude,
                                Voltage::from_complex(v_new).angle,
                            ));
                        }
                        BusType::Load => {
                            node_update.set_voltage(Voltage::from_complex(v_new));
                        }
                        _ => {}
                    }

                    node_update.set_voltage(Voltage::from_complex(v_new));
                    graph.add_node(id, node_update);
                    if voltage_change > max_voltage_change {
                        max_voltage_change = voltage_change;
                    }
                }
            }
            // Convergence is reached when voltage change falls below tolerance, indicating further updates are negligible .
            converged = max_voltage_change < tolerance;
            iteration += 1;
        }
        // If the algorithm converged, set the voltages to the final values.
        // Make final check for convergence to ensure all nodes have valid voltages.
        if converged {
            set_voltages(graph, initial_voltages);
            converged = check_convergence(graph);
        } else {
            check_convergence(graph);
        }
        if converged {
            Ok(())
        } else {
            Err("Gauss-Seidel did not converge")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::edge::{LineType, Transmission};
    use crate::graph::node::{BusNode, PowerType};
    fn test_graph_1() -> UndirectedGraph {
        let mut graph = UndirectedGraph::new(1.0, 10.0, 1.0);
        let slack = BusNode::slack(graph.get_new_id());
        let pq1 = BusNode::load(graph.get_new_id(), 0.25, 0.1);
        let pq2 = BusNode::generator(graph.get_new_id(), 0.2, 0.1, PowerType::Battery);
        let pq3 = BusNode::generator(graph.get_new_id(), 0.2, 0.1, PowerType::Battery);
        let pq4 = BusNode::load(graph.get_new_id(), 0.1, 0.1);
        let pq5 = BusNode::load(graph.get_new_id(), 0.1, 0.01);
        let pq6 = BusNode::load(graph.get_new_id(), 0.1, 0.01);
        let pq7 = BusNode::generator(graph.get_new_id(), 0.25, 0.1, PowerType::Hydro);

        let l1 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l2 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l3 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l4 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l5 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l6 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l7 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l8 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l9 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l10 = Transmission::new(LineType::ACSRConductor, 100.0);

        graph.add_node(pq1.id(), pq1);
        graph.add_node(pq4.id(), pq4);
        graph.add_node(pq5.id(), pq5);
        graph.add_node(pq6.id(), pq6);
        graph.add_node(pq2.id(), pq2);
        graph.add_node(pq3.id(), pq3);
        graph.add_node(pq7.id(), pq7);
        graph.add_node(slack.id(), slack);

        graph.add_edge(slack.id(), pq1.id(), l1);
        graph.add_edge(slack.id(), pq2.id(), l2);
        graph.add_edge(pq1.id(), pq2.id(), l3);
        graph.add_edge(pq2.id(), pq3.id(), l4);
        graph.add_edge(pq3.id(), pq4.id(), l5);
        graph.add_edge(pq4.id(), pq5.id(), l6);
        graph.add_edge(pq5.id(), slack.id(), l7);
        graph.add_edge(pq5.id(), pq6.id(), l8);
        graph.add_edge(pq6.id(), pq7.id(), l9);
        graph.add_edge(pq7.id(), slack.id(), l10);
        graph
    }
    fn test_graph_2() -> UndirectedGraph {
        let mut graph = UndirectedGraph::new(1.0, 10.0, 1.0);
        let slack = BusNode::slack(graph.get_new_id());
        let pq1 = BusNode::load(graph.get_new_id(), 0.25, 0.1);
        let pq2 = BusNode::generator(graph.get_new_id(), 2.2, 2.1, PowerType::Battery);

        let l1 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l2 = Transmission::new(LineType::AAACConductor, 100.0);
        let l3 = Transmission::new(LineType::ACSRConductor, 100.0);

        graph.add_node(slack.id(), slack);
        graph.add_node(pq1.id(), pq1);
        graph.add_node(pq2.id(), pq2);

        graph.add_edge(slack.id(), pq1.id(), l1);
        graph.add_edge(slack.id(), pq2.id(), l2);
        graph.add_edge(pq1.id(), pq2.id(), l3);
        graph
    }
    fn test_graph_3() -> UndirectedGraph {
        let mut graph = UndirectedGraph::new(1.0, 10.0, 1.0);
        let slack = BusNode::slack(graph.get_new_id());
        let pq1 = BusNode::load(graph.get_new_id(), 0.25, 0.1);

        let l1 = Transmission::new(LineType::ACSRConductor, 100.0);

        graph.add_node(slack.id(), slack);
        graph.add_node(pq1.id(), pq1);

        graph.add_edge(slack.id(), pq1.id(), l1);
        graph
    }
    #[test]
    fn test_gauss_seidel() {
        let mut graph = test_graph_1();
        let solver = GaussSeidel::new();
        let result = solver.solve(&mut graph, 100, 0.0001);
        assert_eq!(result, Ok(()));
        let mut graph = test_graph_2();
        let solver2 = GaussSeidel::new();
        let result2 = solver2.solve(&mut graph, 100, 0.0001);
        assert_eq!(result2, Ok(()));
        let mut graph = test_graph_3();
        let solver3 = GaussSeidel::new();
        let result3 = solver3.solve(&mut graph, 100, 0.0001);
        assert_eq!(result3, Ok(()));
    }
    #[test]
    fn test_zero_diagonal_elements() {
        let mut graph = test_graph_1();
        let y_bus = admittance_matrix(&mut graph);
        assert!(!zero_diagonal_elements(&y_bus));
        let y_bus = DMatrix::from_row_slice(
            2,
            2,
            &[
                Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0),
            ],
        );
        assert!(zero_diagonal_elements(&y_bus));
    }
}
