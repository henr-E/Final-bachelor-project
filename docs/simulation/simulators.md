# Simulators

The communication with the simulation manager is documented here: [**Simulator Communication**](./simulator-communication.md).

## Responsibilities
Each simulator is responsible for processing part of a frame (timestep) in a simulation.
Concretely, a simulator takes some components as input and alters some of these, returning them as output to the simulation.

## Load flow analysis
The documentation for the load flow simulation can be found [here](./load-flow-analysis.md)
