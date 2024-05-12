# Creating simulators

Full simulations consist of a set of simulators, where every simulator is focused on a small part
of the simulation. The job of a simulator could be to predict the electricity usage of homes, or to
simulate the electricity transmission over the network. All simulators run completely independently
and communicate with the central manager over [gRPC](https://grpc.io/).

To get a better idea of why and how this communication happens, see the
[design document](../simulator-communication.md) and the [gRPC definitions](../../proto/simulation).

## Implementing in Rust
We created a library to easily create simulators in rust:
[simulator-communication](`../../crates/simulator-communication`).
This library handles the encoding and decoding of components into rust types for you. So you can
focus on creating the simulation itself. See its
[README](../../crates/simulator-communication/README.md) and the rust docs for more information on
how to use it.

Make sure to also read [the general tips](#general-tips-for-creating-simulators) when creating a
new simulator.

## Implementing in other languages
You will have to make raw gRPC calls in other languages, as no other libraries have been created
yet There will be two services you will have to use:
- [`SimulatorConnection`](../../proto/simulation/simulator-connection.proto) as a client to connect
  to the running manager.
- [`Simulator`](../../proto/simulation/simulator.proto) as a server to get and send data to the
  manager during a simulation.

### `SimulatorConnection` handshake
A simulator should start by calling `ConnectSimulator` on the manager. The manager will immediately
try to connect back to the simulator. The manager will use the address the `ConnectSimulator`
call came from with the given port to connect to.
```protobuf

message SimulatorInfo {
    // The port which the simulator is listening on
    uint32 port = 1;
    string name = 2;
}

service SimulatorConnection {
    rpc ConnectSimulator(SimulatorInfo) returns (google.protobuf.Empty);
}
```

### running as a `Simulator`
The `Simulator` service consists of three functions:
```protobuf
service Simulator {
  rpc GetIOConfig (IOConfigRequest) returns (SimulatorIOConfig);

  rpc Setup (InitialState) returns (SetupResponse);

  rpc DoTimestep (simulation.State) returns (TimestepResult);
}

```
- `GetIOConfig`: Will be used by the manager to get information about what components the simulator
  uses and how.
- `Setup`: When a new simulation is started this function will be called.
- `DoTimestep`: Will be called for every timestep in the simulation.

See the documentation in the [proto file](../../proto/simulation/simulator.proto) for more details
about the arguments and return values.

An important point to remember is that only a single simulation can run at the time. So `Setup` being
called will mark the old simulation as done, and start a new one.

## General tips for creating simulators
When creating a simulator keep the following in mind:

### Getting data from another simulator
Simulators communicate with each other by having one simulator write to a component, another
simulator can then read this the next timestep. An important point to remember here is that all
simulators run at the same time, so two simulators can not use the same component as an output
component. To get around this, a component can be split into multiple parts so that every simulator
only outputs to one part of the original component.
