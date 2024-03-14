//! Vertices are the 'actors' of the graph.
//! They contain the data and serve as actors that can perform calculations on every superstep.
//!
//! Any data type can be turned into a a vertex by implementing the [`Vertex`] trait:
//! ```rust
//! # use graph_processing::vertex::*;
//! # struct MyType;
//! impl Vertex for MyType {
//!    fn do_superstep(&mut self, ctx: VertexContext) {
//!        // Implement function that is executed on every iteration or 'superstep'.
//!    }
//! }
//! ```

use std::{any::TypeId, marker::PhantomData};

use downcast_rs::{impl_downcast, Downcast};
use thiserror::Error;

use crate::{
    message::{Message, MessageHandler, MessageReceivers},
    Graph, MessageEntry, VertexItem,
};

/// A type that represents a vertex in a graph.
pub trait Vertex: Downcast + Send {
    /// Called on each superstep after processing incoming messages from the previous superstep.
    fn do_superstep(&mut self, ctx: VertexContext);
}
impl_downcast!(Vertex);

/// Uniquely identifies a vertex of a certain type in the system.
///
/// Is stable across supersteps provided that the target vertex is not deleted. Can be used to get a reference to a
/// certain vertex.
#[derive(Debug)]
pub struct VertexId<V: Vertex> {
    pub(crate) id: u64,
    /// Needed in order for this struct to compile: we need to use each generic argument at least once.
    pub(crate) phantom: PhantomData<V>,
}

impl<V: Vertex> Clone for VertexId<V> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            phantom: PhantomData,
        }
    }
}

/// A reference to a certain vertex. It is guaranteed to always be valid.
///
/// It cannot persist across supersteps. To refer to a vertex over multiple supersteps, look at [`VertexId`].
#[derive(Debug)]
pub struct VertexRef<'a, V: Vertex> {
    id: u64,
    /// Required in order for the struct to compile without needing to use the lifetime in one of the fields.
    phantom: PhantomData<&'a V>,
}

impl<'a, V: Vertex> Clone for VertexRef<'a, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, V: Vertex> Copy for VertexRef<'a, V> {}

/// Context passed along to each superstep. It allows a vertex to interact with the rest of the graph.
#[derive(Clone)]
pub struct VertexContext<'a> {
    pub(crate) self_id: u64,
    /// The data of the vertex to which the context belongs.
    pub(crate) self_item: &'a VertexItem,
    /// The entire graph itself.
    pub(crate) graph: &'a Graph,
}

impl<'a> VertexContext<'a> {
    /// Send a message to a certain other vertex.
    ///
    /// This message will be placed in a queue and handled before running the next `do_superstep` for that vertex. Messages are
    /// guaranteed to arrive in the order they were sent.
    ///
    /// When sending a message while handling another message this message will not be handled during the current batch of messages.
    pub fn send_message<'b, 'c, R, M>(&'c self, receivers: R, message: M)
    where
        'a: 'b,
        'c: 'b,
        M: Message,
        R: MessageReceivers<'b, M>,
    {
        // Check whether the graph already knows how to handle this message and vertex type combination.
        // todo: is there a better way to do this?
        let mut handlers = self
            .graph
            .message_handler_funcs
            .lock()
            .expect("mutex to not be poisoned");
        let handler_key = (TypeId::of::<R::H>(), TypeId::of::<M>());
        if handlers.get(&handler_key).is_none() {
            handlers.insert(handler_key, |ctx, vertex, message| {
                vertex
                    .downcast_mut::<R::H>()
                    .expect("vertex to be able to handle the message")
                    .handle(
                        ctx,
                        *message
                            .downcast::<M>()
                            .expect("message to be of the expected type"),
                    )
            });
        }
        // Early drop to unlock as soon as possible.
        drop(handlers);

        // Determine which queue to send to. This is the opposite of the queue that would be read from in this timestep.
        let queue_index = 1 - (self.graph.timestep % 2) as usize;
        for receiver in receivers.into_vertex_refs(self) {
            let vertex = self
                .graph
                .vertices
                .get(&receiver.id)
                .expect("vertex reference to be valid");
            vertex.message_queues[queue_index]
                .lock()
                .expect("mutex to not be poisoned")
                .push(MessageEntry {
                    sender: self.self_id,
                    message: Box::new(message.clone()),
                });
        }
    }

    /// Tries to resolve a [VertexId] before sending a message to that vertex.
    pub fn send_message_to_id<V, M>(
        &self,
        receiver: VertexId<V>,
        message: M,
    ) -> Result<(), VertexResolveError>
    where
        V: MessageHandler<M>,
        M: Message,
    {
        self.send_message(self.resolve_id(receiver)?, message);
        Ok(())
    }

    /// Get all the neighbouring vertices of type V. This is, all vertices with an edge between this vertex with the edge facing
    /// towards the neighbours.
    pub fn get_neighbours<V: Vertex>(&self) -> impl Iterator<Item = VertexRef<V>> + 'a {
        self.self_item.outgoing_edges.iter().filter_map(|v| {
            // Make sure the neighbour is of the right type. If not, don't return this neighbour.
            if self
                .graph
                .vertices
                .get(v)
                .expect("neighbour to not yet be deleted")
                .type_id
                != TypeId::of::<V>()
            {
                return None;
            }

            Some(VertexRef {
                id: *v,
                phantom: PhantomData,
            })
        })
    }

    /// Returns a reference to a vertex given its unique identifier. This reference can only be used during the
    /// current superstep.
    pub fn resolve_id<'b, V: Vertex>(
        &'b self,
        id: VertexId<V>,
    ) -> Result<VertexRef<'a, V>, VertexResolveError> {
        // Make sure the vertex exists before returning a reference to it.
        let vertex = self
            .graph
            .vertices
            .get(&id.id)
            .ok_or(VertexResolveError::NotFound)?;
        // Make sure the vertex is also of the expected type. At the time of writing, it is not possible for an identifier to
        // ever refer to another vertex but it is not a bad idea to have this check for the time being.
        if vertex.type_id != TypeId::of::<V>() {
            return Err(VertexResolveError::UnexpectedType);
        }
        Ok(VertexRef {
            id: id.id,
            phantom: PhantomData,
        })
    }

    /// Returns the number of timesteps that have been processed since the graph was created.
    ///
    /// Could be useful for algorithms that alternate their behaviour on certain steps.
    pub fn elapsed_timesteps(&self) -> u64 {
        self.graph.elapsed_timesteps()
    }
}

/// An error that may occur when trying to resolve a reference to a vertex.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum VertexResolveError {
    /// Could not find the requested vertex.
    #[error("Unable to find a vertex with the requested id")]
    NotFound,
    /// Found the vertex, but did not match the expected type.
    #[error("The requested vertex did not match the expected type")]
    UnexpectedType,
}
