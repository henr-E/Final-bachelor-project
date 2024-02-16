#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]

use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, BTreeSet, BinaryHeap},
    convert,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard},
};

use ahash::AHashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use vertex::{Vertex, VertexContext, VertexId, VertexResolveError};

pub mod message;
pub mod vertex;

/// A datastructure made up of vertices connected to eachother with edges.
#[derive(Default)]
pub struct Graph {
    /// Stores the next free unique identifier.
    next_id: u64,
    /// Keep track of how many timesteps have elapsed.
    ///
    /// Used to determine whether the current timestep is odd or not. This changes which message queue is used in vertices.
    timestep: u64,
    vertices: BTreeMap<u64, VertexItem>,
    /// Allows vertices to handle type erased messages.
    message_handler_funcs: Mutex<AHashMap<(TypeId, TypeId), MessageHandlerFunc>>,
}

/// Function used to handle type erased messages in vertices.
type MessageHandlerFunc = fn(VertexContext, &mut dyn Vertex, Box<dyn Any>);

/// Data regarding a vertex in a graph.
pub(crate) struct VertexItem {
    /// The [TypeId] corresponding to the vertex. Stored separately for quick access.
    type_id: TypeId,
    /// The implementation of the [`Vertex`] trait.
    vertex: Box<Mutex<dyn Vertex>>,
    /// The edges going away from the current vertex to another vertex. This set contains the destination
    /// vertices.
    ///
    /// Uses a binary tree instead of a hashmap to preserve determinism.
    outgoing_edges: BTreeSet<u64>,
    /// The reverse of the outgoing_edges field: keeps track of which edges point towards our current vertex.
    ///
    /// This information is redundant but significantly speeds up the removal of nodes from the graph as it eliminates
    /// the need of iterating over all nodes.
    incoming_edges: BTreeSet<u64>,
    /// Two queues of incoming messages.
    ///
    /// Two queues are kept in order to ensure messages sent in one timestep are not handled in the same timestep.
    /// A priority queue is used to ensure that messages always arrive in the same order.
    message_queues: [Mutex<BinaryHeap<MessageEntry>>; 2],
}

impl Graph {
    /// Create a new, empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new node to the graph, returning its unique identifier.
    pub fn insert_vertex<V: Vertex>(&mut self, vertex: V) -> VertexId<V> {
        let id = self.next_id;
        self.next_id += 1;

        if self
            .vertices
            .insert(
                id,
                VertexItem {
                    type_id: vertex.type_id(),
                    vertex: Box::new(Mutex::new(vertex)),
                    outgoing_edges: Default::default(),
                    incoming_edges: Default::default(),
                    message_queues: Default::default(),
                },
            )
            .is_some()
        {
            // It will take a very long time before all u64s are exhausted.
            panic!("All vertex ids have been exhausted");
        }

        VertexId {
            phantom: std::marker::PhantomData,
            id,
        }
    }

    /// Obtains a lock on a specific vertex. Will block if a lock on the requested vertex already exists.
    ///
    /// Multiple locks on different vertices can be obtained concurrently.
    pub fn get_and_lock_vertex<V: Vertex>(
        &self,
        id: VertexId<V>,
    ) -> Result<VertexGuard<V>, VertexResolveError> {
        self.vertices
            .get(&id.id)
            .map(|v| {
                if v.type_id != TypeId::of::<V>() {
                    return Err(VertexResolveError::UnexpectedType);
                }
                Ok(VertexGuard {
                    guard: v.vertex.lock().expect("mutex to not be poisoned"),
                    phantom: PhantomData,
                })
            })
            .ok_or(VertexResolveError::NotFound)
            .and_then(convert::identity) // Flatten the result. Result::flatten() is unstable for some reason.
    }

    /// Removes a vertex from the graph, returning its stored value.
    pub fn remove_vertex<V: Vertex>(&mut self, id: VertexId<V>) -> Result<(), VertexResolveError> {
        let id = id.id;
        let vertex = self
            .vertices
            .remove(&id)
            .ok_or(VertexResolveError::NotFound)?;
        if vertex.type_id != TypeId::of::<V>() {
            // This is the least common path (and hopefully inpossible), so this re-insertion should not be a problem.
            self.vertices.insert(id, vertex);
            return Err(VertexResolveError::UnexpectedType);
        }

        // Clean up all edges referencing the vertex.
        for outgoing in vertex.outgoing_edges.into_iter().filter(|v| *v != id) {
            self.vertices
                .get_mut(&outgoing)
                .expect("edge to point to a known vertex")
                .incoming_edges
                .remove(&id);
        }
        for incoming in vertex.incoming_edges.into_iter().filter(|v| *v != id) {
            self.vertices
                .get_mut(&incoming)
                .expect("edge to originate from a known vertex")
                .outgoing_edges
                .remove(&id);
        }
        Ok(())
    }

