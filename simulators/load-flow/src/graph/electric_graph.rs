use crate::graph::edge::Transmission;
use crate::graph::node::BusNode;
use std::collections::HashMap;

use super::node::BusType;

#[allow(dead_code)] // Library code
#[derive(Clone, Debug)]
pub struct UndirectedGraph {
    nodes: HashMap<usize, BusNode>,
    edges: HashMap<(usize, usize), Transmission>,
    v_base: f64,
    s_base: f64,
    p_base: f64,
}
#[allow(dead_code)] // Library code
pub struct DirectedGraph {
    nodes: HashMap<usize, BusNode>,
    edges: HashMap<(usize, usize), Transmission>,
    v_base: f64,
    s_base: f64,
    p_base: f64,
}

/// Trait for easy swap between a directed and undirected graph
pub trait Graph {
    /// Add a node with corresponding id
    fn add_node(&mut self, id: usize, node: BusNode);

    /// Add an edge between id1 and id2
    fn add_edge(&mut self, id1: usize, id2: usize, edge: Transmission);

    /// Get the node at id
    fn node(&self, id: usize) -> Option<&BusNode>;

    /// Get the edge between id1 and id2
    fn edge(&self, id1: usize, id2: usize) -> Option<&Transmission>;

    /// Get the parents of a node (Equal to children in an undirected graph)
    fn parents(&self, id: usize) -> Vec<usize>;

    /// Get all children of a node (Equal to parents in an undirected graph)
    fn children(&self, id: usize) -> Vec<usize>;

    /// Get all nodes connected to the node with id (Union of parents and children)
    fn neighbors(&self, id: usize) -> Vec<usize>;

    /// Remove the node at id
    fn remove_node(&mut self, id: usize);

    /// Remove the edge between nodes at id1 and id2
    fn remove_edge(&mut self, id1: usize, id2: usize);

    /// Return all node ids
    fn nodes(&self) -> Vec<usize>;

    /// Return all id pairs of edges
    fn edges(&self) -> Vec<(usize, usize)>;

    /// Get total count of nodes
    fn node_count(&self) -> usize;
    /// Get base impedance
    fn z_base(&self) -> f64 {
        self.v_base() * self.v_base() / self.s_base()
    }
    /// Get base voltage
    fn v_base(&self) -> f64;
    /// Get base power
    fn s_base(&self) -> f64;
    //get number of generator nodes
    fn generators(&self) -> i32;
    //get number of slack nodes
    fn slacks(&self) -> i32;
    //get number of load nodes
    fn loads(&self) -> i32;
}

impl UndirectedGraph {
    pub fn new(s_base: f64, v_base: f64, p_base: f64) -> Self {
        UndirectedGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            v_base,
            s_base,
            p_base,
        }
    }
}

#[allow(dead_code)] // Library code
impl DirectedGraph {
    pub fn new(s_base: f64, v_base: f64, p_base: f64) -> Self {
        DirectedGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            v_base,
            s_base,
            p_base,
        }
    }
}

impl Graph for UndirectedGraph {
    fn generators(&self) -> i32 {
        self.nodes
            .values()
            .filter(|node| node.bus_type() == BusType::Generator)
            .count() as i32
    }
    fn loads(&self) -> i32 {
        self.nodes
            .values()
            .filter(|node| node.bus_type() == BusType::Load)
            .count() as i32
    }
    fn slacks(&self) -> i32 {
        self.nodes
            .values()
            .filter(|node| node.bus_type() == BusType::Slack)
            .count() as i32
    }
    fn add_node(&mut self, id: usize, node: BusNode) {
        self.nodes.insert(id, node);
    }

    fn add_edge(&mut self, id1: usize, id2: usize, edge: Transmission) {
        // Ensure that the lower ID always comes first to maintain consistency
        if id1 <= id2 {
            self.edges.insert((id1, id2), edge);
        } else {
            self.edges.insert((id2, id1), edge);
        }
    }

    fn node(&self, id: usize) -> Option<&BusNode> {
        self.nodes.get(&id)
    }

    fn edge(&self, id1: usize, id2: usize) -> Option<&Transmission> {
        if id1 <= id2 {
            self.edges.get(&(id1, id2))
        } else {
            self.edges.get(&(id2, id1))
        }
    }

    fn parents(&self, id: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter_map(|((from, to), _)| if *to == id { Some(*from) } else { None })
            .collect()
    }

    fn children(&self, id: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter_map(|((from, to), _)| if *from == id { Some(*to) } else { None })
            .collect()
    }
    fn neighbors(&self, id: usize) -> Vec<usize> {
        let mut neighbors = self.parents(id);
        neighbors.extend(self.children(id));
        neighbors
    }
    fn remove_node(&mut self, id: usize) {
        self.nodes.remove(&id);
        self.edges.retain(|(from, to), _| *from != id && *to != id);
    }
    fn remove_edge(&mut self, id1: usize, id2: usize) {
        self.edges.remove(&(id1, id2));
    }
    fn nodes(&self) -> Vec<usize> {
        self.nodes.keys().cloned().collect()
    }
    fn edges(&self) -> Vec<(usize, usize)> {
        self.edges.keys().cloned().collect()
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }
    fn s_base(&self) -> f64 {
        self.s_base
    }
    fn v_base(&self) -> f64 {
        self.v_base
    }
}

