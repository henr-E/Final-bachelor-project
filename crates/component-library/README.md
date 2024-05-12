# Energy component library for simulators

## Overview

This module contains a series of components designed to simulate different aspects of energy management and distribution. This is only an initial library, and more components may be added in the future.

### Common units for AC circuits:

#### Power
- **Active power**: Power consumed by a device and converted into useful work, such as producing heat, light, or mechanical motion. Measured in watts (W).
- **Reactive power**: Power that flows back and forth between the source and the load, used to support inductive loads and maintain voltage levels. Measured in volt-amperes reactive (VAR).

#### Voltage
- **Voltage Amplitude**: Maximum magnitude of the voltage waveform. Measured in volts (V).
- **Voltage Angle**: Measure of the displacement of an AC voltage waveform relative to a reference waveform. Measured in radians (rad).

## Components

### Global Components Overview
- **TimeComponent**: Tracks the current simulation time. Unit: Millisecond.

### Weather Components
- **TemperatureComponent**: Tracks current temperature. Unit: Degrees Celsius (°C).
- **PrecipitationComponent**: Tracks rainfall amount. Unit: Millimeters per hour (mm/h).
- **IrradianceComponent**: Tracks solar power received. Unit: Watts per square meter (W/m²).
- **IlluminanceComponent**: Tracks light intensity. Unit: Lux (lx).

### General Energy Components
- **TransmissionEdge**: Represents an edge in the energy system, including resistance and reactance per meter, length, and cable type. Includes min/max voltage magnitude and thermal limit.

#### Sensor Energy Components
- **SensorGeneratorNode**: Stores data for generator nodes in load flow analysis. Includes active power and voltage magnitude.
- **SensorLoadNode**: Stores data for load nodes in load flow analysis. Includes active power and reactive power.

#### Load Flow Analysis Specific Components
- **LoadNode**: Represents a node with voltage amplitude, voltage angle, active power, and reactive power.
- **GeneratorNode**: Represents a generator node with voltage amplitude, voltage angle, active power, reactive power, and power type. Includes min/max reactive power limits.
- **SlackNode**: Serves as a reference point in load flow simulations with known voltage magnitude and angle.
- **EnergyLoadFlow**: Provides analytics data on the created graph, such as total generators and total load.
- **ProductionOverview**: Part of 'energy_production_overview', showing energy type produced and its percentage in the system.

#### Supply Demand Specific Components
- **ProducerNode**: Node in load-flow simulation with voltage amplitude, voltage angle, and active power.
- **ConsumerNode**: Node in load-flow simulation with voltage amplitude, voltage angle, active power, and reactive power.

## Additional Enum Types

### CableType
Enumerates types of cables used in transmission edges.
- Above Ground:
    - `ACSR_Conductor`: Aluminum Conductor Steel Reinforced cable.
    - `AAC_Conductor`: All Aluminum Conductor cable.
    - `AAAC_Conductor`: All Aluminum Alloy Conductor cable.
- Underground:
    - `XLPE_Cable`: Cross-linked Polyethylene cable.
    - `PILC_Cable`: Paper Insulated Lead Covered cable.

### PowerType
Enumerates types of power sources used in generator nodes.
- `Fossil`: Power derived from fossil fuels like coal, oil, or natural gas.
- `Renewable`: Power derived from renewable energy sources.
- `Nuclear`: Power derived from nuclear reactors.
- `Hydro`: Power derived from hydroelectric power plants.
- `Solar`: Power derived from solar panels.
- `Wind`: Power derived from wind turbines.
- `Battery`: Power stored in battery systems.
- `Storage`: General category for power storage systems.

### LoadFlowSolvers
Enumerates possible solvers for load flow analysis.
- `GaussSeidel`: Uses the Gauss-Seidel method for solving load flow problems.
- `NewtonRaphson`: Uses the Newton-Raphson method for solving load flow problems.

The library provides functionality to serialize and deserialize these components and types, facilitating integration with other systems and tools.
