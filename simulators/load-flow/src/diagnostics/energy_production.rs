use crate::graph::{
    electric_graph::{Graph, UndirectedGraph},
    node::{BusType, PowerType},
};
use std::collections::HashMap;

pub fn power_type_percentages(graph: &UndirectedGraph) -> HashMap<PowerType, f64> {
    // Count total number of generator nodes
    let total_generators = graph.generators() as f64;

    // Fill power types and calculate percentages
    let mut power_types: HashMap<PowerType, f64> = HashMap::new();
    for node in graph.nodes() {
        if let Some(n) = graph.node(node) {
            if n.bus_type() == BusType::Generator {
                let count = power_types.entry(n.power_type()).or_insert(0.0);
                *count += 1.0;
            }
        }
    }
    // Calculate percentages
    for count in power_types.values_mut() {
        *count = (*count / total_generators) * 100.0; // Convert count to percentage
    }

    power_types
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::graph::node::BusNode;
    #[test]
    fn test_percentages() {
        //make new graph
        let mut graph = UndirectedGraph::new(10.0, 1.0, 1.0);
        let slack = BusNode::load(0.0, 0.0);
        let node1 = BusNode::generator(10.0, 10.0, PowerType::Solar);
        let node2 = BusNode::generator(10.0, 10.0, PowerType::Wind);
        let node3 = BusNode::generator(10.0, 10.0, PowerType::Hydro);
        let node4 = BusNode::generator(10.0, 10.0, PowerType::Nuclear);
        let node5 = BusNode::generator(10.0, 10.0, PowerType::Battery);
        let node6 = BusNode::generator(10.0, 10.0, PowerType::Storage);
        let node7 = BusNode::generator(10.0, 10.0, PowerType::Fossil);
        let node8 = BusNode::generator(10.0, 10.0, PowerType::Renewable);
        let node9 = BusNode::generator(10.0, 10.0, PowerType::Solar);
        let node10 = BusNode::generator(10.0, 10.0, PowerType::Wind);
        let node11 = BusNode::generator(10.0, 10.0, PowerType::Hydro);
        let node12 = BusNode::generator(10.0, 10.0, PowerType::Nuclear);
        let node13 = BusNode::generator(10.0, 10.0, PowerType::Solar);
        let node14 = BusNode::generator(10.0, 10.0, PowerType::Wind);
        let node15 = BusNode::generator(10.0, 10.0, PowerType::Hydro);

        graph.add_node(slack.id(), slack);
        graph.add_node(node1.id(), node1);
        graph.add_node(node2.id(), node2);
        graph.add_node(node3.id(), node3);
        graph.add_node(node4.id(), node4);
        graph.add_node(node5.id(), node5);
        graph.add_node(node6.id(), node6);
        graph.add_node(node7.id(), node7);
        graph.add_node(node8.id(), node8);
        graph.add_node(node9.id(), node9);
        graph.add_node(node10.id(), node10);
        graph.add_node(node11.id(), node11);
        graph.add_node(node12.id(), node12);
        graph.add_node(node13.id(), node13);
        graph.add_node(node14.id(), node14);
        graph.add_node(node15.id(), node15);
        let p = power_type_percentages(&graph);
        assert_eq!(p[&PowerType::Solar], 20.0);
        assert_eq!(p[&PowerType::Wind], 20.0);
        assert_eq!(p[&PowerType::Hydro], 20.0);
        assert_eq!(p[&PowerType::Nuclear], 13.333333333333334);
        assert_eq!(p[&PowerType::Battery], 6.666666666666667);
        assert_eq!(p[&PowerType::Storage], 6.666666666666667);
        assert_eq!(p[&PowerType::Fossil], 6.666666666666667);
        assert_eq!(p[&PowerType::Renewable], 6.666666666666667);
    }
}
