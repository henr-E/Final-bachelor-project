//! The main datatypes used to transfer data from and to the manager.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    iter,
};

use crate::{component::Component, proto, simulator::ComponentsInfo, Value};

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
    pub from: usize,
    /// The index of the destination node.
    pub to: usize,
}

pub(crate) struct ComponentStorage<C: Component> {
    pub(crate) components: Vec<(usize, C)>,
}

type ComponentStorageMap = HashMap<TypeId, Box<dyn Any + Send>>;

/// The main datastructure used to send data to and from the manager.
///
/// Contains a list of nodes containing multiple [`Component`]s.
/// Nodes can be connected with edges, all containing exactly one [`Component`].
///
/// There can also be global [`Component`]s.
#[derive(Default)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    node_components: ComponentStorageMap,
    edge_components: ComponentStorageMap,
    global_components: ComponentStorageMap,
}

impl Graph {
    /// Get all nodes with the [`Component`] `C` in this graph, together with this component.
    pub fn get_all_nodes<C: Component>(&self) -> Option<impl Iterator<Item = (usize, &Node, &C)>> {
        let component_storage = self.node_components.get(&TypeId::of::<C>())?;
        let component_storage = component_storage
            .downcast_ref::<ComponentStorage<C>>()
            .expect("Failed to downcast");
        let iter = component_storage
            .components
            .iter()
            .filter_map(|(index, component)| {
                self.nodes.get(*index).map(|node| (*index, node, component))
            });

        Some(iter)
    }

    /// Get all edges with the [`Component`] `C` in this graph, together with this component.
    pub fn get_all_edges<C: Component>(&self) -> Option<impl Iterator<Item = (&Edge, &C)>> {
        let component_storage = self.edge_components.get(&TypeId::of::<C>())?;
        let component_storage = component_storage
            .downcast_ref::<ComponentStorage<C>>()
            .expect("Failed to downcast");
        let iter = component_storage
            .components
            .iter()
            .filter_map(|(index, component)| self.edges.get(*index).map(|edge| (edge, component)));

        Some(iter)
    }

    /// Get all nodes with the [`Component`] `C` in this graph, together with this component.
    pub fn get_all_nodes_mut<C: Component>(
        &mut self,
    ) -> Option<impl Iterator<Item = (usize, &Node, &mut C)>> {
        let component_storage = self.node_components.get_mut(&TypeId::of::<C>())?;
        let component_storage = component_storage
            .downcast_mut::<ComponentStorage<C>>()
            .expect("Failed to downcast");
        let iter =
            component_storage
                .components
                .iter_mut()
                .filter_map(|(index, ref mut component)| {
                    self.nodes
                        .get(*index)
                        .map(|node: &Node| (*index, node, component))
                });

        Some(iter)
    }

