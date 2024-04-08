# Simulation Manager

The communication with the simulators is documented here: [**Simulator Communication**](./simulator-communication.md).

## Responsibilities

The entire _simulation runner_ group is only responsible for determining what happens in each time step or 'frame' of a simulation.
It is specifically the simulation manager that stores all these frames in a database and makes them available to other services via its gRPC API as defined in `proto/simulation/simulation-manager.proto`.
If necessary, these simulations and all their corresponding frames can later be deleted (also defined in the previous .proto file).
Metadata such as the name of a simulation or who has access to it are not matters that this service should be dealing with.

These frames are all calculated in the simulation runner group.
The manager takes an existing frame _n_, passes it on to each simulator, and combines the results from each of these into frame _n+1_.

Besides storing every previously calculated frame for every simulation, a queue with simulations that are yet to be processed is also stored.
Each simulation is processed on a _first in, first out_ basis.
Only when the previous simulation has been fully processed should the simulation manager move on to the next.
It is possible to extend the simulation manager to process multiple simulations in the future, but this is not strictly necessary as of now.

## Implementation

The manager essentially consists of two parts: a service that communicates with other services, accepts new simulations and is able to send frames and another part that actually runs the simulation.
The first is a gRPC server while the latter is made possible with a collection of gRPC clients.

## Simulating

As mentioned before, simulating a frame is done by getting the previous frame and sending it to each of the simulators.
Simulators should only receive the components they requested in their response to the `GetIOConfig` call.
This should be done in parallel: simulators should be sent their frames at the same time (more or less) as it would otherwise be pointless to split up processing into multiple simulators.

A new frame should be constructed with the responses from the respective simulators.
This can be done trivially as two simulators are not allowed to output the same component type.
The manager should report an error should this be the case.
Components that are not output by any simulator should simply be copied from the previous frame.
A straight forward way to implement this whole process is to copy the entire frame and then replace components output by the simulators in this new copied frame.
This also rests on the fact that no components or entities(nodes, edges) are added or removed when processing a simulation.

## Queueing

A FIFO queue is used to keep track of simulations that are yet to be processed.
It is stored in the database to ensure persistence.
