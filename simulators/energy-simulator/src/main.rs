use component_library::energy::{
    EnergyConsumerNode, EnergyProducerNode, EnergySlackNode, EnergyTransmissionEdge,
};
use graph_processing::vertex::{Vertex, VertexContext};
use messages::electric::DoTimestepMessage;
use nodes::electric::generic::GenericEnergyNode;
use simulator_communication::{ComponentsInfo, Graph, Server, Simulator};
use std::{collections::HashMap, env, net::SocketAddr, process::ExitCode, time::Duration};
use tracing::{error, info};
use units::electrical::utils::Unit;

use crate::{
    nodes::electric::{generator::Generator, load::Load, slack::Slack, transmission::Transmission},
    units::electrical::{admittance::Admittance, power::Power, voltage::Voltage},
};

mod messages;
mod nodes;
mod solvers;
mod units;

struct Root {
    generic: GenericEnergyNode,
}
impl Vertex for Root {
    fn do_superstep(&mut self, ctx: VertexContext) {
        ctx.elapsed_timesteps();
        if ctx.elapsed_timesteps() == 0 {
            for i in ctx.get_outgoing_neighbours::<Load>() {
                ctx.send_message(
                    i,
                    DoTimestepMessage::info_transmission(
                        1.0,
                        Unit::Power(Power::new(100.0, 50.0)),
                        self.generic.get_id(),
                        None,
                        None,
                        None,
                    ),
                );
            }
            for i in ctx.get_incoming_neighbours::<Generator>() {
                ctx.send_message(
                    i,
                    DoTimestepMessage::info_transmission(
                        1.0,
                        Unit::Power(Power::new(100.0, 50.0)),
                        self.generic.get_id(),
                        None,
                        None,
                        None,
                    ),
                );
            }
            for i in ctx.get_outgoing_neighbours::<Slack>() {
                ctx.send_message(
                    i,
                    DoTimestepMessage::info_transmission(
                        1.0,
                        Unit::Power(Power::new(100.0, 50.0)),
                        self.generic.get_id(),
                        None,
                        None,
                        None,
                    ),
                );
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    tracing_subscriber::fmt().init();

    let listen_addr = match env::var("ENERGY_SIMULATOR_ADDR")
        .unwrap_or("0.0.0.0:8101".to_string())
        .parse::<SocketAddr>()
    {
        Ok(v) => v,
        Err(err) => {
            error!("Could not parse bind address: {err}.");
            return ExitCode::FAILURE;
        }
    };

    let server = Server::<EnergySimulator>::new();

    info!("Starting energy simulator server on `{listen_addr}`.");
    if let Err(err) = server.listen_on(listen_addr).await {
        error!("Server return an error: {err}.");
        return ExitCode::FAILURE;
    }

    info!("Server exited successfully.");
    ExitCode::SUCCESS
}

///Simulator that gives a random demand and supply to a consumer and producer node respectively every timestep
pub struct EnergySimulator {
    delta_time: Duration,
}

impl Simulator for EnergySimulator {
    fn get_component_info() -> ComponentsInfo {
        ComponentsInfo::new()
            .add_required_component::<EnergyConsumerNode>()
            .add_required_component::<EnergyProducerNode>()
            .add_required_component::<EnergySlackNode>()
            .add_required_component::<EnergyTransmissionEdge>()
            .add_output_component::<EnergyTransmissionEdge>()
    }

    fn new(delta_time: std::time::Duration, _graph: Graph) -> Self {
        Self {
            // How much time advances per frame.
            delta_time,
        }
    }

    fn do_timestep(&mut self, mut graph: Graph) -> Graph {
        let mut g = graph_processing::Graph::new();

        // We have to do it in this order since the graph library needs to
        // know the types of the vertexes we add.
        let mut to_be_connected = HashMap::new();

        let mut root_vertext = Some(Root {
            generic: GenericEnergyNode::new(
                Unit::Voltage(Voltage::new(100.0, 0.0)),
                Unit::Power(Power::new(100.0, 50.0)),
            ),
        });
        let mut transmision_edge_id_to_vertex = HashMap::new();

        for (edgeid, edge, comp) in graph.get_all_edges::<EnergyTransmissionEdge>().unwrap() {
            let id = g.insert_vertex(Transmission::new(
                comp.operating_voltage,
                comp.maximum_power_capacity,
                comp.resistance_per_meter,
                comp.reactance_per_meter,
                comp.length,
                Admittance::new(comp.conductance, comp.susceptance),
            ));

            if let Some(root_vertex) = root_vertext.take() {
                let root_id = g.insert_vertex(root_vertex);
                g.insert_edge_bidirectional(root_id, id.clone()).unwrap();
            }

            to_be_connected
                .entry(edge.from)
                .or_insert(Vec::new())
                .push(id.clone());
            to_be_connected
                .entry(edge.to)
                .or_insert(Vec::new())
                .push(id.clone());
            transmision_edge_id_to_vertex.insert(edgeid, id);
        }

        for (nodeid, _, comp) in graph.get_all_nodes::<EnergyProducerNode>().unwrap() {
            let active_mwh = comp.energy_production;
            let active_wh = active_mwh * 10.0_f64.powi(6);
            let active_w = active_wh / (self.delta_time.as_secs_f64() / 3600.0);
            let id = g.insert_vertex(Generator::new(
                comp.capacity,
                Power::new(active_w, 0.0),
                Voltage::new(comp.voltage, 0.0),
            ));
            if let Some(transmision_vertexes) = to_be_connected.get(&nodeid) {
                for transmision_vertex in transmision_vertexes {
                    // Unwrap is safe as all we just added the id, and
                    // transmision_vertexes only contains valid ids
                    g.insert_edge_bidirectional(transmision_vertex.clone(), id.clone())
                        .unwrap();
                }
            }
        }

        for (nodeid, _, comp) in graph.get_all_nodes::<EnergyConsumerNode>().unwrap() {
            let active_mwh = comp.demand;
            let active_wh = active_mwh * 10.0_f64.powi(6);
            let active_w = active_wh / (self.delta_time.as_secs_f64() / 3600.0);
            let id = g.insert_vertex(Load::new(
                Power::new(active_w, 0.0),
                Voltage::new(comp.voltage, 0.0),
            ));
            if let Some(transmision_vertexes) = to_be_connected.get(&nodeid) {
                for transmision_vertex in transmision_vertexes {
                    g.insert_edge_bidirectional(transmision_vertex.clone(), id.clone())
                        .unwrap();
                }
            }
        }

        for (nodeid, _, comp) in graph.get_all_nodes::<EnergySlackNode>().unwrap() {
            let id = g.insert_vertex(Slack::new(Voltage::new(comp.voltage, 0.0), 0.0));
            if let Some(transmision_vertexes) = to_be_connected.get(&nodeid) {
                for transmision_vertex in transmision_vertexes {
                    g.insert_edge_bidirectional(transmision_vertex.clone(), id.clone())
                        .unwrap();
                }
            }
        }

        // Run supersteps needed to simulate single time_step
        for _ in 0..7 {
            g.do_superstep();
        }

        // Place updated data back into the graph
        for (edgeid, _, comp) in graph.get_all_edges_mut::<EnergyTransmissionEdge>().unwrap() {
            if let Some(vertex) = transmision_edge_id_to_vertex.get(&edgeid) {
                // Unwrap is safe as all supersteps have completed at this point
                let vertex = g.get_and_lock_vertex(vertex.clone()).unwrap();
                *comp = EnergyTransmissionEdge {
                    operating_voltage: vertex.operating_voltage,
                    maximum_power_capacity: vertex.maximum_power_capacity,
                    current_capacity: vertex.current_capacity,
                    resistance_per_meter: vertex.resistance_per_meter,
                    reactance_per_meter: vertex.resistance_per_meter,
                    length: vertex.length,
                    conductance: vertex.admittance.conductance,
                    susceptance: vertex.admittance.susceptance,
                };
            }
        }

        graph.filter(Self::get_component_info())
    }
}
