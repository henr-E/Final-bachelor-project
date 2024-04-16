# Energy component library for simulators

## Overview

This module contains a series of components, that are made to simulate the different aspects of energy-management and distribution. This is only an initial library as we might add more components in the future.


### Common units for AC circuits:
#### Power
- **Active power**: This is the portion of electrical power that is actually consumed by a device and converted into useful work, such as producing heat, light, or mechanical motion. It is measured in watts (W) and represents the rate at which energy is transferred or used by a circuit.
- **Reactive power**: This is the power that flows back and forth between the source and the load to support the operation of inductive loads and maintain voltage levels. Reactive power is measured in volt-amperes reactive (VAR).

Both active and reactive power are important considerations in electrical systems to ensure efficient and reliable operation.

#### Voltage
Voltage amplitude and angle are two important parameters used to describe an alternating current (AC) voltage waveform.
- **Voltage Amplitude**: This refers to the maximum magnitude of the voltage waveform. In an AC circuit, the voltage continuously oscillates between positive and negative values. The voltage amplitude represents the maximum positive or negative value reached by the voltage waveform during each cycle. It is typically measured in volts (V).
- **Voltage Angle**: This is a measure of the displacement of an AC voltage waveform relative to a reference waveform. It represents the angular position of the voltage waveform at a specific point in time. Voltage angle is often expressed in degrees (°) or radians (rad). In a sinusoidal waveform, the voltage angle determines the position of the waveform relative to its peak or zero crossing.

Together they provide a complete description of the AC voltage waveform and are essential for analyzing and understanding AC circuits. 

### p.u system
In electrical engineering, per unit (p.u.) system is a method used to express the values of electrical quantities relative to a chosen base value. Per unit values are dimensionless and are typically used to simplify calculations and comparisons in power system analysis.

- **Bases**: Per unit values are expressed relative to a chosen base value for each quantity. The base value is typically selected to be equal to the rated or nominal value of the quantity being considered. For example, the base values for voltage, current, and power are often chosen to be the rated voltage, current, and apparent power of a power system. For the load flow, the two required bases are:
    * **Sbase**: This refers to the base power level in a power system, often expressed in kilowatts (VA) or megawatts (MVA). (The formula for apparent power is $S = VI^*$) It serves as a reference point for calculating other system parameters, such as voltages, currents, and impedances.
    * **Vbase**: This refers to the base voltage level in a power system, often expressed in kilovolts (kV) or volts (V). Similar to Sbase, Vbase serves as a reference point for voltage calculations and is used to normalize voltage values in the system.
    * **Pbase**: This refers to the base active power level in a power system, often expressed in watts (W) or megawatts (MW). 

- **Normalization**: To express a value in per unit units, the actual value is divided by the corresponding base value. This normalization process results in dimensionless quantities, which simplifies calculations and allows for easy comparison between different systems with varying ratings.

## Components

The module contains the following energy-related nodes:

### Global Components

- **TimeComponent**: Represents the current time of a frame in the simulation, accurate up to the millisecond.
- **TemperatureComponent**: Represents the current temperature in degrees Celsius.

### General Energy Components

- **TransmissionEdge**: Represents an edge in the energy system with properties like resistance per meter, reactance per meter, length, and cable type.

#### Load Flow Analysis Specific Components

- **LoadNode**: Represents a node in the energy system with properties like voltage amplitude, voltage angle, active power, and reactive power.
- **GeneratorNode**: Represents a generator node in the energy system with properties like voltage amplitude, voltage angle, active power, reactive power, and power type.
- **SlackNode**: Is a mathematical node used for the load-flow simulator. This node serves as a reference point with known voltage magnitude and angle, facilitating power flow analysis and ensuring the balance of power generation and consumption within the system.
- **EnergyLoadFlow**: This node is used to return some analytics data on the graph the user has created (such as total generators/total load/..)
- **ProductionOverview**: this node is an element for the 'energy_production_overview' of the EnergyLoadFlow. The EnergyType produced in the graph with the percentage of the amount of energy it accounts for in the system is added as a element in this structure and saved in energy_production_overview.
#### Supply Demand Specific Components

- **ProducerNode**: Reperensents a node for load-flow simulator. Has properties like voltage amplitude, voltage angle and active power.
- **ConsumerNode**: Represents a node for load-flow simulator. Has properties like voltage amplitude, voltage angle, active power and reactive power.


#### Additional Types

- **CableType**: Enumerates different types of cables used in the transmission edges.
- **PowerType**: Enumerates different types of power sources used in generator nodes.
The library provides functionality to serialize and deserialize these components and types, making it easy to integrate with other systems and tools.