    /// Get all edges with the [`Component`] `C` in this graph, together with this component.
    pub fn get_all_edges_mut<C: Component>(
        &mut self,
    ) -> Option<impl Iterator<Item = (&Edge, &mut C)>> {
        let component_storage = self.edge_components.get_mut(&TypeId::of::<C>())?;
        let component_storage = component_storage
            .downcast_mut::<ComponentStorage<C>>()
            .expect("Failed to downcast");
        let iter =
            component_storage
                .components
                .iter_mut()
                .filter_map(|(index, ref mut component)| {
                    self.edges.get(*index).map(|edge: &Edge| (edge, component))
                });

        Some(iter)
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
    ///     fn do_timestep(&mut self, mut graph: Graph) -> Graph {
    ///         do_timestep(&mut graph);
    ///         graph.filter(Self::get_component_info())
    ///     }
    ///
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
    /// }
    /// ```
    pub fn filter(mut self, component_info: ComponentsInfo) -> Graph {
        let mut new_graph = Graph::default();
        for (id, _) in component_info.output_components.iter() {
            if let Some(component) = self.node_components.remove(id) {
                new_graph.node_components.insert(*id, component);
            } else if let Some(component) = self.edge_components.remove(id) {
                new_graph.edge_components.insert(*id, component);
            } else if let Some(component) = self.global_components.remove(id) {
                new_graph.global_components.insert(*id, component);
            }
        }
        new_graph
    }

    /// Get the [`Component`] `C` for a single node.
    pub fn get_node_component<C: Component>(&self, index: &usize) -> Option<&C> {
        let component_storage = self.node_components.get(&TypeId::of::<C>())?;
        let component_storage = component_storage
            .downcast_ref::<ComponentStorage<C>>()
            .expect("Failed to downcast");
        if let Ok(component) = component_storage
            .components
            .binary_search_by(|(correct_index, _)| correct_index.cmp(index))
        {
            let found_component = component;
            return Some(&component_storage.components[found_component].1);
        }

        None
    }

    /// Get the [`Component`] `C` for a single edge.
    pub fn get_edge_component<C: Component>(&self, index: &usize) -> Option<&C> {
        let component_storage = self.edge_components.get(&TypeId::of::<C>())?;
        let component_storage = component_storage
            .downcast_ref::<ComponentStorage<C>>()
            .expect("Failed to downcast");
        if let Ok(component) = component_storage
            .components
            .binary_search_by(|(correct_index, _)| correct_index.cmp(index))
        {
            let found_component = component;
            return Some(&component_storage.components[found_component].1);
        }

        None
    }

    /// Get the [`Component`] `C` for a single node.
    pub fn get_node_component_mut<C: Component>(&mut self, index: &usize) -> Option<&mut C> {
        let component_storage = self.node_components.get_mut(&TypeId::of::<C>())?;
        let component_storage = component_storage
            .downcast_mut::<ComponentStorage<C>>()
            .expect("Failed to downcast");
        if let Ok(component) = component_storage
            .components
            .binary_search_by(|(correct_index, _)| correct_index.cmp(index))
        {
            let found_component = component;
            return Some(&mut component_storage.components[found_component].1);
        }
        None
    }

    /// Get the [`Component`] `C` for a single edge.
    pub fn get_edge_component_mut<C: Component>(&mut self, index: &usize) -> Option<&mut C> {
        let component_storage = self.edge_components.get_mut(&TypeId::of::<C>())?;
        let component_storage = component_storage
            .downcast_mut::<ComponentStorage<C>>()
            .expect("Failed to downcast");
        if let Ok(component) = component_storage
            .components
            .binary_search_by(|(correct_index, _)| correct_index.cmp(index))
        {
            let found_component = component;
            return Some(&mut component_storage.components[found_component].1);
        }

        None
    }
}

fn create_items_and_components<F, T, I>(
    from: Vec<F>,
    to_item: impl Fn(&F) -> T,
    get_components: impl Fn(F) -> I,
    components_info: &ComponentsInfo,
) -> Option<(Vec<T>, ComponentStorageMap)>
where
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

