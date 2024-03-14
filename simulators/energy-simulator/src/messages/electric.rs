use crate::{
    nodes::electric::{
        generator::Generator,
        generic_node::BusType,
        generic_node::{EnergyNode, GenericEnergyNode},
        load::Load,
        slack::Slack,
        transmission_node::Transmission,
    },
    solvers::electric_eq_solver::converge_gauss_seidel,
    units::electrical::{
        admittance::Admittance,
        power::{self, Power},
        utils::Unit,
        voltage::Voltage,
    },
};
use graph_processing::{
    message::{Message, MessageHandler},
    vertex::VertexContext,
};

use std::collections::HashMap;

const MAX_ITERATIONS: usize = 100;
const TOLERANCE: f64 = 1e-6;
#[derive(Clone, Debug)]
pub struct DoTimestepMessage {
    pub timestep: f64,
    pub incoming_power: Unit,
    pub incoming_id: usize,
    pub info: Option<HashMap<usize, Unit>>,
    pub incoming_angle: Option<f64>,
    pub slack: Option<HashMap<usize, (Unit, f64)>>,
}
impl DoTimestepMessage {
    pub fn info_transmission(
        timestep: f64,
        incoming: Unit,
        incoming_id: usize,
        info: Option<HashMap<usize, Unit>>,
        incoming_angle: Option<f64>,
        slack_info: Option<HashMap<usize, (Unit, f64)>>,
    ) -> Self {
        DoTimestepMessage {
            timestep,
            incoming_power: incoming,
            incoming_id,
            info,
            incoming_angle,
            slack: slack_info,
        }
    }
}
impl Message for DoTimestepMessage {}

/**
 * This function is called when a message is received by a producer or consumer node
 * It will add the incoming line to the node and check if all neighbours have sent info
 * If so, it will calculate the new voltage and power of the node and send it to all neighbours
 */
fn bus_handler(
    _ctx: VertexContext,
    message: DoTimestepMessage,
    node: &mut GenericEnergyNode,
    power: Power,
    angle: Option<f64>,
) -> (power::Power, bool) {
    let id_send = node.get_id();
    let _added = node.get_nr_lines();
    node.add_line(message.incoming_id, message.incoming_power);
    let mut converged = false;
    if message.info.is_some() {
        for (k, v) in message.info.unwrap() {
            node.add_neighbour(k, v);
        }
    }

    if message.slack.is_some() {
        for (k, v) in message.slack.unwrap().iter() {
            node.add_slack_neighbour(*k, v.0, v.1);
        }
    }
    let result = (
        power,
        node.get_voltage().to_voltage().unwrap(),
        node.get_bus_type().clone(),
    );
    //check if all neighbours have sent info
    if node.get_nr_neighbours() + node.get_nr_slack_neighbours()
        >= _ctx.get_outgoing_neighbours::<Transmission>().count()
    {
        //call the solver to calculate new power and voltage
        let n1: HashMap<usize, Voltage> = node
            .get_neighbours()
            .iter()
            .filter_map(|(&key, unit)| (*unit).to_voltage().map(|voltage| (key, voltage)))
            .collect();

        // Transform all values to Admittance and collect into a new map
        let n2: HashMap<usize, Admittance> = node
            .get_lines()
            .iter()
            .filter_map(|(&key, unit)| (*unit).to_admittance().map(|admittance| (key, admittance)))
            .collect();
        let old_result = result.clone();
        let mut result =
            converge_gauss_seidel(TOLERANCE, MAX_ITERATIONS as i32, n2, n1, result.clone(), 0);
        converged = result.3 < MAX_ITERATIONS as i32;
        if result.3 >= MAX_ITERATIONS as i32 {
            (result.0, result.1, result.2) = (old_result.0, old_result.1, old_result.2);
        }
        //remove all old values of neighbours
        node.clear_neighbours();
        node.clear_slack_neighbours();
        match node.get_bus_type().clone() {
            BusType::Generator => {
                node.set_unit(Unit::Voltage(Voltage::new(
                    node.get_voltage().to_voltage().unwrap().amplitude,
                    result.1.angle,
                )));
            }
            _ => {
                node.set_unit(Unit::Voltage(Voltage::new(
                    result.1.amplitude,
                    result.1.angle,
                )));
            }
        }
    }

    for i in _ctx.get_incoming_neighbours::<Transmission>() {
        _ctx.send_message(
            i,
            DoTimestepMessage::info_transmission(
                message.timestep,
                node.get_voltage(),
                id_send,
                None,
                angle,
                None,
            ),
        );
    }
    (result.0, converged)
}

