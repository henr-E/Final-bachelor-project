//! The main datatypes used to transfer data from and to the manager.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::Hash,
    iter,
};

use crate::{component::Component, proto, simulator::ComponentsInfo, Value};

/// A pointer to a specific Node in the [`Graph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

/// A pointer to a specific Edge in the [`Graph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeId(usize);

/// A single node in the world with a location.
#[derive(Debug, PartialEq)]
pub struct Node {
    /// The latitude of this node.
    pub latitude: f64,
    /// The longitude of this node.
    pub longitude: f64,
}

/// A edge linking two nodes in the graph.
///
/// All edges are saved as directional, but some are more logical to interpret als biderectional.
#[derive(Debug, PartialEq, Eq)]
pub struct Edge {
    /// The index of the node of origin.
    pub from: NodeId,
    /// The index of the destination node.
    pub to: NodeId,
}

pub(crate) struct ComponentStorage<C: Component> {
    /// The usizes refernce specific node/edge by there position in the respective Vec
    pub(crate) components: Vec<(usize, C)>,
}

#[derive(Debug)]
struct ComponentStorageMap {
    components: HashMap<TypeId, Box<dyn Any + Send>>,
}

impl ComponentStorageMap {
    fn downcast<C: Component>(&self) -> Option<&ComponentStorage<C>> {
        let component_storage = self.components.get(&TypeId::of::<C>())?;
        match component_storage.downcast_ref::<ComponentStorage<C>>() {
            Some(c) => Some(c),
            None => unreachable!("ComponentStorage had ComponentStorage with wrong type, This is a bug in the simulator communication lib"),
        }
    }

    fn downcast_mut<C: Component>(&mut self) -> Option<&mut ComponentStorage<C>> {
        let component_storage = self.components.get_mut(&TypeId::of::<C>())?;
        match component_storage.downcast_mut::<ComponentStorage<C>>() {
            Some(c) => Some(c),
            None => unreachable!("ComponentStorage had ComponentStorage with wrong type, This is a bug in the simulator communication lib"),
        }
    }
}

/// The main datastructure used to send data to and from the manager.
///
/// Contains a list of nodes containing multiple [`Component`]s.
/// Nodes can be connected with edges, all containing exactly one [`Component`].
///
/// There can also be global [`Component`]s.
#[derive(Debug)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,

    node_components: ComponentStorageMap,
    edge_components: ComponentStorageMap,
    global_components: ComponentStorageMap,

    _edge_manager_id_to_index: HashMap<u64, EdgeId>,
    edge_index_to_manager_id: HashMap<EdgeId, u64>,
    _node_manager_id_to_index: HashMap<u64, NodeId>,
    node_index_to_manager_id: HashMap<NodeId, u64>,
}

impl Graph {
    /// Get all nodes with the [`Component`] `C` in this graph, together with this component.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn get_all_nodes<C: Component>(&self) -> Option<impl Iterator<Item = (NodeId, &Node, &C)>> {
        let iter = self
            .node_components
            .downcast()?
            .components
            .iter()
            .filter_map(|(index, component)| {
                self.nodes
                    .get(*index)
                    .map(|node| (NodeId(*index), node, component))
            });