    /// Inserts an edge from the first vertex towards the second vertex.
    ///
    /// Returns an error if either edges could not be found or are of the wrong type.
    pub fn insert_edge_directed<A, B>(
        &mut self,
        from_id: VertexId<A>,
        to_id: VertexId<B>,
    ) -> Result<(), VertexResolveError>
    where
        A: Vertex,
        B: Vertex,
    {
        // Check that both vertices exist.
        let _ = self
            .vertices
            .get(&to_id.id)
            .ok_or(VertexResolveError::NotFound)?;

        let from = self
            .vertices
            .get_mut(&from_id.id)
            .ok_or(VertexResolveError::NotFound)?;
        from.outgoing_edges.insert(to_id.id);

        let to = self
            .vertices
            .get_mut(&to_id.id)
            .ok_or(VertexResolveError::NotFound)?;
        to.incoming_edges.insert(from_id.id);
        Ok(())
    }

    /// Inserts edges to and from both vertices.
    ///
    /// Returns an error if either edges could not be found.
    pub fn insert_edge_bidirectional<A, B>(
        &mut self,
        from_id: VertexId<A>,
        to_id: VertexId<B>,
    ) -> Result<(), VertexResolveError>
    where
        A: Vertex,
        B: Vertex,
    {
        // Check that both vertices exist.
        let _ = self
            .vertices
            .get(&to_id.id)
            .ok_or(VertexResolveError::NotFound)?;

        let from = self
            .vertices
            .get_mut(&from_id.id)
            .ok_or(VertexResolveError::NotFound)?;
        from.outgoing_edges.insert(to_id.id);
        from.incoming_edges.insert(to_id.id);

        let to = self
            .vertices
            .get_mut(&to_id.id)
            .ok_or(VertexResolveError::NotFound)?;
        to.incoming_edges.insert(from_id.id);
        to.outgoing_edges.insert(from_id.id);
        Ok(())
    }

    /// Removes an edge going from vertex A to vertex B, if it exists.
    pub fn remove_edge_directed<A, B>(
        &mut self,
        from_id: VertexId<A>,
        to_id: VertexId<B>,
    ) -> Result<(), VertexResolveError>
    where
        A: Vertex,
        B: Vertex,
    {
        let from = self
            .vertices
            .get_mut(&from_id.id)
            .ok_or(VertexResolveError::NotFound)?;
        from.outgoing_edges.remove(&to_id.id);

        let to = self
            .vertices
            .get_mut(&to_id.id)
            .ok_or(VertexResolveError::NotFound)?;
        to.incoming_edges.remove(&from_id.id);
        Ok(())
    }

    /// Removes an edge going from vertex A to vertex B, if it exists.
    pub fn remove_edge_bidirectional<A, B>(
        &mut self,
        from_id: VertexId<A>,
        to_id: VertexId<B>,
    ) -> Result<(), VertexResolveError>
    where
        A: Vertex,
        B: Vertex,
    {
        let from = self
            .vertices
            .get_mut(&from_id.id)
            .ok_or(VertexResolveError::NotFound)?;
        from.outgoing_edges.remove(&to_id.id);
        from.incoming_edges.remove(&to_id.id);

        let to = self
            .vertices
            .get_mut(&to_id.id)
            .ok_or(VertexResolveError::NotFound)?;
        to.incoming_edges.remove(&from_id.id);
        to.outgoing_edges.remove(&from_id.id);
        Ok(())
    }

    /// Executes one superstep in the graph.
    pub fn do_superstep(&mut self) {
        let graph: &Graph = self;

        #[cfg(feature = "parallel")]
        let graph_iter = graph.vertices.par_iter();
        #[cfg(not(feature = "parallel"))]
        let graph_iter = graph.vertices.iter();

        graph_iter.for_each(|(id, item)| {
            let ctx = VertexContext {
                self_id: *id,
                self_item: item,
                graph,
            };

            let mut vertex = item.vertex.lock().expect("mutex to not be poisoned");
            let mut messages = item.message_queues[(self.timestep % 2) as usize]
                .lock()
                .expect("mutex to not be poisoned");

            // Handle all incoming messages.
            for message in messages.drain() {
                let Some(handler_func) = self
                    .message_handler_funcs
                    .lock()
                    .expect("mutex to not be poisoned")
                    .get(&(item.type_id, message.message.as_ref().type_id()))
                    .cloned()
                else {
                    // This should already be enforced at compile time.
                    unreachable!("Vertex received message it cannot handle");
                };
                handler_func(ctx.clone(), &mut *vertex, message.message);
            }

            // Execute this vertex's superstep.
            vertex.do_superstep(ctx);
        });

        self.timestep += 1;
    }