impl MessageHandler<DoTimestepMessage> for Slack {
    fn handle(&mut self, _ctx: VertexContext, message: DoTimestepMessage) {
        //calculate phase angle between bus and the line
        //call the distributed_round function to calculate the new power and voltage
        //update the power/voltage and angle of the bus
        //send neighbours the update values
        //send to neihgbour it is a slackbus

        let mut s = self.get_phase_angle();
        let sending_power = self.generic_node.get_power().to_power().unwrap();
        let p = bus_handler(
            _ctx,
            message,
            &mut self.generic_node,
            sending_power,
            Some(s),
        );
        if p.1 {
            self.generic_node.set_unit(Unit::Power(p.0));
            s = (p.0.active.powi(2) + p.0.reactive.powi(2)).sqrt();
            self.set_phase_angle(p.0.active, s);
        }
    }
}

impl MessageHandler<DoTimestepMessage> for Load {
    fn handle(&mut self, _ctx: VertexContext, message: DoTimestepMessage) {
        let sending_power = self.generic_node.get_power().to_power().unwrap();
        let p = bus_handler(_ctx, message, &mut self.generic_node, sending_power, None);
        if p.1 {
            self.generic_node.set_unit(Unit::Power(p.0));
        }
    }
}

impl MessageHandler<DoTimestepMessage> for Generator {
    fn handle(&mut self, _ctx: VertexContext, message: DoTimestepMessage) {
        let sending_power = self.generic_node.get_power().to_power().unwrap();
        let p = bus_handler(_ctx, message, &mut self.generic_node, sending_power, None);
        if p.1 {
            self.generic_node.set_unit(Unit::Power(p.0));
        }
    }
}

