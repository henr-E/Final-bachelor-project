//! Types to build a simulator.
//!
//! To use this library you will have to implement the [`Simulator`] trait.

use core::panic;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    time::Duration,
};

use crate::{
    component::Component,
    graph::{ComponentStorage, Graph},
    Value,
};

type ValueToComponentsFn = fn(Vec<(usize, Value)>) -> Option<Box<dyn Any + Send>>;
type ComponentsToValueFn = fn(Box<dyn Any + Send>) -> Vec<(usize, Value)>;

/// Contains all the data needed to easily work with different components at runtime.
#[derive(Debug)]
pub(crate) struct ComponentInfo {
    pub(crate) type_id: TypeId,
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) proto_spec: crate::ComponentSpecification,
    pub(crate) values_to_components: ValueToComponentsFn,
    pub(crate) components_to_values: ComponentsToValueFn,
}

/// Generic function to go from a Vec of Values to a ComponentStorage.
/// Function pointers of this function are stored in the ComponentInfo's.
fn values_to_components<C: Component>(values: Vec<(usize, Value)>) -> Option<Box<dyn Any + Send>> {
    let components: Option<Vec<(usize, C)>> = values
        .into_iter()
        .map(|(i, v)| -> Option<(usize, C)> { Some((i, C::from_value(v)?)) })
        .collect();

    let mut components = components?;

    components.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

    Some(Box::new(ComponentStorage::<C> { components }))
}

/// Generic function to go from a ComponentStorage to a Vec of Values.
/// Function pointers to this function are stored in the ComponentInfo's.
fn components_to_values<C: Component>(
    component_storage: Box<dyn Any + Send>,
) -> Vec<(usize, Value)> {
    let component_storage = component_storage.downcast::<ComponentStorage<C>>().unwrap();

    component_storage
        .components
        .into_iter()
        .map(|(i, c)| (i, c.to_value()))
        .collect()
}

impl ComponentInfo {
    fn new<C: Component>(required: bool) -> Self {
        ComponentInfo {
            type_id: TypeId::of::<C>(),
            name: C::get_name(),
            required,
            proto_spec: C::get_spec(),
            values_to_components: values_to_components::<C>,
            components_to_values: components_to_values::<C>,
        }
    }
}

/// Contains information about the [`Component`]s a [`Simulator`] wants to use.
///
/// Use the builder style functions to add new [`Component`]s to the [`ComponentsInfo`].
///
/// # Example:
/// ```
/// # use simulator_communication::{ComponentPiece, Component, ComponentsInfo};
/// #
/// #[derive(ComponentPiece, Component)]
/// #[component(name = "node_example", ty = "node")]
/// pub struct ExampleNodeComponent {
///     pub some_int: i32,
///     pub some_string: String,
///     pub some_list: Vec<bool>,
/// }
///
/// #[derive(ComponentPiece, Component)]
/// #[component(name = "edge_example", ty = "edge")]
/// pub struct ExampleEdgeComponent(pub f64);
///
/// #[derive(ComponentPiece, Component)]
/// #[component(name = "global_example", ty = "global")]
/// pub struct ExampleGlobalComponent {
///     pub interesting_value: u32,
/// }
///
/// # fn main() {
/// ComponentsInfo::new()
///     // We want to both use `ExampleNodeComponent` as input and output.
///     .add_required_component::<ExampleNodeComponent>()
///     .add_output_component::<ExampleNodeComponent>()
///     // Only ask for `ExampleEdgeComponent` as input.
///     .add_required_component::<ExampleEdgeComponent>()
///     // Ask for `ExampleGlobalComponent` as input only if it exists.
///     .add_optional_component::<ExampleGlobalComponent>()
/// # ;
/// # }
/// ```
#[derive(Default, Debug)]
pub struct ComponentsInfo {
    pub(crate) string_to_typeid: HashMap<String, TypeId>,
    pub(crate) components: HashMap<TypeId, ComponentInfo>,
    pub(crate) output_components: HashMap<TypeId, ComponentInfo>,
}

impl ComponentsInfo {
    /// Create a new empty [`ComponentsInfo`].
    pub fn new() -> Self {
        ComponentsInfo::default()
    }

    /// Adds a component to the string_to_typeid map. Will panic if there is already
    /// an component with this name, but a difrent typeid.
    fn add_to_string_map(&mut self, info: &ComponentInfo) {
        if let Some(old_id) = self
            .string_to_typeid
            .insert(info.name.clone(), info.type_id)
        {
            if old_id == info.type_id {
                return;
            }
            panic!(
                "Added two difrent components with the same name: `{}`",
                info.name
            );
        }
    }

    /// Add the [`Component`] `C` as a component we promise to send back to the
    /// manager.
    pub fn add_output_component<C: Component>(mut self) -> Self {
        let info = ComponentInfo::new::<C>(true);
        self.add_to_string_map(&info);
        self.output_components.insert(info.type_id, info);
        self
    }

    /// Add the [`Component`] `C` as a component we need from the manager.
    pub fn add_required_component<C: Component>(mut self) -> Self {
        let info = ComponentInfo::new::<C>(true);
        self.add_to_string_map(&info);
        self.components.insert(info.type_id, info);
        self
    }

    /// Add the [`Component`] `C` as a component we want the manager to send if it has it.
    pub fn add_optional_component<C: Component>(mut self) -> Self {
        let info = ComponentInfo::new::<C>(false);
        self.add_to_string_map(&info);
        self.components.insert(info.type_id, info);
        self
    }
}

/// A simulator capable of anwsering requests from the manger.
///
/// Implement this trait on a struct representing a running simulation.
/// As the [`new`](Simulator::new) function will be used to create a
/// new instance of this simulator for every new simulation the manager asks for.
pub trait Simulator: Send + 'static {
    /// Get info about the components this [`Simulator`] will be using.
    ///
    /// See [`ComponentsInfo`] for more information.
    fn get_component_info() -> ComponentsInfo;

    /// Start a new simulation by creating a new instance of this [`Simulator`].
    ///
    /// This function will be called every time a new simulation is started by the manager.
    /// The graph represents the starting state of the simulation and will be sent again in
    /// the first timestep.
    fn new(delta_time: Duration, graph: Graph) -> Self;

    /// Handle a single timestep.
    ///
    /// The `graph` will contain the components that where asked for in the [`ComponentsInfo`]
    /// from [`get_component_info`](Simulator::get_component_info). These are the results from
    /// the other simulators that are running in the manager.
    ///
    /// You sould return a [`Graph`] containing all the components marked as output components in
    /// the [`ComponentsInfo`]. With the result of the simulation this timestep.
    fn do_timestep(&mut self, graph: Graph) -> Graph;
}
