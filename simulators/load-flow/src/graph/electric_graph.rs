use crate::graph::edge::Transmission;
use crate::graph::node::BusNode;
use std::collections::HashMap;

use super::node::BusType;

#[allow(dead_code)] // Library code
#[derive(Clone, Debug)]
pub struct UndirectedGraph {
    default_graph: DefaultGraph,
}
#[allow(dead_code)] // Library code
pub struct DirectedGraph {
    default_graph: DefaultGraph,
}
#[derive(Clone, Debug)]
struct DefaultGraph {
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
    /// Return all bus nodes
    fn busnodes(&self) -> Vec<&BusNode>;
    /// Return all id pairs of edges
    fn edges(&self) -> Vec<(usize, usize)>;

    /// Get total count of nodes
    fn node_count(&self) -> usize;
    /// Get base impedance
    fn z_base(&self) -> f64 {
        self.v_base() * self.v_base() / self.s_base()
    }
    /// Get base power
    fn s_base(&self) -> f64;
    /// Get base voltage
    fn v_base(&self) -> f64;
    /// calculate optimal bases for the system
    fn calculate_optimal_bases(&self) -> (f64, f64, f64);
    /// set bases for the system
    fn set_bases(&mut self, v_base: f64, s_base: f64, p_base: f64);
    /// reset all nodes to original values
    fn reset_bases(&mut self);
    /// get number of generator nodes
    fn generators(&self) -> i32;
    /// get number of slack nodes
    fn slacks(&self) -> i32;
    /// get number of load nodes
    fn loads(&self) -> i32;
    /// get mutable access to a node
    fn get_node_mut(&mut self, node_id: usize) -> Option<&mut BusNode>;
    //reset node with existing id
    fn reset_node(&mut self, node: BusNode);
}