        Some(iter)
    }

    /// Get all edges with the [`Component`] `C` in this graph, together with this component.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn get_all_edges<C: Component>(&self) -> Option<impl Iterator<Item = (EdgeId, &Edge, &C)>> {
        let iter = self
            .edge_components
            .downcast()?
            .components
            .iter()
            .filter_map(|(index, component)| {
                self.edges
                    .get(*index)
                    .map(|edge| (EdgeId(*index), edge, component))
            });

        Some(iter)
    }

    /// Get all nodes with the [`Component`] `C` in this graph, together with this component.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn get_all_nodes_mut<C: Component>(
        &mut self,
    ) -> Option<impl Iterator<Item = (NodeId, &Node, &mut C)>> {
        let iter = self
            .node_components
            .downcast_mut()?
            .components
            .iter_mut()
            .filter_map(|(index, ref mut component)| {
                self.nodes
                    .get(*index)
                    .map(|node: &Node| (NodeId(*index), node, component))
            });

        Some(iter)
    }

    /// Get all edges with the [`Component`] `C` in this graph, together with this component.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn get_all_edges_mut<C: Component>(
        &mut self,
    ) -> Option<impl Iterator<Item = (EdgeId, &Edge, &mut C)>> {
        let iter = self
            .edge_components
            .downcast_mut()?
            .components
            .iter_mut()
            .filter_map(|(index, ref mut component)| {
                self.edges
                    .get(*index)
                    .map(|edge: &Edge| (EdgeId(*index), edge, component))
            });

        Some(iter)
    }

    /// Get the [`Component`] `C` for a single node.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn get_node_component<C: Component>(&self, id: NodeId) -> Option<&C> {
        let components = &self.node_components.downcast()?.components;
        if let Ok(i) = components.binary_search_by(|(correct_index, _)| correct_index.cmp(&id.0)) {
            return Some(&components[i].1);
        }

        None
    }

    /// Get the [`Component`] `C` for a single edge.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn get_edge_component<C: Component>(&self, id: EdgeId) -> Option<&C> {
        let components = &self.edge_components.downcast()?.components;
        if let Ok(i) = components.binary_search_by(|(correct_index, _)| correct_index.cmp(&id.0)) {
            return Some(&components[i].1);
        }

        None
    }

    /// Get the [`Component`] `C` for a single node.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn get_node_component_mut<C: Component>(&mut self, id: NodeId) -> Option<&mut C> {
        let components = &mut self.node_components.downcast_mut()?.components;
        if let Ok(i) = components.binary_search_by(|(correct_index, _)| correct_index.cmp(&id.0)) {
            return Some(&mut components[i].1);
        }

        None
    }

    /// Get the [`Component`] `C` for a single edge.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn get_edge_component_mut<C: Component>(&mut self, id: EdgeId) -> Option<&mut C> {
        let components = &mut self.edge_components.downcast_mut()?.components;
        if let Ok(i) = components.binary_search_by(|(correct_index, _)| correct_index.cmp(&id.0)) {
            return Some(&mut components[i].1);
        }

        None
    }

    /// Get the [`Component`] `C` from the global components.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn get_global_component<C: Component>(&self) -> Option<&C> {
        let component = &self.global_components.downcast()?.components;
        match &component[..] {
            [] => None,
            [c] => Some(&c.1),
            _ => {
                unreachable!("All global components should have exactly one or zero component in storage, This is a bug in the simulator communication lib")
            }
        }
    }

    /// Get the [`Component`] `C` from the global components.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn get_global_component_mut<C: Component>(&mut self) -> Option<&mut C> {
        let component = &mut self.global_components.downcast_mut()?.components;
        match &mut component[..] {
            [] => None,
            [c] => Some(&mut c.1),
            _ => {
                unreachable!("All global components should have exactly one or zero component in storage, This is a bug in the simulator communication lib")
            }
        }
    }

    /// Returns an iterator iterating over all the nodes connected *from*
    /// the node with id `from` via an edge with the [`Component`] `C`.
    /// Giving a tupple of the Edge component, NodeId, and node itself.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn neighbors_directed<C: Component>(
        &self,
        from: NodeId,
    ) -> Option<impl Iterator<Item = (NodeId, &Node, &C)>> {
        let components = &self.edge_components.downcast::<C>()?.components;

        Some(components.iter().filter_map(move |(edge_id, comp)| {
            let edge = &self.edges[*edge_id];
            if edge.from == from {
                Some((edge.to, &self.nodes[edge.to.0], comp))
            } else {
                None
            }
        }))
    }

    /// Returns an iterator iterating over all the nodes connected *with*
    /// the node with id `from` via an edge with the [`Component`] `C`.
    /// Giving a tupple of the Edge component, NodeId, and node itself.
    ///
    /// Returns [`None`] if the [`Component`] `C` does not exist in this [`Graph`].
    pub fn neighbors_undirected<C: Component>(
        &self,
        from: NodeId,
    ) -> Option<impl Iterator<Item = (NodeId, &Node, &C)>> {
        let components = &self.edge_components.downcast::<C>()?.components;

        Some(components.iter().filter_map(move |(edge_id, comp)| {
            let edge = &self.edges[*edge_id];
            if edge.from == from {
                Some((edge.to, &self.nodes[edge.to.0], comp))
            } else if edge.to == from {
                Some((edge.from, &self.nodes[edge.from.0], comp))
            } else {
                None
            }
        }))
    }

    /// Create a new graph containing only the components marked as output components in
    /// the given [`ComponentsInfo`].
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use simulator_communication::{simulator::{Simulator, ComponentsInfo}, graph::Graph};
    /// # fn do_timestep(_: &mut Graph) {}
    /// #
    /// struct ExampleSimulator;
    ///
    /// impl Simulator for ExampleSimulator {
    ///     fn get_component_info() -> ComponentsInfo {
    ///         // < SNIP >
    /// #       todo!()
    ///     }
    ///         
    ///     fn new(
    ///         _delta_time: std::time::Duration,
    ///         _graph: simulator_communication::graph::Graph,
    ///     ) -> Self {
    ///         // < SNIP >
    /// #       todo!()
    ///     }
    ///         
    ///     fn do_timestep(&mut self, mut graph: Graph) -> Graph {
    ///         do_timestep(&mut graph);
    ///         graph.filter(Self::get_component_info())
    ///     }
    /// }
    /// ```
    pub fn filter(mut self, component_info: ComponentsInfo) -> Graph {
        let mut new_graph = Self {
            node_components: ComponentStorageMap {
                components: HashMap::new(),
            },
            edge_components: ComponentStorageMap {
                components: HashMap::new(),
            },
            global_components: ComponentStorageMap {
                components: HashMap::new(),
            },
            // Copy nodes and edges
            ..self
        };
        for (id, _) in component_info.output_components.iter() {
            if let Some(component) = self.node_components.components.remove(id) {
                new_graph.node_components.components.insert(*id, component);
            } else if let Some(component) = self.edge_components.components.remove(id) {
                new_graph.edge_components.components.insert(*id, component);
            } else if let Some(component) = self.global_components.components.remove(id) {
                new_graph
                    .global_components
                    .components
                    .insert(*id, component);
            }
        }
        new_graph
    }
}

