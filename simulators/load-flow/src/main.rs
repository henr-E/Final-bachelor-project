mod diagnostics;
mod graph;
mod solvers;
mod units;
mod utils;
use crate::graph::edge::Transmission;
use crate::graph::electric_graph::Graph as sim_graph;
use crate::graph::electric_graph::UndirectedGraph;
use component_library::energy::{
    Bases, CableType, GeneratorNode, LoadNode, PowerType, ProductionOverview, SlackNode,
    TransmissionEdge,
};
use component_library::global::LoadFlowAnalytics;
use component_library::global::PowerTypeAnalytics;
use diagnostics::energy_production::power_type_percentages;
use diagnostics::total_power;
use graph::{edge::LineType, node::BusNode, node::PowerType as BusNodeType};
use solvers::solver::Solver;
use std::{collections::HashMap, env, net::SocketAddr, process::ExitCode};
use tracing::debug;
// Add the following line to import the `tracing` crate
use simulator_communication::{ComponentsInfo, Graph, Server, Simulator};
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
    if let Err(err) = server.start(listen_addr, connector_addr).await {
        error!("Server return an error: {err}.");
        return ExitCode::FAILURE;
    }
    info!("Server exited successfully.");
    ExitCode::SUCCESS
}

/// Simulator that gives a random demand and supply to a consumer and producer node respectively every timestep
#[allow(dead_code)]
pub struct LoadFlowSimulator {}

impl Simulator for LoadFlowSimulator {
    fn get_component_info() -> ComponentsInfo {
        ComponentsInfo::new()
            .add_required_component::<GeneratorNode>()
            .add_required_component::<LoadNode>()
            .add_required_component::<SlackNode>()
            .add_required_component::<TransmissionEdge>()
            .add_optional_component::<LoadFlowAnalytics>()
            .add_output_component::<TransmissionEdge>()
            .add_required_component::<Bases>()
    }

    fn new(_: std::time::Duration, _graph: Graph) -> Self {
        Self {}
    }

    fn do_timestep(&mut self, mut graph: Graph) -> Graph {
        let mut s_base = 0.0;
        let mut v_base = 0.0;
        let mut p_base = 0.0;
        for (_nodeid, _, comp) in graph.get_all_nodes::<Bases>().unwrap() {
            s_base = comp.s_base;
            v_base = comp.v_base;
            p_base = comp.p_base;
        }
        // Sbase, Vbase: example: 1.0, 10.0
        let mut g = UndirectedGraph::new(s_base, v_base, p_base);
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        for (nodeid, _, comp) in graph.get_all_nodes::<LoadNode>().unwrap() {
            let load = BusNode::load(comp.active_power, comp.reactive_power);
            g.add_node(load.id(), load);
            nodes.insert(nodeid, load.id());
        }
        for (nodeid, _, comp) in graph.get_all_nodes::<GeneratorNode>().unwrap() {
            let generator = BusNode::generator(
                comp.active_power,
                comp.voltage_amplitude,
                power_type_to_busnode_type(comp.power_type),
            );
            g.add_node(generator.id(), generator);
            nodes.insert(nodeid, generator.id());
        }
        for (nodeid, _, _comp) in graph.get_all_nodes::<SlackNode>().unwrap() {
            let slack = BusNode::slack();
            g.add_node(slack.id(), slack);
            nodes.insert(nodeid, slack.id());
        }
        for (edgeid, edge, comp) in graph.get_all_edges::<TransmissionEdge>().unwrap() {
            //roulet for which line type to use
            let line = Transmission::new(cable_type_to_line_type(comp.line_type), comp.length);
            //need to find id of node corresponding to the nodeid

            if let (Some(nid1), Some(nid2)) = (nodes.get(&edge.from), nodes.get(&edge.to)) {
                g.add_edge(*nid1, *nid2, line);
                edges.insert(edgeid, (*nid1, *nid2));
            }
        }
        let (_total_in, _total_out) = total_power::total_power_checker(&g);
        //call gauss seidel
        let solver = solvers::gauss_seidel::GaussSeidel::new();
        let _result = solver.solve(&mut g, 200, 0.001);
        //update the grap of communication library
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
                                    * g.z_base(),
                                line_type: line_type_to_cable_type(line.line_type()),
                            };
                        }
                    }
                }
            }
        }
        //add items to load flow over
        if let Some(load_flow_analytics) = graph.get_global_component_mut::<LoadFlowAnalytics>() {
            let (total_in, total_out) = total_power::total_power_checker(&g);
            let mut vec_overview = Vec::<ProductionOverview>::new();
            for (power_type, percentage_overview) in power_type_percentages(&g) {
                vec_overview.push(ProductionOverview {
                    power_type: busnode_type_to_power_type(power_type),
                    percentage: percentage_overview,
                });
                load_flow_analytics
                    .power_type_analytics
                    .push(PowerTypeAnalytics {
                        power_type: power_type.fmt(),
                        total_generators: g.generators(),
                        total_slack_nodes: g.slacks(),
                        total_load_nodes: g.loads(),
                        total_transmission_edges: g.edges().len() as i32,
                        total_nodes: g.nodes().len() as i32,
                        total_incoming_power: total_in,
                        total_outgoing_power: total_out,
                        energy_production_overview: vec_overview.clone(),
                    });
            }
        } else {
            debug!("No analytics component found");
        }

        graph.filter(Self::get_component_info())
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