    for (i, item) in from.into_iter().enumerate() {
        items.push(to_item(&item));

        for (name, component) in get_components(item) {
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

    let item_components = item_components
        .into_iter()
        .map(|(type_id, (vec, info))| {
            let component_storage = (info.values_to_components)(vec)?;
            Some((type_id, component_storage))
        })
        .collect::<Option<_>>()?;

    Some((items, item_components))
}

impl Graph {
    /// Create a graph from a [`proto::State`].
    pub(crate) fn from_state(
        state: proto::State,
        components_info: &ComponentsInfo,
    ) -> Option<Graph> {
        let proto::Graph { nodes, edge } = state.graph?;
        let global_components = state.global_components;

        let (nodes, node_components) = create_items_and_components(
            nodes,
            |node| Node {
                latitude: node.latitude,
                longitude: node.longitude,
            },
            |n| n.components.into_iter(),
            components_info,
        )?;

        let (edges, edge_components) = create_items_and_components(
            edge,
            |edge| Edge {
                from: edge.from as usize,
                to: edge.to as usize,
            },
            |e| iter::once((e.component_type, e.component_data.unwrap())),
            components_info,
        )?;

        // TODO: clone not needed if create_items_and_components would not be used like this
        let (_, global_components) = create_items_and_components(
            vec![()],
            |()| (),
            move |()| global_components.clone().into_iter(),
            components_info,
        )?;

        Some(Graph {
            nodes,
            edges,
            node_components,
            edge_components,
            global_components,
        })
    }

    /// Create [`proto::State`] from a [`Graph`].
    pub(crate) fn into_state(self, components_info: &ComponentsInfo) -> Option<proto::State> {
        let mut global_components = HashMap::new();
        for (type_id, component_storage) in self.global_components {
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
            .map(|n| proto::Node {
                longitude: n.longitude,
                latitude: n.latitude,
                components: HashMap::new(),
            })
            .collect();

        for (type_id, component_storage) in self.node_components {
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

        let mut edges: Vec<_> = self
            .edges
            .into_iter()
            .map(|e| proto::Edge {
                from: e.from as u64,
                to: e.to as u64,
                component_type: String::new(),
                component_data: None,
            })
            .collect();

        for (type_id, component_storage) in self.edge_components {
            let Some(info) = components_info.output_components.get(&type_id) else {
                // Skip componts that aren't output components
                continue;
            };

            for (i, component) in (info.components_to_values)(component_storage) {
                let edge = edges.get_mut(i)?;
                edge.component_type = info.name.clone();
                edge.component_data = Some(component);
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
mod tests {
    use super::*;
    use crate::{component::ComponentPiece, Component, ComponentPiece};

    #[test]
    fn create_items_and_components_test() {
        #[derive(ComponentPiece, Component, Clone, Copy, Debug, PartialEq, Eq)]
        #[component(name = "test-node", ty = "node")]
        struct TestNodeComp(u32);

        let components_info = ComponentsInfo::new().add_required_component::<TestNodeComp>();

        let c1 = TestNodeComp(1);
        let c2 = TestNodeComp(2);
        let c3 = TestNodeComp(3);
        let c4 = TestNodeComp(4);

        let nodes = vec![
            ::proto::simulator::Node {
                longitude: 1.0,
                latitude: 1.0,
                components: HashMap::from([("test-node".to_owned(), c1.to_value())]),
            },
            ::proto::simulator::Node {
                longitude: 2.0,
                latitude: 2.0,
                components: HashMap::from([("test-node".to_owned(), c2.to_value())]),
            },
            ::proto::simulator::Node {
                longitude: 3.0,
                latitude: 3.0,
                components: HashMap::from([("test-node".to_owned(), c3.to_value())]),
            },
            ::proto::simulator::Node {
                longitude: 4.0,
                latitude: 4.0,
                components: HashMap::from([("test-node".to_owned(), c4.to_value())]),
            },
        ];

        let (nodes, node_components) = create_items_and_components(
            nodes,
            |node| Node {
                latitude: node.latitude,
                longitude: node.longitude,
            },
            |n| n.components.into_iter(),
            &components_info,
        )
        .unwrap();

        assert_eq!(
            nodes,
            vec![
                Node {
                    latitude: 1.0,
                    longitude: 1.0
                },
                Node {
                    latitude: 2.0,
                    longitude: 2.0
                },
                Node {
                    latitude: 3.0,
                    longitude: 3.0
                },
                Node {
                    latitude: 4.0,
                    longitude: 4.0
                }
            ]
        );

        let component_storage: &ComponentStorage<TestNodeComp> = node_components
            .get(&TypeId::of::<TestNodeComp>())
            .unwrap()
            .downcast_ref()
            .unwrap();

        assert_eq!(
            component_storage.components,
            vec![(0, c1), (1, c2), (2, c3), (3, c4)]
        );
    }
}