impl Graph for DirectedGraph {
    fn generators(&self) -> i32 {
        self.nodes
            .values()
            .filter(|node| node.bus_type() == BusType::Generator)
            .count() as i32
    }
    fn loads(&self) -> i32 {
        self.nodes
            .values()
            .filter(|node| node.bus_type() == BusType::Load)
            .count() as i32
    }
    fn slacks(&self) -> i32 {
        self.nodes
            .values()
            .filter(|node| node.bus_type() == BusType::Slack)
            .count() as i32
    }
    fn add_node(&mut self, id: usize, node: BusNode) {
        self.nodes.insert(id, node);
    }

    fn add_edge(&mut self, from_id1: usize, to_id2: usize, edge: Transmission) {
        self.edges.insert((from_id1, to_id2), edge);
    }

    fn node(&self, id: usize) -> Option<&BusNode> {
        self.nodes.get(&id)
    }

    fn edge(&self, from_id1: usize, to_id2: usize) -> Option<&Transmission> {
        self.edges.get(&(from_id1, to_id2))
    }

    fn parents(&self, id: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter_map(|((from, to), _)| if *to == id { Some(*from) } else { None })
            .collect()
    }
    fn children(&self, id: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter_map(|((from, to), _)| if *from == id { Some(*to) } else { None })
            .collect()
    }
    fn neighbors(&self, id: usize) -> Vec<usize> {
        let mut neighbors = self.parents(id);
        neighbors.extend(self.children(id));
        neighbors
    }
    fn remove_node(&mut self, id: usize) {
        self.nodes.remove(&id);
        self.edges.retain(|(from, to), _| *from != id && *to != id);
    }
    fn remove_edge(&mut self, from_id1: usize, to_id2: usize) {
        self.edges.remove(&(from_id1, to_id2));
    }
    fn nodes(&self) -> Vec<usize> {
        self.nodes.keys().cloned().collect()
    }
    fn edges(&self) -> Vec<(usize, usize)> {
        self.edges.keys().cloned().collect()
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }
    fn s_base(&self) -> f64 {
        self.s_base
    }
    fn v_base(&self) -> f64 {
        self.v_base
    }
}

#[cfg(test)]
// Path: simulators/energy-simulator/src/messages/current_checker.rs
mod tests {

    // Add the missing graphlib module
    use crate::graph::edge::{LineType, Transmission};
    use crate::graph::electric_graph::{DirectedGraph, Graph, UndirectedGraph};
    use crate::graph::node::{BusNode, PowerType};

    pub fn create_test_graph() -> UndirectedGraph {
        let id0 = BusNode::generator(1.0, 1.0, PowerType::Battery);

        let id1 = BusNode::generator(1.0, 1.0, PowerType::Fossil);

        let id2 = BusNode::load(1.0, 0.0);
        let l1 = Transmission::new(LineType::ACSRConductor, 200.0);
        let l2 = Transmission::new(LineType::ACSRConductor, 200.0);
        let mut graph = UndirectedGraph::new(10.0, 1.0, 1.0);
        graph.add_node(0, id0);
        graph.add_node(1, id1);
        graph.add_node(2, id2);
        graph.add_edge(0, 1, l1);
        graph.add_edge(1, 2, l2);
        graph
    }

    #[test]
    fn test_graph() {
        let graph = create_test_graph();
        let _l1 = graph.edge(0, 1).unwrap();
        let _l2 = graph.edge(1, 2).unwrap();
        assert_eq!(graph.nodes().len(), 3);
    }
    #[test]
    fn test_parents() {
        let graph = create_test_graph();
        let parents = graph.parents(1);
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0], 0);
    }
    #[test]
    fn test_children() {
        let graph = create_test_graph();
        let children = graph.children(1);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], 2);
    }
    #[test]
    fn test_directed() {
        let mut graph = DirectedGraph::new(10.0, 1.0, 1.0);
        let slack = BusNode::slack();
        let pq1 = BusNode::load(1.0, 0.0);
        let pq2 = BusNode::load(1.0, 0.0);
        let pq3 = BusNode::load(1.0, 0.0);
        let pq4 = BusNode::load(1.0, 0.0);
        let pq5 = BusNode::load(1.0, 0.0);
        let l1 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l2 = Transmission::new(LineType::ACSRConductor, 100.0);
        let l3 = Transmission::new(LineType::ACSRConductor, 100.0);
        graph.add_node(slack.id(), slack);
        graph.add_node(pq1.id(), pq1);
        graph.add_node(pq2.id(), pq2);
        graph.add_node(pq3.id(), pq3);
        graph.add_node(pq4.id(), pq4);
        graph.add_node(pq5.id(), pq5);
        graph.add_edge(slack.id(), pq1.id(), l1);
        graph.add_edge(slack.id(), pq2.id(), l2);
        graph.add_edge(pq1.id(), pq2.id(), l3);
        let id = pq1.id();
        assert_eq!(graph.parents(id).len(), 1);
        assert_eq!(graph.children(id).len(), 1);
        assert_eq!(graph.neighbors(id).len(), 2);
    }
}
