mod graph;
mod solvers;
mod units;
mod utils;
use crate::graph::edge::Transmission;
use crate::graph::electric_graph::Graph as sim_graph;
use crate::graph::electric_graph::UndirectedGraph;
use component_library::energy::{
    CableType, GeneratorNode, LoadNode, PowerType, SensorGeneratorNode, SensorLoadNode, SlackNode,
    TransmissionEdge,
};
use graph::{edge::LineType, node::BusNode, node::PowerType as BusNodeType};
use simulator_communication::graph::{Node, NodeId};
use simulator_communication::simulator::SimulationError;
use simulator_communication::{ComponentsInfo, Graph, Server, Simulator};
use solvers::solver::Solver;
use std::{collections::HashMap, env, net::SocketAddr, process::ExitCode};
use tracing::{error, info};
#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    tracing_subscriber::fmt().init();

    let listen_addr = match env::var("LOAD_FLOW_SIMULATOR_ADDR")
        .unwrap_or("0.0.0.0:8101".to_string())
        .parse::<SocketAddr>()
    {
        Ok(v) => v,
        Err(err) => {
            error!("Could not parse bind address: {err}.");
            return ExitCode::FAILURE;
        }
    };

    // Manager address
    let connector_addr =
        env::var("SIMULATOR_CONNECTOR_ADDR").unwrap_or("http://127.0.0.1:8099".to_string());

    let server = Server::<LoadFlowSimulator>::new();

    info!("Starting energy simulator server on `{listen_addr}`.");
    if let Err(err) = server.start(listen_addr, connector_addr, "load flow").await {
        error!("Server return an error: {err}.");
        return ExitCode::FAILURE;
    }
    info!("Server exited successfully.");
    ExitCode::SUCCESS
}

#[allow(dead_code)]
pub struct LoadFlowSimulator {}

impl LoadFlowSimulator {
    /// Collects nodes of a specific type from the graph.
    fn collect_nodes<C: simulator_communication::component::Component>(
        graph: &Graph,
    ) -> Vec<(NodeId, &Node)> {
        graph
            .get_all_nodes::<C>()
            .unwrap()
            .map(|(id, node, _)| (id, node))
            .collect()
    }

    /// Creates a translation map from sensor nodes to real nodes.
    fn create_translation<
        C1: simulator_communication::component::Component,
        C2: simulator_communication::component::Component,
    >(
        graph: &Graph,
    ) -> Result<HashMap<NodeId, NodeId>, String> {
        let sensor_nodes = Self::collect_nodes::<C1>(graph);
        let real_nodes = Self::collect_nodes::<C2>(graph);

        if sensor_nodes.len() != real_nodes.len() {
            return Err("The count of sensor and real nodes must be equal.".to_owned());
        }

        let trans = sensor_nodes
            .iter()
            .zip(real_nodes.iter())
            .map(|((sensor_id, _), (real_id, _))| (*sensor_id, *real_id))
            .collect();
        Ok(trans)
    }

    /// Main function to create node translation map for the graph.
    fn create_node_translation_map(graph: &Graph) -> Result<HashMap<NodeId, NodeId>, String> {
        let load_translations = Self::create_translation::<SensorLoadNode, LoadNode>(graph)?;
        let generator_translations =
            Self::create_translation::<SensorGeneratorNode, GeneratorNode>(graph)?;

        let mut node_translations = load_translations;
        node_translations.extend(generator_translations);

        Ok(node_translations)
    }
}

impl Simulator for LoadFlowSimulator {
    fn get_component_info() -> ComponentsInfo {
        ComponentsInfo::new()
            .add_required_component::<TransmissionEdge>()
            .add_required_component::<SensorLoadNode>()
            .add_required_component::<SensorGeneratorNode>()
            .add_required_component::<LoadNode>()
            .add_required_component::<GeneratorNode>()
            .add_optional_component::<SlackNode>()
            .add_output_component::<GeneratorNode>()
            .add_output_component::<LoadNode>()
            .add_output_component::<SlackNode>()
            .add_output_component::<TransmissionEdge>()
    }

    async fn new(_: std::time::Duration, _graph: Graph) -> Result<Self, SimulationError> {
        Ok(Self {})
    }

