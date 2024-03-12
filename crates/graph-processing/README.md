# Graph Processing
Allows for efficient processing of graph problems.
This library is made in a way such that it is easy to compute it in a distributed fashion in the future.
Much of the complexities in this library are because of this requirement and to be able to enforce compile time guarantees.

In this crate, graphs are made up of vertices with possibly differing types.
The processing happens in iterations called 'supersteps'.
Each vertex has a function that is called on each superstep, and it is called on all vertices before beginning the next superstep.
No order between different vertices is guaranteed however, and multiple vertices may be handled in parallel.

Vertices can send messages to eachother, which are handled at the beginning of each superstep (before calling `do_superstep`).
To send a message to a vertex of a certain type it has to implement a handler for that message.
It is guaranteed at compile time that messages are only sent to a vertex type that can handle it.

## Feature flags
- **parallel** (enabled by default)\
  Enables processing of the graph in parallel, while keeping the API the same. This enables the rayon dependency.

## Example
This example shows the basics of the library. It is best viewed in the generated rust docs.
```rust
use graph_processing::{
     message::{Message, MessageHandler},
     vertex::{Vertex, VertexContext, VertexId, VertexRef},
};

/// Message that can be sent between vertices.
/// 
/// It can contain any number of fields. Different message types can be used.
#[derive(Clone, Message)]
struct SomeMessage {
    some_data: i32,
}

/// A message does not necessarily need to contain data.
#[derive(Clone, Message)]
struct AnotherMessage;

/// Vertex in a graph.
///
/// Multiple vertex types can be used in one graph.
struct SomeVertex {
    other_data: i32,
}

/// Implements the function that will be run every 'frame'.
impl Vertex for SomeVertex {
    fn do_superstep(&mut self, ctx: VertexContext) {
        // Send a message to all the neighbouring nodes of type `SomeVertex`. The program will 
        // fail to compile if it does not implement MessageHandler for that message.
        ctx.send_message(
            ctx.get_neighbours::<SomeVertex>(),
            SomeMessage { some_data: 1 },
        );
    }
}

/// We can implement the MessageHandler trait for a vertex type to allow it to receive a type of
/// message.
impl MessageHandler<SomeMessage> for SomeVertex {
    fn handle(&mut self, ctx: VertexContext, message: SomeMessage) {
        // Do something to handle the message ...
        // You can also send notification here. They will be handled in the next superstep.
        // If you would like to use this message in the `do_superstep` function itself, you 
        // could store it in the vertex.
        
        // A VertexId is used to store a reference to a certain vertex. It may point to a vertex
        // that was already deleted from the graph, so it is not guaranteed to refer to an 
        // existing vertex.
        let vertex: VertexId<SomeVertex> = somehow_get_id();
        # // Not shown in the rust docs, this is mainly so that the example can compile (doctest).
        # fn somehow_get_id() -> VertexId<SomeVertex> {
        #     unimplemented!();
        # }
        
        // A VertexId can be turned into a VertexRef. The difference here is that a vertex ref
        // is guaranteed to point to a valid vertex. This is enforced at compile time, so as
        // long as your program compiles your usage of this reference is valid. You will see
        // that it is not possible to store this reference in the current vertex. Since the id
        // is not guaranteed to be valid, an error is returned when this is not the case.
        let Ok(vertex_ref): Result<VertexRef<SomeVertex>, _> = ctx.resolve_id(vertex) else {
            panic!("VertexId was invalid"); // In real implementations it is probably not a good
                                            // idea to panic.
        };
        
        // Now that we have a reference we can send a message to that vertex.
        // A shorthand function to do both the id lookup and message sending exists: 
        // `send_message_to_id`.
        ctx.send_message(vertex_ref, AnotherMessage);
    }
}

/// A vertex can handle multiple types of notification.
impl MessageHandler<AnotherMessage> for SomeVertex {
    fn handle(&mut self, ctx: VertexContext, message: AnotherMessage) {
        // Ideally you would want to implement something for each handler!
        unimplemented!();
    }
}
```

For another example of how to use this library, please look at the test in `src/lib.rs`.
