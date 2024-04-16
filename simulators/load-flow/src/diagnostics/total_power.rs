use crate::graph::electric_graph::{Graph, UndirectedGraph};
use crate::graph::node::BusType;

/// For a rough estimate of the power consumption of the system it returns the total produced (active) power and total consumed (active) power
pub fn total_power_checker(graph: &UndirectedGraph) -> (f64, f64) {
    let mut total_incoming_power = 0.0;
    let mut total_outgoing_power = 0.0;
    for node in graph.nodes() {
        if let Some(n) = graph.node(node) {
            match n.bus_type() {
                BusType::Load => {
                    total_outgoing_power += n.power().active;
                }
                BusType::Generator => {
                    total_incoming_power += n.power().active;
                }
                _ => {}
            }
        }
    }
    (total_incoming_power, total_outgoing_power)
}

#[cfg(test)]
mod tests {
    use crate::graph::{
        edge::{LineType, Transmission},
        node::{BusNode, PowerType},
    };

    use super::*;
    #[test]
    fn test_() {
        //make a graph and check if values are correct:
        let mut graph = UndirectedGraph::new(10.0, 1.0, 1.0);
        let slack = BusNode::slack();
        let node1 = BusNode::load(20.0, 10.0);

        let node2 = BusNode::generator(50.0, 10.0, PowerType::Nuclear);
        let node3 = BusNode::load(20.0, 10.0);
        graph.add_node(slack.id(), slack);
        graph.add_node(node1.id(), node1);
        graph.add_node(node2.id(), node2);
        graph.add_node(node3.id(), node3);
        graph.add_edge(
            slack.id(),
            node1.id(),
            Transmission::new(LineType::ACSRConductor, 100.0),
        );
        graph.add_edge(
            node1.id(),
            node2.id(),
            Transmission::new(LineType::ACSRConductor, 100.0),
        );
        graph.add_edge(
            node2.id(),
            node3.id(),
            Transmission::new(LineType::ACSRConductor, 100.0),
        );
        graph.add_edge(
            node3.id(),
            slack.id(),
            Transmission::new(LineType::ACSRConductor, 100.0),
        );
        let (total_incoming_power, total_outgoing_power) = total_power_checker(&graph);
        assert_eq!(total_incoming_power, 50.0);
        assert_eq!(total_outgoing_power, 40.0);
    }
}