    async fn do_timestep(&mut self, mut graph: Graph) -> Result<Graph, SimulationError> {
        let mut nodes_position: HashMap<usize, (f64, f64)> = HashMap::new();
        let gs_solver = false;
        // Set default values if input is invalid
        let max_iterations = 1000;
        let tolerance = 0.001;
        let mut g = UndirectedGraph::new(1.0, 1.0, 1.0);
        let mut nodes: HashMap<NodeId, usize> = HashMap::new();
        let mut edges = HashMap::new();

        // Translate base node id to corresponding load flow analysis node
        let Ok(node_translations) = Self::create_node_translation_map(&graph) else {
            return Err(SimulationError::InvalidInput("The load flow simulation needs a sensor load node for every load node and a sensor generator node for every generator node".to_owned()));
        };

        for (nodeid, node, comp) in graph.get_all_nodes::<SensorLoadNode>().unwrap() {
            let load = BusNode::load(g.get_new_id(), comp.active_power, comp.reactive_power);
            g.add_node(load.id(), load);
            let mapped_id = node_translations.get(&nodeid).unwrap();
            nodes.insert(*mapped_id, load.id());
            nodes_position.insert(load.id(), (node.latitude, node.longitude));
        }

        for (nodeid, node, comp) in graph.get_all_nodes::<SensorGeneratorNode>().unwrap() {
            let generator = BusNode::generator(
                g.get_new_id(),
                comp.active_power,
                comp.voltage_magnitude,
                power_type_to_busnode_type(comp.power_type),
            );
            g.add_node(generator.id(), generator);
            let mapped_id = node_translations.get(&nodeid).unwrap();
            nodes.insert(*mapped_id, generator.id());
            nodes_position.insert(generator.id(), (node.latitude, node.longitude));
        }

        // if slack nodes in graph, add them to the graph
        for (nodeid, node, _comp) in graph.get_all_nodes::<SlackNode>().unwrap() {
            let slack = BusNode::slack(g.get_new_id());
            g.add_node(slack.id(), slack);
            nodes.insert(nodeid, slack.id());
            nodes_position.insert(slack.id(), (node.latitude, node.longitude));
        }

        for (edgeid, edge, comp) in graph.get_all_edges::<TransmissionEdge>().unwrap() {
            //need to find id of node corresponding to the nodeid
            if let (Some(nid1), Some(nid2)) = (nodes.get(&edge.from), nodes.get(&edge.to)) {
                //get lat and long of nodes
                //unwrap is safe as all nodes are added to the graph
                let (lat1, long1) = nodes_position.get(nid1).unwrap();
                let (lat2, long2) = nodes_position.get(nid2).unwrap();
                //calculate distance between nodes
                let distance = utils::haversine_distance(*lat1, *long1, *lat2, *long2);
                if distance == 0.0 {
                    return Err(SimulationError::InvalidInput(
                        "Distance between load-flow busses is 0".to_owned(),
                    ));
                }
                if distance == std::f64::INFINITY {
                    return Err(SimulationError::InvalidInput(
                        "Distance between load-flow busses is infinitely big".to_owned(),
                    ));
                }
                //add edge to graph
                let line = Transmission::new(cable_type_to_line_type(comp.line_type), distance);
                g.add_edge(*nid1, *nid2, line);
                edges.insert(edgeid, (*nid1, *nid2));
            }
        }
        // iterate over all nodes and edges to find the max values
        // and set all values to p.u
        let (v_base, p_base, s_base) = g.calculate_optimal_bases();
        g.set_bases(v_base, s_base, p_base);

        // Match on the solver's result to handle Ok and Err cases
        if gs_solver {
            // Use Gauss-Seidel solver
            let solver = solvers::gauss_seidel::GaussSeidel::new();
            let _ = solver.solve(&mut g, max_iterations as usize, tolerance); // Solve the power flow
        } else {
            // Use Newton-Raphson solver
            let solver = solvers::newton_raphson::NewtonRaphson::new();
            let _ = solver.solve(&mut g, max_iterations as usize, tolerance); // Solve the power flow
        }
        // reset all values to original values
        g.reset_bases();

        // update the grap of communication library
        // Place updated data back into the graph
        for (nodeid, _, comp) in graph.get_all_nodes_mut::<LoadNode>().unwrap() {
            if let Some(vertex) = nodes.get(&nodeid) {
                // Unwrap is safe as all supersteps have completed at this point
                let vertex = g.node(*vertex).unwrap();
                *comp = LoadNode {
                    active_power: vertex.power().active,
                    reactive_power: vertex.power().reactive,
                    voltage_amplitude: vertex.voltage().amplitude,
                    voltage_angle: vertex.voltage().angle,
                };
            }
        }

        for (nodeid, _, comp) in graph.get_all_nodes_mut::<GeneratorNode>().unwrap() {
            if let Some(vertex) = nodes.get(&nodeid) {
                // Unwrap is safe as all supersteps have completed at this point
                let og_node = g.node(*vertex).unwrap();
                *comp = GeneratorNode {
                    active_power: og_node.power().active,
                    voltage_amplitude: og_node.voltage().amplitude,
                    voltage_angle: og_node.voltage().angle,
                    power_type: busnode_type_to_power_type(og_node.energy_type()),
                    ..*comp
                };
            }
        }
        for (_edgeid, edge, comp) in graph.get_all_edges_mut::<TransmissionEdge>().unwrap() {
            if let (Some(nid1), Some(nid2)) = (nodes.get(&edge.from), nodes.get(&edge.to)) {
                if let Some(line) = g.edge(*nid1, *nid2) {
                    if let Some(sending) = nodes.get(&edge.from) {
                        if let Some(receiving) = nodes.get(&edge.to) {
                            *comp = TransmissionEdge {
                                resistance_per_meter: line.resistance() / line.length(),
                                reactance_per_meter: line.impedance(g.z_base()).reactance
                                    / line.length(),
                                length: line.length(),
                                current: line
                                    .current(
                                        g.node(*sending).unwrap().voltage(),
                                        g.node(*receiving).unwrap().voltage(),
                                        g.z_base(),
                                    )
                                    .magnitude
                                    .abs()
                                    * g.z_base(),
                                line_type: line_type_to_cable_type(line.line_type()),
                                ..*comp
                            };
                        }
                    }
                }
            }
        }
        Ok(graph.filter(Self::get_component_info()))
    }
}
fn power_type_to_busnode_type(power_type: PowerType) -> BusNodeType {
    match power_type {
        PowerType::Renewable => BusNodeType::Renewable,
        PowerType::Storage => BusNodeType::Storage,
        PowerType::Battery => BusNodeType::Battery,
        PowerType::Hydro => BusNodeType::Hydro,
        PowerType::Wind => BusNodeType::Wind,
        PowerType::Solar => BusNodeType::Solar,
        PowerType::Nuclear => BusNodeType::Nuclear,
        PowerType::Fossil => BusNodeType::Fossil,
    }
}
fn busnode_type_to_power_type(busnode_type: BusNodeType) -> PowerType {
    match busnode_type {
        BusNodeType::Renewable => PowerType::Renewable,
        BusNodeType::Storage => PowerType::Storage,
        BusNodeType::Battery => PowerType::Battery,
        BusNodeType::Hydro => PowerType::Hydro,
        BusNodeType::Wind => PowerType::Wind,
        BusNodeType::Solar => PowerType::Solar,
        BusNodeType::Nuclear => PowerType::Nuclear,
        BusNodeType::Fossil => PowerType::Fossil,
        _ => PowerType::Storage,
    }
}
fn cable_type_to_line_type(cable_type: CableType) -> LineType {
    match cable_type {
        CableType::ACSRConductor => LineType::ACSRConductor,
        CableType::AACConductor => LineType::AACConductor,
        CableType::AAACConductor => LineType::AAACConductor,
        CableType::XLPECable => LineType::XLPECable,
        CableType::PILCCable => LineType::PILCCable,
    }
}
fn line_type_to_cable_type(line_type: LineType) -> CableType {
    match line_type {
        LineType::ACSRConductor => CableType::ACSRConductor,
        LineType::AACConductor => CableType::AACConductor,
        LineType::AAACConductor => CableType::AAACConductor,
        LineType::XLPECable => CableType::XLPECable,
        LineType::PILCCable => CableType::PILCCable,
    }
}