    /// Returns the number of timesteps that have been completed so far.
    pub fn elapsed_timesteps(&self) -> u64 {
        self.timestep
    }
}

/// Allows a lock to a vertex of a certain type to be obtained and kept outside of a graph.
pub struct VertexGuard<'a, V: Vertex> {
    guard: MutexGuard<'a, dyn Vertex>,
    phantom: PhantomData<V>,
}

impl<'a, V: Vertex> Deref for VertexGuard<'a, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.guard
            .downcast_ref()
            .expect("the vertex's type to be checked before creating VertexGuard")
    }
}

impl<'a, V: Vertex> DerefMut for VertexGuard<'a, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard
            .downcast_mut()
            .expect("the vertex's type to be checked before creating VertexGuard")
    }
}

/// Entry for a message queue.
///
/// This exists to allow a message to be used in a priority queue using its sender as priority.
struct MessageEntry {
    sender: u64,
    message: Box<dyn Any + Send>,
}

impl PartialOrd for MessageEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.sender.cmp(&other.sender))
    }
}

impl Ord for MessageEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sender.cmp(&other.sender)
    }
}

impl PartialEq for MessageEntry {
    fn eq(&self, other: &Self) -> bool {
        self.sender.eq(&other.sender)
    }
}

impl Eq for MessageEntry {}

#[cfg(test)]
mod tests {
    use crate::{
        message::{Message, MessageHandler},
        vertex::{Vertex, VertexContext},
        Graph,
    };

    /// Message that can be sent between vertices.
    #[derive(Clone, Message, Debug)]
    struct SomeMessage {
        some_data: i32,
    }

    /// Vertex in a graph.
    #[derive(Debug, PartialEq, Eq)]
    struct SomeVertex {
        amount_of_messages: u32,
        amount_of_neighbours: u32,
        other_data: i32,
    }

    /// Implements the function that will be run every 'frame'.
    impl Vertex for SomeVertex {
        fn do_superstep(&mut self, ctx: VertexContext) {
            // We can update the Vertex in each superstep.
            self.amount_of_neighbours = ctx.get_neighbours::<SomeVertex>().count() as u32;

            // Send a message to all the neighbouring nodes of type `SomeVertex`. The program will fail to compile if it
            // does not implement MessageHandler for that message.
            ctx.send_message(
                ctx.get_neighbours::<SomeVertex>(),
                SomeMessage { some_data: 1 },
            );

            // We can get a reference to some neighbour vertex like this:
            let Some(neighbour) = ctx.get_neighbours::<SomeVertex>().next() else {
                // Return if there were no neighbours (the next neighbour was None).
                return;
            };
            // Sending a message works with any number of recipients.
            ctx.send_message(neighbour, SomeMessage { some_data: 2 });
        }
    }

    // We can implement the MessageHandler trait for a vertex type to allow it to receive a type of message.
    impl MessageHandler<SomeMessage> for SomeVertex {
        fn handle(&mut self, _ctx: VertexContext, message: SomeMessage) {
            // Do something to handle the message ...
            self.amount_of_messages += 1;
            self.other_data += message.some_data;
        }
    }

    /// Simple test that runs some arbitrary graph calculations and see if they produce the expected result.
    #[test]
    fn test_graph() {
        let mut graph = Graph::new();
        let ids = (0..5)
            .map(|v| {
                graph.insert_vertex(SomeVertex {
                    amount_of_messages: 0,
                    amount_of_neighbours: 0,
                    other_data: v,
                })
            })
            .collect::<Vec<_>>();

        for i in 1..5 {
            assert_eq!(
                Ok(()),
                graph.insert_edge_bidirectional(ids[0].clone(), ids[i].clone()),
                "could not insert edge 0->{i}"
            );
        }

        for _ in 0..3 {
            graph.do_superstep();
        }

        let test_data = [
            SomeVertex {
                amount_of_messages: 16,
                amount_of_neighbours: 4,
                other_data: 24,
            },
            SomeVertex {
                amount_of_messages: 4,
                amount_of_neighbours: 1,
                other_data: 7,
            },
            SomeVertex {
                amount_of_messages: 2,
                amount_of_neighbours: 1,
                other_data: 4,
            },
            SomeVertex {
                amount_of_messages: 2,
                amount_of_neighbours: 1,
                other_data: 5,
            },
            SomeVertex {
                amount_of_messages: 2,
                amount_of_neighbours: 1,
                other_data: 6,
            },
        ];
        for (i, (id, expected)) in ids.into_iter().zip(test_data.into_iter()).enumerate() {
            assert_eq!(
                &*graph.get_and_lock_vertex(id).unwrap(),
                &expected,
                "testing equality of vertex {i}"
            );
        }
    }
}