impl MessageHandler<DoTimestepMessage> for Transmission {
    fn handle(&mut self, _ctx: VertexContext, message: DoTimestepMessage) {
        //get all neighbours that are consumers and producers
        //calculate the power that reaches them, given the length the node travels
        let consumers = _ctx.get_outgoing_neighbours::<Load>();
        let producers = _ctx.get_outgoing_neighbours::<Generator>();
        let slack = _ctx.get_outgoing_neighbours::<Slack>();
        if message.incoming_angle.is_some() {
            self.generic_node.add_slack_neighbour(
                message.incoming_id,
                message.incoming_power,
                message.incoming_angle.unwrap() - self.get_angle(),
            );
        } else {
            self.generic_node
                .add_neighbour(message.incoming_id, message.incoming_power);
        }

        let line_info = Unit::Admittance(self.admittance);
        let mut neighbour_info = HashMap::new();
        let mut send = Voltage::new(0.0, 0.0);
        let mut receive = Voltage::new(0.0, 0.0);
        let total_neighbours =
            self.generic_node.get_nr_neighbours() + self.generic_node.get_nr_slack_neighbours();
        if total_neighbours >= 2 && self.generic_node.get_nr_neighbours() >= 1 {
            for (k, v) in self.generic_node.get_neighbours() {
                if *k != message.incoming_id {
                    neighbour_info.insert(self.generic_node.get_id(), *v);
                }
                if (*v).to_voltage().unwrap().amplitude > 0.0 {
                    send = (*v).to_voltage().unwrap();
                }
                if (*v).to_voltage().unwrap().amplitude < 0.0 {
                    receive = (*v).to_voltage().unwrap();
                }
            }
        }
        let mut slack_neighbour_info: HashMap<usize, (Unit, f64)> = HashMap::new();
        if self.generic_node.get_nr_slack_neighbours() >= 1 && total_neighbours >= 2 {
            for (k, v) in self.generic_node.get_slack_neighbours() {
                if *k != message.incoming_id {
                    slack_neighbour_info.insert(self.generic_node.get_id(), *v);
                }
                if v.0.to_voltage().unwrap().amplitude > 0.0 {
                    send = v.0.to_voltage().unwrap();
                }
                if v.0.to_voltage().unwrap().amplitude < 0.0 {
                    receive = v.0.to_voltage().unwrap();
                }
            }
        }
        self.current = self.calculate_sending_current(send, receive);
        for n in consumers {
            _ctx.send_message(
                n,
                DoTimestepMessage::info_transmission(
                    message.timestep,
                    line_info,
                    self.generic_node.get_id(),
                    Some(neighbour_info.clone()),
                    None,
                    Some(slack_neighbour_info.clone()),
                ),
            );
        }

        for n in producers {
            _ctx.send_message(
                n,
                DoTimestepMessage::info_transmission(
                    message.timestep,
                    line_info,
                    self.generic_node.get_id(),
                    Some(neighbour_info.clone()),
                    None,
                    Some(slack_neighbour_info.clone()),
                ),
            );
        }
        for n in slack {
            _ctx.send_message(
                n,
                DoTimestepMessage::info_transmission(
                    message.timestep,
                    line_info,
                    self.generic_node.get_id(),
                    Some(neighbour_info.clone()),
                    None,
                    Some(slack_neighbour_info.clone()),
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::nodes::electric::generic_node::GenericEnergyNode;
    use crate::units::electrical::power::Power;
    use crate::units::electrical::voltage::Voltage; // Add this line

    use super::*;
    use graph_processing::vertex::Vertex;
    use graph_processing::Graph;
    #[test]
    fn test_graph() {
        struct TestVertex {
            generic_node: GenericEnergyNode,
        }
        impl Vertex for TestVertex {
            fn do_superstep(&mut self, ctx: VertexContext) {
                ctx.elapsed_timesteps();
                if ctx.elapsed_timesteps() == 0 {
                    for i in ctx.get_outgoing_neighbours::<Load>() {
                        ctx.send_message(
                            i,
                            DoTimestepMessage::info_transmission(
                                1.0,
                                Unit::Power(Power::new(100.0, 50.0)),
                                self.generic_node.get_id(),
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
                                self.generic_node.get_id(),
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
                                self.generic_node.get_id(),
                                None,
                                None,
                                None,
                            ),
                        );
                    }
                }
            }
        }
        impl MessageHandler<DoTimestepMessage> for TestVertex {
            fn handle(&mut self, _ctx: VertexContext, message: DoTimestepMessage) {
                for i in _ctx.get_outgoing_neighbours::<Load>() {
                    _ctx.send_message(
                        i,
                        DoTimestepMessage::info_transmission(
                            message.timestep,
                            Unit::Power(Power::new(100.0, 50.0)),
                            0,
                            None,
                            None,
                            None,
                        ),
                    );
                }
                for i in _ctx.get_incoming_neighbours::<Load>() {
                    _ctx.send_message(
                        i,
                        DoTimestepMessage::info_transmission(
                            message.timestep,
                            Unit::Power(Power::new(100.0, 50.0)),
                            0,
                            None,
                            None,
                            None,
                        ),
                    );
                }
            }
        }

        let mut graph = Graph::new();

        let _t = Transmission::new(
            110.0,
            300.0,
            0.00015,
            0.0007,
            50000.0,
            Admittance::new(0.0, 0.0),
        );
        let test_vertext = TestVertex {
            generic_node: GenericEnergyNode::new(
                Unit::Voltage(Voltage::new(100.0, 0.0)),
                Unit::Power(Power::new(100.0, 50.0)),
            ),
        };
        let id1 = graph.insert_vertex(test_vertext);
        let id4 = graph.insert_vertex(Load {
            generic_node: GenericEnergyNode::new(
                Unit::Voltage(Voltage::new(100.0, 0.0)),
                Unit::Power(Power::new(100.0, 50.0)),
            ),
        });
        let id2 = graph.insert_vertex(Transmission::new(
            100.0,
            50.0,
            1.0,
            2.0,
            10.0,
            Admittance::new(20.0, 10.0),
        ));

        let id3 = graph.insert_vertex(Transmission::new(
            50.0,
            10.0,
            0.02,
            2.0,
            10.0,
            Admittance::new(40.0, 50.0),
        ));
        let id5 = graph.insert_vertex(Transmission::new(
            200.0,
            100.0,
            1.0,
            2.0,
            10.0,
            Admittance::new(30.0, 30.0),
        ));
        let id6 = graph.insert_vertex(Load::new(Power::new(20.0, 30.9), Voltage::new(20.0, 90.0)));
        let id7 = graph.insert_vertex(Generator::new(
            20.0,
            Power::new(20.0, 30.9),
            Voltage::new(20.0, 90.0),
        ));
        let id8 = graph.insert_vertex(Generator::new(
            20.0,
            Power::new(220.0, 330.9),
            Voltage::new(120.0, 190.0),
        )); //connect nodes

        let _id9 = graph.insert_vertex(Slack::new(Voltage::new(120.0, 190.0), 0.30));
        let _id10 = graph.insert_vertex(Slack::new(Voltage::new(120.0, 190.0), 0.10));
        let _ = graph.insert_edge_bidirectional(id1.clone(), id4.clone());
        _ = graph.insert_edge_bidirectional(id4.clone(), id2.clone());
        _ = graph.insert_edge_bidirectional(id4.clone(), id3.clone());
        _ = graph.insert_edge_bidirectional(id4.clone(), id5.clone());
        _ = graph.insert_edge_bidirectional(id2.clone(), id6.clone());
        _ = graph.insert_edge_bidirectional(id3.clone(), id7.clone());
        _ = graph.insert_edge_bidirectional(id5.clone(), id8.clone());
        for _ in 0..7 {
            graph.do_superstep();
        }
    }
}
