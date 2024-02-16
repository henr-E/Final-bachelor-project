//! Messages are how vertices in a graph can share information with eachother.
//!
//! Any structure can be turned into a message by deriving the [`Message`] trait:
//! ```rust
//! # use graph_processing::message::Message;
//! #[derive(Clone)]
//! struct MyMessage;
//! ```
//! A message also needs to implement [`Clone`].

use crate::vertex::{Vertex, VertexContext, VertexRef};

/// Allow the implementation for the [`Message`] trait to be derived with a macro.
pub use graph_processing_macros::Message;

/// Sent between vertices to communicate with eachother.
pub trait Message: Clone + Sized + Send + 'static {}

/// Implemented by a vertex to handle a certain type of messages.
pub trait MessageHandler<M: Message>: Vertex {
    /// Handles an incoming message from another vertex.
    fn handle(&mut self, ctx: VertexContext, message: M);
}

/// 'Sugar trait' allowing [`VertexRef`] or an iterator containing multiple receivers to be used as a receiver.
///
/// This is the trait that allows `send_message` to work with both a single [`VertexRef`] or a collection of them.
pub trait MessageReceivers<'a, M: Message> {
    /// The type to which the returned [`VertexRef`]s refer.
    type H: MessageHandler<M> + 'a;

    /// Turn self into an interator of [`VertexRef`]s.
    ///
    /// This function must not fail.
    fn into_vertex_refs(
        self,
        ctx: &'a VertexContext,
    ) -> impl Iterator<Item = VertexRef<'a, Self::H>>;
}

/// Trivial conversion from a [`VertexRef`] to an interator containing only itself.
impl<'a, V: Vertex, M: Message> MessageReceivers<'a, M> for VertexRef<'a, V>
where
    V: MessageHandler<M> + 'a,
{
    type H = V;

    fn into_vertex_refs(
        self,
        _ctx: &'a VertexContext,
    ) -> impl Iterator<Item = VertexRef<'a, Self::H>> {
        [self].into_iter()
    }
}

/// Allow all iterators of [`MessageReceivers`] to also be used as this trait.
impl<'a, M, R, I> MessageReceivers<'a, M> for I
where
    M: Message,
    R: MessageReceivers<'a, M>,
    I: IntoIterator<Item = R>,
{
    type H = R::H;

    fn into_vertex_refs(
        self,
        ctx: &'a VertexContext,
    ) -> impl Iterator<Item = VertexRef<'a, Self::H>> {
        self.into_iter().flat_map(move |v| v.into_vertex_refs(ctx))
    }
}
