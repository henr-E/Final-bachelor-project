use crate::graph::electric_graph::{Graph, UndirectedGraph};
use crate::graph::node::BusType;
use crate::solvers::solver::Solver;
use crate::units::voltage::Voltage;
use crate::utils::{admittance_matrix, print_matrix};
use nalgebra::Complex;
use nalgebra::ComplexField;
use std::collections::HashMap;
pub struct GaussSeidel;
#[allow(dead_code)]
impl GaussSeidel {
    pub fn new() -> Self {
        GaussSeidel {}
    }
}
/// Check if the voltage of any node is NaN and replace it with a default value.
fn check_convergence(graph: &mut UndirectedGraph) -> bool {
    let mut convergence = true;
    for node in graph.nodes() {
        if let Some(node) = graph.node(node) {
            if node.voltage().amplitude.is_nan() || node.voltage().angle.is_nan() {
                convergence = false;
                let mut new_node = *node;
                new_node.set_voltage(Voltage::new(1.0, 0.0));
                graph.add_node(new_node.id(), new_node);
            }
        }
    }
    convergence
}
/// set voltage magnitude of generator nodes to initial values. This is necessary to prevent the algorithm from changing the voltage of generator nodes.
/// For maximimization of convergence, the voltage of generator nodes can change during the algorithm but should be reset.
fn initial_voltages(graph: &mut UndirectedGraph) -> HashMap<usize, Voltage> {
    let mut voltages: HashMap<usize, Voltage> = HashMap::new();
    for id in graph.nodes() {
        if let Some(node) = graph.node(id) {
            voltages.insert(id, node.voltage());
        }
    }
    voltages
}
/// set voltage of node to the value in the hashmap.
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
        let initial_voltages = initial_voltages(graph);
        if y_bus.determinant() == Complex::new(0.0, 0.0) {
            print_matrix(y_bus);
            return Err("Singular matrix");
        }
        while !converged && iteration < max_iterations {
            let mut max_voltage_change = 0.0;

            for id in graph.nodes() {
                let i = id % graph.node_count(); // ID is a static counter
                if let Some(node) = graph.node(id) {
                    if node.is_slack() {
                        // Slack stays constant during Gauss-Seidel
                        continue;
                    }

                    let v_old = node.voltage().to_complex();

                    // Total current
                    let mut vy_sum: Complex<f64> = Complex::new(0.0, 0.0);

                    for neighbor_id in graph.neighbors(id) {
                        if let Some(neighbor) = graph.node(neighbor_id) {
                            let k = neighbor_id % graph.node_count();

                            // y_ik represents the admittance between node i and its neighbor k.
                            let y_ik = y_bus[(i, k)];
                            let v_k = neighbor.voltage().to_complex();

                            vy_sum += v_k * y_ik;
                        }
                    }

                    let y_ii = y_bus[(i, i)];

                    let s_i = node.power().to_complex();
                    // Update the voltage with the current values
                    // Conjugate of complex number: a + bi -> a - bi
                    let v_new = y_ii.recip() * ((s_i.conj() / v_old.conj()) - vy_sum);

                    // Calculate the magnitude of voltage change; complex numbers lack order, so we use norm for comparison.
                    let voltage_change = (v_new - v_old).norm();

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
    #[test]
    fn test_gauss_seidel() {
        let mut graph = UndirectedGraph::new(1.0, 10.0, 1.0);
        let slack = BusNode::slack();
        let pq1 = BusNode::load(0.25, 0.1);
        let pq2 = BusNode::generator(0.2, 0.1, PowerType::Battery);
        let pq3 = BusNode::generator(0.2, 0.1, PowerType::Battery);
        let pq4 = BusNode::load(0.1, -0.1);
        let pq5 = BusNode::load(0.1, -0.0);
        let pq6 = BusNode::load(0.1, -0.0);
        let pq7 = BusNode::generator(0.25, 0.1, PowerType::Hydro);

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

        graph.add_node(slack.id(), slack);
        graph.add_node(pq1.id(), pq1);
        graph.add_node(pq2.id(), pq2);
        graph.add_node(pq3.id(), pq3);
        graph.add_node(pq4.id(), pq4);
        graph.add_node(pq5.id(), pq5);
        graph.add_node(pq6.id(), pq6);
        graph.add_node(pq7.id(), pq7);

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

        let solver = GaussSeidel::new();
        let result = solver.solve(&mut graph, 100, 0.0001);
        assert_eq!(result, Ok(()));
    }
}
