# Simulator Communication

## Goals

- Provide a way for the simulation manager to communicate with arbitrary simulators, even external ones.
- Providing a way to add new simulators (possibly made by other people) without needing to update the internals of the manager.

This last point also implies that it should be possible to easily add new types of data to the simulation.

## Challenges

There are a few different challenges involved in achieving the goals above:
- It should be possible to save every step of the simulation in a database.
- New simulation types could need new data types.
- There needs to be some way to link the same location between different simulations (like the same house having an electic and gas connection).

## Solution

Our solution consists of a way to represent the data in the city and extends the way the communication is done with the manager below. 

Represent the city using:
- A graph consisting of:
  - A list of nodes:
    - A location in the city
    - A map of multiple components (component name -> component data)\
      Every node can have only one of every component name. Examples of components are: "electricity-consumer", "gas-producer".
  - A list of relations between nodes (similar to edges, but there can be more than 1 relation between 2 nodes):
    - A `from` and a `to` node.
    - Every relation has exactly 1 component (e.g: a powerline for transmission of energy) these will probably not be same components as for the nodes.
    - In this structure all relations are represented as directed edges. For some components (such as "energy-transmission" for example) it may not make sense for these to be directed. These can be treated as undirected by the simulators.
- Global data structures. For example: the weather data for the city.

=== Component Structure

Each type of component should have a structure that is predetermined for that type.
The following rust type represents all the different data structures that can make up a component.
An actual concrete component would have a data structure that is represented by exactly one such value of this enum.

```rust
/// Values of this enum represent a data structure that a component can have.
enum ComponentStructure {
    Integer(u64),
    Float(f64),
    Bool(bool),
    String(String),
    /// Some fields can either not have a value or a value of a certain type.
    Optional(Option<ComponentStructure>),
    /// A list of data, where each element is of the same type.
    List(Vec<ComponentStructure>),
    /// A struct or map where each field has its a predefined type.
    Struct(HashMap<String, ComponentStructure>),
}
```

For the sake of brevity this example type combines all the different integer and float types into one.

## Communication between Manager and Simulator

### Adding new simulator (or startup of manager)
When a new simulator is added, the manager asks the simulator which components it needs to perform a timestep and which components it produces after performing that timestep.
In the response to the manager it also gives the data types the simulator expects for these components.

### Starting a new simulation
The manager builds the city nodes and edges from data out from the database and sends it to all the simulators the manager wants to run for this simulation.
It sends only the components that the specific simulator asked for to prevent excess data being sent.
Any metadata for a simulation is also sent here. (like the size of a single time step)

The simulator start a new simulation from this data internally.

### Simulating
Every step of the simulation the manager sends the data from the previous step of the simulation to all the simulators.
The simulators perform their single time step and send the data they produce back to the manager.
Who can then use this new data in the next step.

## Example
Here is an example using an energy transmission simulator and an energy production/consumption simulator.
Hopefully it becomes clear that it does not make sense to combine the transmission and production/consumption into one simulator.

The manager then checks if the expected data structures it received for the components match what it already knows for those components.
If for example `energy-transmission-node` is a component not yet known to the manager, it defines it then.

We are using something like json in these examples, but this is only for the example.
The exact names are also not fixed yet.

### Adding new simulator (or startup of manager)

The energy production/consumption simulator would send:
```json
{
  required_inputs: ["energy-comsumer", "energy-producer"],
  optional_inputs: ["temperature"],
  outputs: ["energy-comsumer", "energy-producer"],
  component_structures: {
    temperature: "float",
    energy-consumer: {
      demanded_energy: "float",
      current_energy: "float"
    },
    energy-producer: {
      max_production: "float",
      produced_energy: "float"
    }
  }
}
```

The transmission simulator would send:
```json
{
  required_inputs: ["energy-transmission-node", "energy-transmission-line", "energy-comsumer", "energy-producer"],
  optional_inputs: [],
  outputs: ["energy-transmission-node", "energy-transmission-line"],
  // Each of the component structures follows the values defined above for `ComponentStructure`.
  // The actual concrete structures are not final and just an example.
  component_structures: {
    energy-transmission-node: {
      max_capacity: "float",
      combined_demand: "float"
    },
    energy-transmission-line: {
      max_capacity: "float",
      current_capacity: "float",
      length: "int"
    }
    // The examples from above ...
  }
}
```

### Starting a new simulation
The manager start a new simulation by sending the following informaion to the simulators:
- To the energy production/consumption simulator:
 ```json
{
  nodes: [
    {
      x: 0.123, y: 0.433,
      components: {
        energy-consumer: {
          demanded_energy: 3.2,
          current_energy: 3.2
        }
      }
    },
    {
      x: 0.123, y: 0.433,
      components: {
        energy-consumer: {
          demanded_energy: 2.4,
          current_energy: 2.3
        }
      }
    },
    {
      x: 0.123, y: 0.433,
      components: {
        energy-producer: {
          max_production: 30.2,
          produced_energy: 15.2
        }
      }
    }
    // ...
  ],
  edges: [
    // Since the energy production/consumption simulator does not request edge components, no edges need to be sent.
  ],
  global_components: {
    temperature: 25.0,
  }
}
```
Something similar is sent to the transmission simulator, but with more components.

### Simulating
Every tick is the exact same format as described above is sent, but with different data.
The simulator also responds with the same format but only containing components denoted in the `outputs` field.
Sending the result of the simulation.