#[allow(clippy::type_complexity)]
fn create_items_and_components<F, T, I, ID>(
    from: Vec<F>,
    get_manger_id: impl Fn(&F) -> u64,
    wrap_index: impl Fn(usize) -> ID,
    to_item: impl Fn(&F) -> Option<T>,
    get_components: impl Fn(F) -> Option<I>,
    components_info: &ComponentsInfo,
) -> Option<(
    Vec<T>,
    ComponentStorageMap,
    HashMap<u64, ID>,
    HashMap<ID, u64>,
)>
where
    ID: Clone + PartialEq + Eq + Hash,
    I: Iterator<Item = (String, Value)>,
{
    let mut items = Vec::new();

    // Holds all the components of all the items, grouped by component type
    // to be easly converted into ComponentStorages.
    let mut item_components: HashMap<_, _> = components_info
        .components
        .iter()
        .map(|(type_id, info)| (*type_id, (Vec::<(usize, Value)>::new(), info)))
        .collect();

    let mut index_to_manager_id = HashMap::new();
    let mut manager_id_to_index = HashMap::new();

    for (i, item) in from.into_iter().enumerate() {
        let manager_id = get_manger_id(&item);
        let wraped_index = wrap_index(i);

        index_to_manager_id.insert(wraped_index.clone(), manager_id);
        manager_id_to_index.insert(manager_id, wraped_index);

        items.push(to_item(&item)?);

        for (name, component) in get_components(item)? {
            let Some(type_id) = components_info.string_to_typeid.get(&name) else {
                // Skip component we didn't ask for
                continue;
            };
            let Some(info) = components_info.components.get(type_id) else {
                continue;
            };

            item_components
                .entry(info.type_id)
                .or_insert_with(|| (Vec::new(), info))
                .0
                .push((i, component));
        }
    }

    let components = item_components
        .into_iter()
        .map(|(type_id, (vec, info))| {
            let component_storage = (info.values_to_components)(vec)?;
            Some((type_id, component_storage))
        })
        .collect::<Option<_>>()?;
    let item_components = ComponentStorageMap { components };

    Some((
        items,
        item_components,
        manager_id_to_index,
        index_to_manager_id,
    ))
}

