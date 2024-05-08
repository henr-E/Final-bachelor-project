# Simulation integration tests

The simulation integration tests, test everything from the separate simulators, the manger simulator
side, to the manger frontend communication side. However, it does not test the frontend itself.)
They test runner is implemented in `./tools/manager-integration-tests/`, and runs tests located in
`./integration-tests/`.

Test cases are specified in `.toml` files, with auxiliary `.csv` files used to specify big blocks of
data. Every test case specifies the used simulators, what components should exist in the manager for
those simulators, the initial state, and of course the expected output. See more about how to create
a test case [here](#creating-test-cases). It is also possible to specify a mock simulator to replace
any non-deterministic simulator.


> ℹ️ **Note**:
>
> All command specified assume that you are running from the workspace root (where the `Cargo.lock`
> is located).

## Running tests
If you just want to run the test, first build the production containers using:
```sh
sudo ./tools/build-production.sh
```
You can now use the following command to run all the test cases specified in `./integration-tests/`.
(note that this is NOT ran with sudo)
```sh
./tools/run-integration-test.sh
```
And you should get output that looks like this:
```
Found 2 tests.
Running `./integration-tests/transport.toml`  : Pass :)
Running `./integration-tests/time.toml`       : Pass :)

All tests passed!
```
Or if you are less lucky:
```
Found 2 tests.
Running `./integration-tests/transport.toml`  : Pass :)
Running `./integration-tests/time.toml`       : Fail!
in frame 2: in component `global_time`: in field `unix_timestamp_millis`: got 12345 expected 12000

Error: Some tests failed
```

## Creating test cases
> ℹ️ **Note**:
>
> The rest of this document assumes you know how the simulation architecture works. You can read
> more about it [here](simulators.md) and [here](simulator-communication.md).

All `.toml` files in `./integration-tests/` will be considered a test case, so start by making a new
`.toml` file there, the file name will be used as the test name.

As an example, here is an abridged version of the transport test case:
```toml
used_simulators = ["time-simulator", "load-flow"]
expected_output_file = "transport_output.csv"
amount_of_timesteps = 19
timestep_delta_seconds = 1

ignored_components = ["load_flow_analytics"]

[mock_simulator]
output_components = ["sensor_generator_node", "sensor_load_node"]
data_file = "transport_sensor_mock.csv"

# Expected components

[expected_components.global_time]
ty = "global"
structure = { unix_timestamp_millis = "i64"  }

[expected_components.energy_bases]
ty = "global"
structure = { p_base = "f64", v_base = "f64", s_base = "f64"}

[expected_components.energy_slack_node]
ty = "node"
structure = { voltage_amplitude = "f64", active_power = "f64", reactive_power = "f64", voltage_angle = "f64" }
< -- even more nodes -- >

[expected_components.energy_transmission_edge]
ty = "edge"
structure = { thermal_limit = "f64", resistance_per_meter = "f64", current = "f64", length = "f64", min_voltage_magnitude = "f64", line_type = "string", max_voltage_magnitude = "f64", reactance_per_meter = "f64"}

# Initial state

[initial_state.global]
global_time = { unix_timestamp_millis = 10000 }
energy_bases = { p_base = 1.0, v_base = 1.0, s_base = 1.0 }

[[initial_state.nodes]]
longitude = 4.0
latitude = 50.0
id = 0
components  = [
  { energy_load_node = { reactive_power = 0.0, voltage_angle = 0.0, active_power = 40.0, voltage_amplitude = 240.0 } },
  { sensor_load_node = { reactive_power = 0.25, active_power = 0.1 } },
]
< -- even more nodes -- >

[[initial_state.edges]]
from = 0
to = 1
component_type = "energy_transmission_edge"
component_data = { resistance_per_meter = 0.1, length = 8.0, line_type = "AAC_Conductor", reactance_per_meter = 0.8, max_voltage_magnitude = 500.0, min_voltage_magnitude = 0.0, thermal_limit = 20.0, current = 20.0 }
< -- another edge -- >

```

### Test case header
Every test starts with a header containing some general information every test case needs. In our
example:
```toml
used_simulators = ["time-simulator", "load-flow"]
expected_output_file = "transport_output.csv"
amount_of_timesteps = 19
timestep_delta_seconds = 1

ignored_components = ["load_flow_analytics"]
```
- `used_simulators`: List of simulator binary names to use in this test case.

  These names need to match the names of the simulator binaries from their `Cargo.toml`.
- `expected_output_file`: File name of the `.csv` file containing the expected output of the test
case. See [Expected output](#expected-output) fore more information.
- `amount_of_timesteps`: How many timesteps to run the simulation for.
- `timestep_delta_seconds`: Time, in seconds, between two timesteps of the simulation.

And optionally:
- `ignored_components`: list of component names that will be ignored in all future parts of the
testing process.

### Expected components
Every test also needs to specify all the component definitions that are expected to be used in the
simulation. The rest of test case definitions will be checked against this list, so it is important
you get this right. But don't worry, it is also the first thing the test runner will check against
the real manager. This implies you can start by only creating this section before moving on to
creating the rest of the test.

These expected components can be specified in a table `expected_components` like in the example:
```
[expected_components.global_time]
ty = "global"
structure = { unix_timestamp_millis = "i64"  }

[expected_components.energy_slack_node]
ty = "node"
structure = { voltage_amplitude = "f64", active_power = "f64", reactive_power = "f64", voltage_angle = "f64" }

[expected_components.energy_transmission_edge]
ty = "edge"
structure = { thermal_limit = "f64", resistance_per_meter = "f64", current = "f64", length = "f64", min_voltage_magnitude = "f64", line_type = "string", max_voltage_magnitude = "f64", reactance_per_meter = "f64"}
```
Here we specify tree components `global_time`, `energy_slack_node`, and `energy_transmission_edge`.
Every component needs a field `ty`, with one of the following strings: `"global"`, `"node"`,
or `"edge"`. And another field `structure` to give to structure of the component.

> 💡**Tip**:
>
> You can add the following temporary initial state:
> ```toml
> [initial_state]
> global = {}
> nodes = []
> edges = []
> ```
> to run the test runner as given above to make sure your components are correct.

### Initial state
Every test case also needs to specify an initial state of the graph that will be used at the start of
the simulation. Simulators will then be able to modify it. These modifications to the graph are what
you will actually be testing. But first, let's show how to specify the initial state with the
following example:
```toml
[initial_state.global]
global_time = { unix_timestamp_millis = 10000 }
energy_bases = { p_base = 1.0, v_base = 1.0, s_base = 1.0 }

[[initial_state.nodes]]
longitude = 4.0
latitude = 50.0
id = 0
components  = [
  { energy_load_node = { reactive_power = 0.0, voltage_angle = 0.0, active_power = 40.0, voltage_amplitude = 240.0 } },
  { sensor_load_node = { reactive_power = 0.25, active_power = 0.1 } },
]

[[initial_state.edges]]
from = 0
to = 1
component_type = "energy_transmission_edge"
component_data = { resistance_per_meter = 0.1, length = 8.0, line_type = "AAC_Conductor", reactance_per_meter = 0.8, max_voltage_magnitude = 500.0, min_voltage_magnitude = 0.0, thermal_limit = 20.0, current = 20.0 }  
```
The initial state consists of global components, a list of nodes, and a list of edges connecting
the nodes. For every global component you can just specify the component directly in the
`initial_state.global` table. The nodes live in a list `initial_state.nodes`. Every node needs a
`longitude`, `latitude`, `id` and some components. Components are specified in a list of maps
because toml does not allow new lines in a map.

Edges are stored in the `initial_state.edges` list. Every edge has a `from` and `to` field with the
ids of the nodes it connects. Since edges can only have a single component, they specify a single
component name in `componet_type` and the component values is `component_data`.

### Expected output
The expected output file specifies some fields in the graph to check for predefined values. This is
done using a `.csv` file pointed to by the `expected_output_file` in the header fields. The path is
relative to the location of the test case, so if both are in the same folder you can just us the file
name. Here is an example of such a file:
```csv
edge.0.current     ,edge.1.current     ,node.0.energy_load_node.voltage_amplitude
20                 ,20                 ,240
0.19771404527896183,0.08817425913985033,1.3622839030828535
0.3019691907240774 ,1.0028160187818949 ,1.8512385089171104
0.7285751304281037 ,202.6871815570757  ,1.6766314159064486
0.31265183038619565,27.891684051531364 ,0.11932971803901776
0.5781523836450886 ,0.5678432001131496 ,2.4469116757470397
0.9043495479609291 ,0.9593673233875196 ,3.126590902628981
1.2272900714304469 ,1.1842935559632717 ,3.439606843096507
1.2308202408133508 ,1.672135475451397  ,3.2676876267314356
0.9465622379478176 ,1.8496914325069    ,1.9269658575316748
```

The first line of the file is used to set the path of that column. So for example: something like
`node.0.energy_load_node.voltage_amplitude` means to look in the node with id `0` for a component
with the name `energy_load_node`. And then to check if the field `voltage_amplitude` in that
component has the values specified in the column bellow. The "validness" off the paths is checked
before the tests are done using expected components, so make sure those are correct.

Paths are strings that are separated by dots (`.`). The first part needs to be one of `global`,
`edge`, or `node`. The rest is type dependent:
- **`global`**: You can directly specify the name of the global component, and then the path to the
  field inside that component.

  For example: `global.global_time.unix_timestamp_millis` will look in global component with the
  name `global_time` for a field `unix_timestamp_millis`.
- **`node`**: You need to first specify the id of the node you want to inspect. Afterwards choose a
  component name, and then the path to the field in the component.

  See the first paragraph bellow the example file for an example.
- **`edge`**: For an edge you also need to give the id first. Ids for edges are there location in the
  `initial_state.edges` list. But you should not use the component name here since an edge only has
  a single component anyway.

  For example: `edge.1.current` will look in the component of the second edge (zero index) for a
  field `current`.

Only fields specified in the file expected output file will be checked, all other fields can take
any value and the test will still pass.

To make this whole process easier, the test runner provides some commands to help you.

#### Generate headers
```
./tools/run-integration-test.sh generate output-headers <PATH TO TEST>.toml
```
This command can be used to generate the headers for the expected output `.csv` file. It will read
the given test case file and use the `expected_output_file` there to create/replace the csv file
with a new one containing all the possible headers for the `intial_state` present in the test case.

#### Generate the output
```
./tools/run-integration-test.sh generate output <PATH TO TEST>.toml
```
This option will read the existing headers in the csv file from `expected_output_file` and run the
given test case. But instead of testing if the values match, it will replace the values with the
output of the simulation that was just run.

### Mock simulator
A test case can optionally contain a mock simulator. This can be used to, for example, run many
tests in one case by either changing the input configuration or replacing some non-deterministic
simulators. This is done by the test runner creating a new simulator that has output components
specified in the test case. The test case then also has a `.csv` file to specify the values of those
components for every frame. As an example, here is the mock simulator section from above:
```toml
[mock_simulator]
output_components = ["sensor_generator_node", "sensor_load_node"]
data_file = "transport_sensor_mock.csv"
```
The `output_components` field contains a list of components the mock simulator will advertise as
output components to the manger. As the mock simulator can't do any logic, it will never need input
components. The `data_file` is a `.csv` file with the same format as the [expected output file]
(#expected-output), containing for every frame what the mock simulator should output. Any field
not specified in this file will be filled with the value from the initial state for each frame.
Also, only components specified in `output_components` are allowed in this file. See below for
an example:
```csv
node.0.sensor_load_node.reactive_power,node.0.sensor_load_node.active_power,node.2.sensor_generator_node.voltage_magnitude,node.2.sensor_generator_node.active_power 
0.25                                  ,0.5                                 ,0.4                                           ,20.0
0.2                                   ,300                                 ,500                                           ,320
0.2                                   ,25                                  ,50                                            ,127
1                                     ,1                                   ,1                                             ,1
2                                     ,2                                   ,2                                             ,2
3                                     ,3                                   ,3                                             ,3
4                                     ,4                                   ,4                                             ,4
5                                     ,5                                   ,5                                             ,5
6                                     ,6                                   ,6                                             ,6
7                                     ,7                                   ,7                                             ,7
8                                     ,8                                   ,8                                             ,8
9                                     ,9                                   ,9                                             ,9
10                                    ,10                                  ,10                                            ,10
11                                    ,11                                  ,11                                            ,11
12                                    ,12                                  ,12                                            ,12
13                                    ,13                                  ,13                                            ,13
14                                    ,14                                  ,14                                            ,14
15                                    ,15                                  ,15                                            ,15
16                                    ,16                                  ,16                                            ,16
```
