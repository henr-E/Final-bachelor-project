# Energy supply and demand simulator

This simulator is responsible for simulating the energy supply and demand.
Both energy supply and energy demand should vary with date and time, and they are also affected by weather variables.
For example, the energy required for household heating will increase as the outdoor temperature decreases. Renewable
energy sources such as solar panels and wind turbines produce different amounts of energy based on weather variables
such as solar irradiance and wind speed.

## Components

The simulator makes use of multiple components:

1. **TimeComponent (required)**: A component to track the time and date in the simulation.
2. **Building (required)**: A component to represent a building. The components contain a building id, that corresponds
   to a building in the digital twin. Buildings are usually associated with an energy consumer and/or energy producer.
3. **SensorLoadNode (required/output)**: A component that represents an entity that demands energy. The demanded energy
   is expressed in watts.
4. **SensorGeneratorNode (required/output)**: A component that represents an entity that produces energy. The produced
   energy is expressed in watts. The type<sup>[1](#power_type)</sup> of the energy source should be specified as well.
5. **WindSpeedComponent (optional)**: A component representing the wind speed in m/s.
6. **IrradianceComponent (optional)**: A component representing solar irradiance in W/m^2.
7. **SupplyAndDemandAnalytics (optional/output)**: : A component that contains some analytics about the energy supply
   and demand simulation. For example the number of energy consumer and producers, the total demand, ... This component
   can also contain an overview of the different types<sup>[1](#power_type)</sup> of energy sources present in the
   simulation and their respective contribution (expressed in percentage) to the total produced energy.

## Explanation

The simulation manager will pass a graph containing one or more building components to the energy-supply-and-demand
simulator.
Since a building component contains a building id, the sensor that is associated with a building and its captured
data can be retrieved.
For buildings, the energy-supply-and-demand simulator is mainly interested in sensors measuring values for the
quantity 'Power'.
Data captured by global sensors for quantities 'Temperature, 'WindSpeed' and 'Irradiance' is then combined with the
power sensor data of the building. The simulator expects that a global sensor for these weather related quantities
exists.
The combined data is then presented to the prediction model, which for each building with a power sensor attached will
train a version of itself individually.

For each 'SensorLoadNode' component in the graph, the associated 'Building' component can be found. The trained
prediction model will
predict values for the energy demand of the building and updates the 'SensorLoadNode' component. In the end, the '
SensorLoadNode' components will be returned to the simulation manager. In case that no 'Building' component corresponds
to the 'SensorLoadNode' component or no sensor data for the power consumption of the building is present, the
SensorLoadNode will just be returned unchanged.

A graph can also contain one or more 'SensorGeneratorNode' components. These components present energy producers and are
associated with a power type<sup>[1](#power_type)</sup>.
For nuclear, solar and wind power the produced energy is calculated using simple formulas. For solar and wind power, the
formulas make use of a parameter representing the irradiance and wind speed respectively.
The values for these parameters are obtained from the "WindSpeedComponent" and the "IrradianceComponent" in case they
are present in the graph.

<a name="power_type">1</a>: The type of the energy source should be one of: 'Fossil', 'Renewable', 'Nuclear', 'Hydro', '
Solar', 'Wind', 'Battery', 'Storage'