impl Graph {
    /// Create a graph from a [`proto::State`].
    pub(crate) fn from_state(
        state: proto::State,
        components_info: &ComponentsInfo,
    ) -> Option<Graph> {
        let proto::Graph { nodes, edge } = state.graph?;
        let global_components = state.global_components;

        let (nodes, node_components, node_manager_id_to_index, node_index_to_manager_id) =
            create_items_and_components(
                nodes,
                |node| node.id,
                NodeId,
                |node| {
                    Some(Node {
                        latitude: node.latitude,
                        longitude: node.longitude,
                    })
                },
                |n| Some(n.components.into_iter()),
                components_info,
            )?;

        let (edges, edge_components, edge_manager_id_to_index, edge_index_to_manager_id) =
            create_items_and_components(
                edge,
                |edge| edge.id,
                EdgeId,
                |edge| {
                    Some(Edge {
                        from: *node_manager_id_to_index.get(&edge.from)?,
                        to: *node_manager_id_to_index.get(&edge.to)?,
                    })
                },
                |e| Some(iter::once((e.component_type, e.component_data?))),
                components_info,
            )?;

        // TODO: clone not needed if create_items_and_components would not be used like this
        let (_, global_components, _, _) = create_items_and_components(
            vec![()],
            |()| 0,
            |i| i,
            |()| Some(()),
            move |()| Some(global_components.clone().into_iter()),
            components_info,
        )?;

        Some(Graph {
            nodes,
            edges,
            node_components,
            edge_components,
            global_components,

            node_index_to_manager_id,
            _node_manager_id_to_index: node_manager_id_to_index,
            edge_index_to_manager_id,
            _edge_manager_id_to_index: edge_manager_id_to_index,
        })
    }