impl DefaultGraph {
    pub fn new(s_base: f64, v_base: f64, p_base: f64) -> Self {
        DefaultGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            v_base,
            s_base,
            p_base,
        }
    }
    fn busnodes(&self) -> Vec<&BusNode> {
        self.nodes.values().collect()
    }
    fn add_node(&mut self, id: usize, node: BusNode) {
        self.nodes.insert(id, node);
    }
    fn reset_bases(&mut self) {
        // Collect all the node IDs in a separate vector
        let node_ids: Vec<usize> = self.nodes.keys().cloned().collect();

        // Iterate through the collected node IDs
        for node_id in node_ids {
            // Get mutable access to the node
            if let Some(node) = self.nodes.get_mut(&node_id) {
                // Modify the node directly
                node.reset_pu(self.v_base, self.s_base);
            }
        }

        // Reset the base values
        self.s_base = 1.0;
        self.v_base = 1.0;
        self.p_base = 1.0;
    }
    fn set_bases(&mut self, v_base: f64, s_base: f64, p_base: f64) {
        self.v_base = v_base;
        self.s_base = s_base;
        self.p_base = p_base;
        //set all nodes to p.u values
        for i in self.nodes() {
            if let Some(node) = self.node(i) {
                let mut new_node = *node;
                new_node.set_pu(v_base, s_base);
                self.add_node(i, new_node);
            }
        }
    }
    // Method to get mutable access to a node by its ID
    fn get_node_mut(&mut self, node_id: usize) -> Option<&mut BusNode> {
        self.nodes.get_mut(&node_id)
    }
    fn calculate_optimal_bases(&self) -> (f64, f64, f64) {
        let mut v_base = 0.0;
        let mut s_base = 0.0;
        for i in self.nodes() {
            if let Some(node) = self.node(i) {
                if node.bus_type() == BusType::Slack {
                    continue;
                }
                if node.voltage().amplitude > v_base {
                    v_base = node.voltage().amplitude;
                }
                if node.power().active > s_base {
                    s_base = node.power().active;
                }
            }
        }
        (v_base, s_base, s_base)
    }
    fn node(&self, id: usize) -> Option<&BusNode> {
        self.nodes.get(&id)
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
    fn add_edge(&mut self, id1: usize, id2: usize, edge: Transmission) {
        self.edges.insert((id1, id2), edge);
    }
    fn edge(&self, id1: usize, id2: usize) -> Option<&Transmission> {
        self.edges.get(&(id1, id2))
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
    fn reset_node(&mut self, node: BusNode) {
        self.nodes.remove(&node.id());
        self.nodes.insert(node.id(), node);
    }
}

impl UndirectedGraph {
    pub fn new(s_base: f64, v_base: f64, p_base: f64) -> Self {
        UndirectedGraph {
            default_graph: DefaultGraph::new(s_base, v_base, p_base),
        }
    }
}

#[allow(dead_code)] // Library code
impl DirectedGraph {
    pub fn new(s_base: f64, v_base: f64, p_base: f64) -> Self {
        DirectedGraph {
            default_graph: DefaultGraph::new(s_base, v_base, p_base),
        }
    }
}

impl Graph for UndirectedGraph {
    fn add_node(&mut self, id: usize, node: BusNode) {
        self.default_graph.add_node(id, node);
    }

    fn add_edge(&mut self, id1: usize, id2: usize, edge: Transmission) {
        // Ensure that the lower ID always comes first to maintain consistency
        if id1 <= id2 {
            self.default_graph.add_edge(id1, id2, edge);
        } else {
            self.default_graph.add_edge(id2, id1, edge);
        }
    }

    fn node(&self, id: usize) -> Option<&BusNode> {
        self.default_graph.node(id)
    }

    fn edge(&self, id1: usize, id2: usize) -> Option<&Transmission> {
        if id1 <= id2 {
            self.default_graph.edge(id1, id2)
        } else {
            self.default_graph.edge(id2, id1)
        }
    }

    fn parents(&self, id: usize) -> Vec<usize> {
        self.default_graph.parents(id)
    }

    fn children(&self, id: usize) -> Vec<usize> {
        self.default_graph.children(id)
    }
    fn neighbors(&self, id: usize) -> Vec<usize> {
        self.default_graph.neighbors(id)
    }
    fn remove_node(&mut self, id: usize) {
        self.default_graph.remove_node(id);
    }
    fn remove_edge(&mut self, id1: usize, id2: usize) {
        self.default_graph.remove_edge(id1, id2);
    }
    fn nodes(&self) -> Vec<usize> {
        self.default_graph.nodes()
    }
    fn busnodes(&self) -> Vec<&BusNode> {
        self.default_graph.busnodes()
    }
    fn edges(&self) -> Vec<(usize, usize)> {
        self.default_graph.edges()
    }

    fn node_count(&self) -> usize {
        self.default_graph.node_count()
    }
    fn s_base(&self) -> f64 {
        self.default_graph.s_base()
    }
    fn v_base(&self) -> f64 {
        self.default_graph.v_base()
    }
    fn calculate_optimal_bases(&self) -> (f64, f64, f64) {
        self.default_graph.calculate_optimal_bases()
    }
    fn set_bases(&mut self, v_base: f64, s_base: f64, p_base: f64) {
        self.default_graph.set_bases(v_base, s_base, p_base);
    }
    fn reset_bases(&mut self) {
        self.default_graph.reset_bases();
    }

    fn generators(&self) -> i32 {
        self.default_graph.generators()
    }
    fn slacks(&self) -> i32 {
        self.default_graph.slacks()
    }
    fn loads(&self) -> i32 {
        self.default_graph.loads()
    }

    fn get_node_mut(&mut self, node_id: usize) -> Option<&mut BusNode> {
        self.default_graph.get_node_mut(node_id)
    }
    fn reset_node(&mut self, node: BusNode) {
        self.default_graph.reset_node(node);
    }
}

impl Graph for DirectedGraph {
    fn add_node(&mut self, id: usize, node: BusNode) {
        self.default_graph.add_node(id, node);
    }

    fn add_edge(&mut self, from_id1: usize, to_id2: usize, edge: Transmission) {
        self.default_graph.add_edge(from_id1, to_id2, edge);
    }

    fn node(&self, id: usize) -> Option<&BusNode> {
        self.default_graph.node(id)
    }

    fn edge(&self, from_id1: usize, to_id2: usize) -> Option<&Transmission> {
        self.default_graph.edge(from_id1, to_id2)
    }

    fn parents(&self, id: usize) -> Vec<usize> {
        self.default_graph.parents(id)
    }
    fn children(&self, id: usize) -> Vec<usize> {
        self.default_graph.children(id)
    }
    fn neighbors(&self, id: usize) -> Vec<usize> {
        self.default_graph.neighbors(id)
    }
    fn remove_node(&mut self, id: usize) {
        self.default_graph.remove_node(id);
    }
    fn remove_edge(&mut self, from_id1: usize, to_id2: usize) {
        self.default_graph.remove_edge(from_id1, to_id2);
    }
    fn nodes(&self) -> Vec<usize> {
        self.default_graph.nodes()
    }
    fn busnodes(&self) -> Vec<&BusNode> {
        self.default_graph.busnodes()
    }
    fn edges(&self) -> Vec<(usize, usize)> {
        self.default_graph.edges()
    }

    fn node_count(&self) -> usize {
        self.default_graph.node_count()
    }
    fn s_base(&self) -> f64 {
        self.default_graph.s_base()
    }
    fn v_base(&self) -> f64 {
        self.default_graph.v_base()
    }
    fn calculate_optimal_bases(&self) -> (f64, f64, f64) {
        self.default_graph.calculate_optimal_bases()
    }
    fn set_bases(&mut self, v_base: f64, s_base: f64, p_base: f64) {
        self.default_graph.set_bases(v_base, s_base, p_base);
    }
    fn reset_bases(&mut self) {
        self.default_graph.reset_bases();
    }
    fn generators(&self) -> i32 {
        self.default_graph.generators()
    }

    fn slacks(&self) -> i32 {
        self.default_graph.slacks()
    }
    fn loads(&self) -> i32 {
        self.default_graph.loads()
    }
    // Method to get mutable access to a node by its ID
    fn get_node_mut(&mut self, node_id: usize) -> Option<&mut BusNode> {
        self.default_graph.get_node_mut(node_id)
    }
    fn reset_node(&mut self, node: BusNode) {
        self.default_graph.reset_node(node);
    }
}

#[cfg(test)]
// Path: simulators/energy-simulator/src/messages/current_checker.rs
mod tests {

    // Add the missing graphlib module
    use crate::graph::edge::{LineType, Transmission};
    use crate::graph::electric_graph::{DirectedGraph, Graph, UndirectedGraph};
    use crate::graph::node::{BusNode, PowerType};
    use crate::units::voltage::Voltage;

    pub fn create_test_graph() -> UndirectedGraph {
        let id0 = BusNode::generator(100.0, 100.0, PowerType::Battery);

        let id1 = BusNode::generator(200.0, 200.0, PowerType::Fossil);

        let id2 = BusNode::load(300.0, 0.0);
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
        assert_eq!(graph.node_count(), 6);
        graph.remove_node(pq1.id());
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.parents(id).len(), 0);
    }
    #[test]
    fn test_bases() {
        let mut graph = create_test_graph();
        let (v_base, s_base, p_base) = graph.calculate_optimal_bases();
        assert_eq!(v_base, 200.0);
        assert_eq!(s_base, 300.0);
        assert_eq!(p_base, 300.0);

        graph.set_bases(v_base, s_base, v_base);
        assert_eq!(graph.v_base(), v_base);
        assert_eq!(graph.s_base(), s_base);
        graph.reset_bases();
        assert_eq!(graph.v_base(), 1.0);
        assert_eq!(graph.s_base(), 1.0);
    }
    #[test]
    fn test_mut_node() {
        let mut graph = DirectedGraph::new(10.0, 1.0, 1.0);
        let slack = BusNode::slack();
        let pq1 = BusNode::load(1.0, 0.0);
        graph.add_node(slack.id(), slack);
        graph.add_node(pq1.id(), pq1);
        let id = pq1.id();
        let node = graph.get_node_mut(id).unwrap();
        node.set_voltage(Voltage::new(100.0, 0.0));
        let mut update = *node;
        update.set_voltage(Voltage::new(200.0, 0.0));
        graph.add_node(update.id(), update);
        assert_eq!(graph.node_count(), 2);
    }
}
