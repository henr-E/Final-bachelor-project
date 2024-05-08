use std::collections::HashMap;

use crate::graph::electric_graph::{Graph, UndirectedGraph};
use crate::graph::node::BusType;
use crate::solvers::solver::Solver;
use crate::units::power::Power;
use crate::units::voltage::{self, Voltage};
use crate::utils::{admittance_matrix, check_convergence};
use nalgebra::{Complex, DMatrix};
use num_complex::ComplexFloat;
/// Apply damping to the jacobian matrix
fn apply_damping(jacobian: &mut DMatrix<f64>, epsilon: f64) {
    // Apply damping by adding epsilon to each diagonal element
    for i in 0..jacobian.nrows() {
        // Add epsilon to the diagonal element
        jacobian[(i, i)] += epsilon;
    }
}

/// Order the nodes in the graph and return a hashmap with the position of the node in the jacobian matrix
fn order_nodes(graph: &UndirectedGraph) -> (HashMap<usize, usize>, HashMap<usize, usize>) {
    let mut loads_generators: HashMap<usize, usize> = HashMap::new();
    let mut loads: HashMap<usize, usize> = HashMap::new();
    for node in graph.nodes() {
        if let Some(node) = graph.node(node) {
            match node.bus_type() {
                BusType::Generator => {
                    loads_generators.insert(node.id(), loads_generators.len());
                }
                BusType::Load => {
                    loads.insert(node.id(), loads.len());
                    loads_generators.insert(node.id(), loads_generators.len());
                }
                _ => {}
            }
        }
    }
    (loads_generators, loads)
}
pub struct NewtonRaphson;

impl NewtonRaphson {
    pub fn new() -> Self {
        NewtonRaphson {}
    }
}
use std::f64::INFINITY;

fn find_min_amplitude(graph: &UndirectedGraph) -> f64 {
    // Initialize min_amplitude to infinity
    let mut min_amplitude = INFINITY;

    // Iterate through all nodes in the graph
    for node_id in graph.nodes() {
        // Get the node and its voltage
        if let Some(node) = graph.node(node_id) {
            let amplitude = node.voltage().amplitude.abs();

            // Update min_amplitude if a smaller value is found
            if amplitude < min_amplitude {
                min_amplitude = amplitude;
            }
        }
    }

    // Return the minimum amplitude
    min_amplitude
}
fn average_amplitude(graph: &UndirectedGraph) -> Option<f64> {
    let mut total_amplitude = 0.0; // Variable to keep track of the total amplitude
    let mut count = 0; // Variable to count the number of nodes with voltages

    // Iterate through all nodes in the graph
    for node_id in graph.nodes() {
        // Get the node
        if let Some(node) = graph.node(node_id) {
            // Increment the count
            count += 1;
            // Add the amplitude of the voltage to the total amplitude
            total_amplitude += node.voltage().amplitude.abs();
        }
    }

    // Check if count is greater than zero to avoid division by zero
    if count > 0 {
        // Calculate the average amplitude
        Some(total_amplitude / count as f64)
    } else {
        // If there are no nodes with voltages, return None
        None
    }
}

/// Refactor the voltages of the graph
///
/// This function is used to refactor the voltages of the graph after the newton raphson algorithm has converged
/// the voltage angles are refactored to be between -2pi and 2pi
/// the voltage amplitudes can be refactored either by a common denominator or by taking the values of the voltages right before the convergence