    /// Create [`proto::State`] from a [`Graph`].
    pub(crate) fn into_state(self, components_info: &ComponentsInfo) -> Option<proto::State> {
        let mut global_components = HashMap::new();
        for (type_id, component_storage) in self.global_components.components {
            let Some(info) = components_info.output_components.get(&type_id) else {
                // Skip componts that aren't output components
                continue;
            };

            for (_, component) in (info.components_to_values)(component_storage) {
                global_components.insert(info.name.clone(), component);
            }
        }

        let mut nodes: Vec<_> = self
            .nodes
            .into_iter()
            .enumerate()
            .map(|(index, n)| proto::Node {
                longitude: n.longitude,
                latitude: n.latitude,
                id: self.node_index_to_manager_id[&NodeId(index)],
                components: HashMap::new(),
            })
            .collect();

        for (type_id, component_storage) in self.node_components.components {
            let Some(info) = components_info.output_components.get(&type_id) else {
                // Skip componts that aren't output components
                continue;
            };

            for (i, component) in (info.components_to_values)(component_storage) {
                nodes
                    .get_mut(i)?
                    .components
                    .insert(info.name.clone(), component);
            }
        }

        let mut edges = Vec::new();

        for (type_id, component_storage) in self.edge_components.components {
            let Some(info) = components_info.output_components.get(&type_id) else {
                // Skip componts that aren't output components
                continue;
            };

            for (i, component) in (info.components_to_values)(component_storage) {
                let edge = &self.edges[i];
                edges.push(proto::Edge {
                    from: self.node_index_to_manager_id[&edge.from],
                    to: self.node_index_to_manager_id[&edge.to],
                    id: self.edge_index_to_manager_id[&EdgeId(i)],
                    component_type: info.name.clone(),
                    component_data: Some(component),
                });
            }
        }

        for edge in &edges {
            debug_assert!(edge.component_data.is_some());
        }

        Some(proto::State {
            graph: Some(proto::Graph { nodes, edge: edges }),
            global_components,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::{component::ComponentPiece, Component, ComponentPiece};

    use super::*;

    #[derive(ComponentPiece, Component, Clone, Copy, Debug, PartialEq, Eq)]
    #[component(name = "test-node", ty = "node")]
    struct TestNodeComp(u32);

    #[derive(ComponentPiece, Component, Clone, Copy, Debug, PartialEq, Eq)]
    #[component(name = "test-edge", ty = "edge")]
    struct TestEdgeComp(u32);

    #[derive(ComponentPiece, Component, Clone, Copy, Debug, PartialEq, Eq)]
    #[component(name = "test-global", ty = "global")]
    struct TestGlobalComp(u32);

    #[derive(ComponentPiece, Component, Clone, Copy, Debug, PartialEq, Eq)]
    #[component(name = "test-node2", ty = "node")]
    struct TestNodeComp2(u32);

    #[derive(ComponentPiece, Component, Clone, Copy, Debug, PartialEq, Eq)]
    #[component(name = "test-edge2", ty = "edge")]
    struct TestEdgeComp2(u32);

    #[derive(ComponentPiece, Component, Clone, Copy, Debug, PartialEq, Eq)]
    #[component(name = "test-global2", ty = "global")]
    struct TestGlobalComp2(u32);

    #[test]
    fn create_items_and_components_test() {
        let components_info = ComponentsInfo::new().add_required_component::<TestNodeComp>();

        let c1 = TestNodeComp(1);
        let c2 = TestNodeComp(2);
        let c3 = TestNodeComp(3);
        let c4 = TestNodeComp(4);

        let nodes: Vec<_> = [&c1, &c2, &c3, &c4]
            .into_iter()
            .enumerate()
            .map(|(i, c)| ::proto::simulation::Node {
                longitude: (i + 1) as f64,
                latitude: (i + 1) as f64,
                id: (i * 2) as u64,
                components: HashMap::from([(TestNodeComp::get_name(), c.to_value())]),
            })
            .collect();

        let (nodes, node_components, edge_manager_id_to_index, edge_index_to_manager_id) =
            create_items_and_components(
                nodes,
                |node| node.id,
                NodeId,
                |node| {
                    Some(Node {
                        latitude: node.latitude,
                        longitude: node.longitude,
                    })
                },
                |n| Some(n.components.into_iter()),
                &components_info,
            )
            .unwrap();

        assert_eq!(
            nodes,
            vec![
                Node {
                    latitude: 1.0,
                    longitude: 1.0,
                },
                Node {
                    latitude: 2.0,
                    longitude: 2.0,
                },
                Node {
                    latitude: 3.0,
                    longitude: 3.0,
                },
                Node {
                    latitude: 4.0,
                    longitude: 4.0,
                },
            ]
        );

        let component_storage: &ComponentStorage<TestNodeComp> = node_components
            .components
            .get(&TypeId::of::<TestNodeComp>())
            .unwrap()
            .downcast_ref()
            .unwrap();

        assert_eq!(
            component_storage.components,
            vec![(0, c1), (1, c2), (2, c3), (3, c4)]
        );

        assert_eq!(
            edge_manager_id_to_index,
            HashMap::from([
                (0, NodeId(0)),
                (2, NodeId(1)),
                (4, NodeId(2)),
                (6, NodeId(3)),
            ])
        );
        assert_eq!(
            edge_index_to_manager_id,
            HashMap::from([
                (NodeId(0), 0),
                (NodeId(1), 2),
                (NodeId(2), 4),
                (NodeId(3), 6),
            ])
        );
    }

    fn create_test_graph() -> Graph {
        let components_info = ComponentsInfo::new()
            .add_required_component::<TestNodeComp>()
            .add_required_component::<TestNodeComp2>()
            .add_required_component::<TestEdgeComp>()
            .add_required_component::<TestEdgeComp2>()
            .add_required_component::<TestGlobalComp>()
            .add_required_component::<TestGlobalComp2>();

        let mut nodes: Vec<_> = [
            TestNodeComp(1),
            TestNodeComp(2),
            TestNodeComp(3),
            TestNodeComp(4),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, c)| ::proto::simulation::Node {
            longitude: (i + 1) as f64,
            latitude: (i + 1) as f64,
            id: (i * 2) as u64,
            components: HashMap::from([(TestNodeComp::get_name(), c.to_value())]),
        })
        .collect();

        nodes[0]
            .components
            .insert(TestNodeComp2::get_name(), TestNodeComp2(1).to_value());
        nodes[2]
            .components
            .insert(TestNodeComp2::get_name(), TestNodeComp2(3).to_value());

        let mut edges: Vec<_> = [
            (0, 2, TestEdgeComp(1)),
            (2, 4, TestEdgeComp(2)),
            (6, 4, TestEdgeComp(3)),
            (0, 4, TestEdgeComp(4)),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (from, to, c))| ::proto::simulation::Edge {
            from,
            to,
            id: i as u64,
            component_type: TestEdgeComp::get_name(),
            component_data: Some(c.to_value()),
        })
        .collect();

        edges.push(::proto::simulation::Edge {
            from: 0,
            to: 2,
            id: 100,
            component_type: TestEdgeComp2::get_name(),
            component_data: Some(TestEdgeComp2(5).to_value()),
        });
        edges.push(::proto::simulation::Edge {
            from: 0,
            to: 6,
            id: 101,
            component_type: TestEdgeComp2::get_name(),
            component_data: Some(TestEdgeComp2(6).to_value()),
        });

        let global_components =
            HashMap::from([(TestGlobalComp::get_name(), TestGlobalComp(1).to_value())]);

        Graph::from_state(
            ::proto::simulation::State {
                graph: Some(::proto::simulation::Graph { nodes, edge: edges }),
                global_components,
            },
            &components_info,
        )
        .unwrap()
    }

    #[test]
    fn get_all() {
        let graph = create_test_graph();

        assert_eq!(
            graph
                .get_all_nodes::<TestNodeComp>()
                .unwrap()
                .collect::<Vec<_>>(),
            vec![
                (
                    NodeId(0),
                    &Node {
                        latitude: 1.0,
                        longitude: 1.0,
                    },
                    &TestNodeComp(1)
                ),
                (
                    NodeId(1),
                    &Node {
                        latitude: 2.0,
                        longitude: 2.0,
                    },
                    &TestNodeComp(2)
                ),
                (
                    NodeId(2),
                    &Node {
                        latitude: 3.0,
                        longitude: 3.0,
                    },
                    &TestNodeComp(3)
                ),
                (
                    NodeId(3),
                    &Node {
                        latitude: 4.0,
                        longitude: 4.0,
                    },
                    &TestNodeComp(4)
                ),
            ]
        );

        assert_eq!(
            graph
                .get_all_nodes::<TestNodeComp2>()
                .unwrap()
                .collect::<Vec<_>>(),
            vec![
                (
                    NodeId(0),
                    &Node {
                        latitude: 1.0,
                        longitude: 1.0,
                    },
                    &TestNodeComp2(1)
                ),
                (
                    NodeId(2),
                    &Node {
                        latitude: 3.0,
                        longitude: 3.0,
                    },
                    &TestNodeComp2(3)
                ),
            ]
        );

        assert_eq!(
            graph
                .get_all_edges::<TestEdgeComp>()
                .unwrap()
                .collect::<Vec<_>>(),
            vec![
                (
                    EdgeId(0),
                    &Edge {
                        from: NodeId(0),
                        to: NodeId(1),
                    },
                    &TestEdgeComp(1)
                ),
                (
                    EdgeId(1),
                    &Edge {
                        from: NodeId(1),
                        to: NodeId(2),
                    },
                    &TestEdgeComp(2)
                ),
                (
                    EdgeId(2),
                    &Edge {
                        from: NodeId(3),
                        to: NodeId(2),
                    },
                    &TestEdgeComp(3)
                ),
                (
                    EdgeId(3),
                    &Edge {
                        from: NodeId(0),
                        to: NodeId(2),
                    },
                    &TestEdgeComp(4)
                ),
            ]
        );

        assert_eq!(
            graph
                .get_all_edges::<TestEdgeComp2>()
                .unwrap()
                .collect::<Vec<_>>(),
            vec![
                (
                    EdgeId(4),
                    &Edge {
                        from: NodeId(0),
                        to: NodeId(1),
                    },
                    &TestEdgeComp2(5)
                ),
                (
                    EdgeId(5),
                    &Edge {
                        from: NodeId(0),
                        to: NodeId(3),
                    },
                    &TestEdgeComp2(6)
                ),
            ]
        );
    }

    #[test]
    fn get_single() {
        let graph = create_test_graph();

        assert_eq!(
            graph.get_node_component::<TestNodeComp>(NodeId(0)),
            Some(&TestNodeComp(1))
        );
        assert_eq!(
            graph.get_node_component::<TestNodeComp>(NodeId(1)),
            Some(&TestNodeComp(2))
        );
        assert_eq!(
            graph.get_node_component::<TestNodeComp2>(NodeId(0)),
            Some(&TestNodeComp2(1))
        );
        assert_eq!(graph.get_node_component::<TestNodeComp2>(NodeId(1)), None);

        assert_eq!(
            graph.get_edge_component::<TestEdgeComp>(EdgeId(0)),
            Some(&TestEdgeComp(1))
        );
        assert_eq!(
            graph.get_edge_component::<TestEdgeComp>(EdgeId(1)),
            Some(&TestEdgeComp(2))
        );
        assert_eq!(graph.get_edge_component::<TestEdgeComp2>(EdgeId(1)), None);
        assert_eq!(
            graph.get_edge_component::<TestEdgeComp2>(EdgeId(4)),
            Some(&TestEdgeComp2(5))
        );

        assert_eq!(
            graph.get_global_component::<TestGlobalComp>(),
            Some(&TestGlobalComp(1))
        );
        assert_eq!(graph.get_global_component::<TestGlobalComp2>(), None);
    }

    #[test]
    fn get_via_edges() {
        let graph = create_test_graph();

        assert_eq!(
            graph
                .neighbors_directed::<TestEdgeComp>(NodeId(1))
                .unwrap()
                .collect::<Vec<_>>(),
            vec![(
                NodeId(2),
                &Node {
                    latitude: 3.0,
                    longitude: 3.0,
                },
                &TestEdgeComp(2)
            ),]
        );

        assert_eq!(
            graph
                .neighbors_undirected::<TestEdgeComp>(NodeId(1))
                .unwrap()
                .collect::<Vec<_>>(),
            vec![
                (
                    NodeId(0),
                    &Node {
                        latitude: 1.0,
                        longitude: 1.0,
                    },
                    &TestEdgeComp(1)
                ),
                (
                    NodeId(2),
                    &Node {
                        latitude: 3.0,
                        longitude: 3.0,
                    },
                    &TestEdgeComp(2)
                ),
            ]
        );
    }
}