fn refactor_voltages(
    graph: &mut UndirectedGraph,
    voltages: HashMap<usize, Voltage>,
    max_amplitude: f64,
) {
    for (i, voltage) in voltages {
        if let Some(node) = graph.node(i) {
            let mut update = *node;
            // Calculate scaling factor
            let scale_factor = max_amplitude / voltage.amplitude;
            // Scale down the amplitude
            let scaled_amplitude = voltage.amplitude * scale_factor;
            update.set_voltage(Voltage::new(
                scaled_amplitude,
                voltage.angle % 2.0 * std::f64::consts::PI,
            ));

            graph.add_node(i, update);
        }
    }
}
/// Set the jacobian matrix for the newton raphson algorithm
/// nodes is a hashmap that indicates in which position the node derrivatis are placed in the jacobian matrix
fn set_jacobian(
    graph: &UndirectedGraph,
    i: usize,
    y_bus: &DMatrix<Complex<f64>>,
    loads_generators: &HashMap<usize, usize>,
    loads: &HashMap<usize, usize>,
    jacobian: &mut DMatrix<f64>,
) -> Power {
    // f_pk = P_k - P_k^d = V_i * sum(V_j * (G_ij * cos(l_kj) + B_ij * sin(l_kj)))
    let mut f_pk: f64 = 0.0;
    // f_qk = Q_k - Q_k^d = V_i * sum(V_j * (G_ij * sin(l_kj) - B_ij * cos(l_kj)))
    let mut f_qk: f64 = 0.0;
    // derrivatives of f_pk with respect to l_k with l_k = theta_i - theta_j and v_k = V_i
    let mut der_f_pk_l_k = 0.0;
    // derrivatives of f_pk with respect to v_k with l_k = theta_i - theta_j and v_k = V_i
    let mut der_f_pk_v_k = 0.0;
    // derrivatives of f_qk with respect to l_k with l_k = theta_i - theta_j and v_k = V_i
    let mut der_f_qk_l_k = 0.0;
    // derrivatives of f_qk with respect to v_k with l_k = theta_i - theta_j and v_k = V_i
    let mut der_f_qk_v_k = 0.0;

    let mut i_pos: usize = 0;
    if let Some(pos) = loads_generators.get(&i) {
        i_pos = *pos;
    }
    //safe to use unwrap since we are iterating over the nodes
    let node = graph.node(i).unwrap();

    for n in graph.neighbors(i) {
        if let Some(neighbor) = graph.node(n) {
            let l_kj = node.voltage().angle - neighbor.voltage().angle;
            let y_ik = y_bus[(i % graph.node_count(), n % graph.node_count())];
            f_pk += neighbor.voltage().amplitude.abs()
                * (y_ik.re() * l_kj.cos() + y_ik.im() * l_kj.sin());

            f_qk += neighbor.voltage().amplitude.abs()
                * (y_ik.re() * l_kj.sin() - y_ik.im() * l_kj.cos());
            //compute derivative of f_pk with respect to l_k
            der_f_pk_l_k += neighbor.voltage().amplitude.abs()
                * (-1.0 * y_ik.re() * l_kj.sin() + y_ik.im() * l_kj.cos());
            //compute derivative of f_vk with respect to v_k
            der_f_pk_v_k += neighbor.voltage().amplitude.abs()
                * (y_ik.re() * l_kj.cos() + y_ik.im() * l_kj.sin());
            //compute derivative of f_qk with respect to l_k
            der_f_qk_l_k += neighbor.voltage().amplitude.abs()
                * (y_ik.re() * l_kj.cos() + y_ik.im() * l_kj.sin());
            //compute derivative of f_qk with respect to v_k
            der_f_qk_v_k += neighbor.voltage().amplitude.abs()
                * (y_ik.re() * l_kj.sin() - y_ik.im() * l_kj.cos());
            //fill the jacobian matrix
            if let Some(pos) = loads_generators.get(&n) {
                let pos_index = *pos;
                //der_f_pk_l_l
                jacobian[(i_pos, pos_index)] = node.voltage().amplitude
                    * neighbor.voltage().amplitude.abs()
                    * (y_ik.re() * l_kj.sin() - y_ik.im() * l_kj.cos());

                if let Some(l_index) = loads.get(&n) {
                    //der_f_pk_v_l
                    jacobian[(i_pos, loads_generators.len() + l_index)] =
                        node.voltage().amplitude.abs()
                            * (y_ik.re() * l_kj.cos() + y_ik.im() * l_kj.sin());
                }
                if let Some(l_i_index) = loads.get(&i) {
                    //der_f_qk_v_l
                    if let Some(g_index) = loads.get(&n) {
                        jacobian[(
                            loads_generators.len() + l_i_index,
                            loads_generators.len() + g_index,
                        )] = node.voltage().amplitude.abs()
                            * (y_ik.re() * l_kj.sin() - y_ik.im() * l_kj.cos());
                    }

                    //der_f_qk_l_l
                    jacobian[(loads_generators.len() + l_i_index, pos_index)] =
                        node.voltage().amplitude.abs()
                            * neighbor.voltage().amplitude.abs()
                            * (-1.0 * y_ik.re() * l_kj.cos() + y_ik.im() * l_kj.sin());
                }
            }
        }
        f_pk *= node.voltage().amplitude.abs();
        f_pk += node.voltage().amplitude.abs()
            * node.voltage().amplitude.abs()
            * y_bus[(i % graph.node_count(), i % graph.node_count())].re();

        f_qk *= node.voltage().amplitude.abs();
        f_qk += -node.voltage().amplitude.abs()
            * node.voltage().amplitude.abs()
            * y_bus[(i % graph.node_count(), i % graph.node_count())].im();

        der_f_pk_l_k *= node.voltage().amplitude.abs();
        let preval = 2.0
            * node.voltage().amplitude.abs()
            * y_bus[(i % graph.node_count(), i % graph.node_count())].re();
        der_f_pk_v_k += preval;
        der_f_qk_l_k *= node.voltage().amplitude.abs();
        der_f_qk_v_k += -2.0
            * node.voltage().amplitude.abs()
            * y_bus[(i % graph.node_count(), i % graph.node_count())].im();

        //fill the jacobian matrix
        jacobian[(i_pos, i_pos)] = der_f_pk_l_k;

        if let Some(l_i_index) = loads.get(&i) {
            jacobian[(i_pos, loads_generators.len() + *l_i_index)] = der_f_pk_v_k;
            jacobian[(loads_generators.len() + *l_i_index, i_pos)] = der_f_qk_l_k;
            jacobian[(
                loads_generators.len() + *l_i_index,
                loads_generators.len() + *l_i_index,
            )] = der_f_qk_v_k;
        }
    }
    Power::new(f_pk, f_qk)
}
impl Solver for NewtonRaphson {
    fn solve(
        self,
        graph: &mut UndirectedGraph,
        max_iterations: usize,
        tolerance: f64,
    ) -> Result<(), &'static str> {
        let mut converged = false;
        let mut iteration = 0;
        let mut max_norm_v;
        let mut max_norm_l;
        let mut min_amplitude = find_min_amplitude(graph).abs();
        if let Some(average) = average_amplitude(graph) {
            min_amplitude = average;
        }
        let mut voltages_vec: Vec<HashMap<usize, Voltage>> = Vec::new();
        let (loads_gen, loads) = order_nodes(graph);
        let mut _old_voltages = DMatrix::from_element(loads_gen.len() + loads.len(), 1, 0.0);
        while !converged && max_iterations > iteration {
            let mut voltages_entry: HashMap<usize, Voltage> = HashMap::new();
            max_norm_v = 0.0;
            max_norm_l = 0.0;
            let y_bus = admittance_matrix(graph);
            let mut mismatch = DMatrix::from_element(loads_gen.len() + loads.len(), 1, 0.0);
            _old_voltages = DMatrix::from_element(loads_gen.len() + loads.len(), 1, 0.0);
            let mut jacobian = DMatrix::from_element(
                loads_gen.len() + loads.len(),
                loads_gen.len() + loads.len(),
                0.0,
            );
            for i in loads_gen.keys() {
                if let Some(node) = graph.node(*i) {
                    if let Some(pos) = loads_gen.get(i) {
                        let power =
                            set_jacobian(graph, *i, &y_bus, &loads_gen, &loads, &mut jacobian);
                        mismatch[(*pos, 0)] = power.active + node.power().active;
                        _old_voltages[(*pos, 0)] = node.voltage().angle;
                        if node.bus_type() == BusType::Load {
                            if let Some(i_pos) = loads.get(i) {
                                mismatch[(loads_gen.len() + i_pos, 0)] =
                                    power.reactive - node.power().reactive;
                                _old_voltages[(loads_gen.len() + i_pos, 0)] =
                                    node.voltage().amplitude;
                            }
                        }
                    }
                }
            }
            if jacobian.determinant() == 0.0 {
                apply_damping(&mut jacobian, 0.0001);
            }
            let inv = jacobian.try_inverse();
            let inverse = match inv {
                Some(matrix) => matrix,
                None => return Err(
                    "Jacobian is a singular matrix. Cannot invert to solve with Newton-Raphson.",
                ),
            };
            let update = -1.0 * inverse * mismatch.clone() + _old_voltages.clone();
            for (i, _v) in loads_gen.clone() {
                if let Some(pos) = loads_gen.get(&i) {
                    if let Some(node) = graph.node(i) {
                        let mut new_node = *node;

                        match node.bus_type() {
                            BusType::Generator => {
                                //set max norm
                                if (update[(*pos, 0)] - new_node.voltage().angle).abs() > max_norm_l
                                {
                                    max_norm_l =
                                        (update[(*pos, 0)] - new_node.voltage().angle).abs();
                                }
                                //update the node
                                new_node.set_voltage(voltage::Voltage::new(
                                    node.voltage().amplitude,
                                    update[(*pos, 0)],
                                ));
                                graph.add_node(new_node.id(), new_node);
                                voltages_entry.insert(i, new_node.voltage());
                            }
                            BusType::Load => {
                                //set max norm
                                if (update[(*pos, 0)] - new_node.voltage().angle).abs() > max_norm_l
                                {
                                    max_norm_l =
                                        (update[(*pos, 0)] - new_node.voltage().angle).abs();
                                }
                                if let Some(l_index) = loads.get(&i) {
                                    if (update[(loads_gen.len() + l_index, 0)]
                                        - new_node.voltage().amplitude)
                                        .abs()
                                        > max_norm_v
                                    {
                                        max_norm_v = (update[(loads_gen.len() + l_index, 0)]
                                            - new_node.voltage().angle)
                                            .abs();
                                    }
                                    //update the node
                                    new_node.set_voltage(voltage::Voltage::new(
                                        update[(loads_gen.len() + l_index, 0)],
                                        update[(*pos, 0)],
                                    ));
                                    graph.add_node(new_node.id(), new_node);
                                    voltages_entry.insert(i, new_node.voltage());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            converged = max_norm_l < tolerance && max_norm_v < tolerance;
            iteration += 1;
            voltages_vec.push(voltages_entry.clone());
        }
        // if the newton raphson has converged refactor the voltages and check sensible voltage voltage values
        // if the voltages are not sensible reset to default values and return an error
        if converged {
            if voltages_vec.len() > 1 {
                let pos = voltages_vec[voltages_vec.len() - 2].clone();
                refactor_voltages(graph, pos, min_amplitude);
            }
            converged = check_convergence(graph);
        } else {
            check_convergence(graph);
        }
        if converged {
            Ok(())
        } else {
            Err("Newton-Raphson did not converge")
        }
    }
}

#[cfg(test)]
mod test {
    use std::f64::NAN;

    use super::*;
    use crate::graph::{
        edge::{LineType, Transmission},
        node::{BusNode, PowerType},
    };
    fn test_graph1() -> UndirectedGraph {
        let mut graph = UndirectedGraph::new(1.0, 1.0, 1.0);
        let pq1 = BusNode::load(graph.get_new_id(), 0.80, 0.1);
        let pq2 = BusNode::generator(graph.get_new_id(), 1.0, 1.1, PowerType::Battery);
        let pq3 = BusNode::load(graph.get_new_id(), 0.80, 0.1);

        let l1 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l2 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l3 = Transmission::new(LineType::ACSRConductor, 100.0);

        graph.add_node(pq1.id(), pq1);
        graph.add_node(pq2.id(), pq2);
        graph.add_node(pq3.id(), pq3);

        graph.add_edge(pq1.id(), pq2.id(), l1);
        graph.add_edge(pq2.id(), pq3.id(), l2);
        graph.add_edge(pq3.id(), pq1.id(), l3);
        graph
    }
    fn test_graph2() -> UndirectedGraph {
        let mut graph = UndirectedGraph::new(1.0, 1.0, 1.0);
        let slack = BusNode::slack(graph.get_new_id());
        let pq1 = BusNode::load(graph.get_new_id(), 1.0, -0.01);
        let pq2 = BusNode::generator(graph.get_new_id(), 1.2, 0.1, PowerType::Battery);
        let pq3 = BusNode::generator(graph.get_new_id(), 1.2, 0.1, PowerType::Battery);
        let pq4 = BusNode::load(graph.get_new_id(), 1.0, -0.1);
        let pq5 = BusNode::load(graph.get_new_id(), 100.0, -0.1);
        let pq6 = BusNode::load(graph.get_new_id(), 1.1, -0.1);
        let pq7 = BusNode::generator(graph.get_new_id(), 1.0, 100.10, PowerType::Fossil);

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
        let l11 = Transmission::new(LineType::ACSRConductor, 100.0);

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
        graph.add_edge(pq7.id(), pq1.id(), l10);
        graph.add_edge(pq7.id(), pq2.id(), l11);

        graph
    }
    fn test_graph_3() -> UndirectedGraph {
        let mut graph = UndirectedGraph::new(1.0, 1.0, 1.0);
        let pq1 = BusNode::load(graph.get_new_id(), 1.0, 0.1);
        let pq2 = BusNode::generator(graph.get_new_id(), 1.2, -10.1, PowerType::Battery);

        let l1 = Transmission::new(LineType::ACSRConductor, 100.0);
        graph.add_node(pq2.id(), pq2);
        graph.add_node(pq1.id(), pq1);
        graph.add_edge(pq2.id(), pq1.id(), l1);
        graph
    }

    #[test]
    fn test_newton_raphson() {
        let mut graph = test_graph1();
        //make instance of newton raphson
        let newton_raphson1 = NewtonRaphson::new();
        //solve the graph
        assert_eq!(newton_raphson1.solve(&mut graph, 100, 0.001), Ok(()));
        graph = test_graph2();
        let newton_raphson2 = NewtonRaphson::new();
        assert_eq!(newton_raphson2.solve(&mut graph, 1000, 0.001), Ok(()));
        graph = test_graph_3();
        let newton_raphson3 = NewtonRaphson::new();
        assert_eq!(newton_raphson3.solve(&mut graph, 900, 0.001), Ok(()));
    }
    #[test]
    fn test_jacobian_filler() {
        let mut graph = test_graph1();
        let y_bus = admittance_matrix(&mut graph);
        let (load_gens, loads) = order_nodes(&graph);
        let mut jacobian = DMatrix::from_element(
            load_gens.len() + loads.len(),
            load_gens.len() + loads.len(),
            0.0,
        );
        for i in load_gens.keys() {
            let _power = set_jacobian(&graph, *i, &y_bus, &load_gens, &loads, &mut jacobian);
        }

        apply_damping(&mut jacobian, 0.0001);
        assert_eq!(jacobian.nrows(), jacobian.ncols());
    }
    #[test]
    fn test_sensible_results() {
        let mut graph = UndirectedGraph::new(1.0, 1.0, 1.0);
        let node1 = BusNode::generator(graph.get_new_id(), 1.0, 1.0, PowerType::Fossil);
        let node2 = BusNode::generator(graph.get_new_id(), 1.0, NAN, PowerType::Fossil);

        graph.add_node(node1.id(), node1);
        graph.add_node(node2.id(), node2);
        let edge = Transmission::new(LineType::ACSRConductor, 100.0);
        graph.add_edge(node1.id(), node2.id(), edge);
        let check = check_convergence(&mut graph);
        assert!(!check);
        let check2 = check_convergence(&mut graph);
        assert!(check2);
    }
}
