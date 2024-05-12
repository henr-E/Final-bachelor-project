# Weather simulator

This simulator is responsible for simulation of the weather for the digital twin.

## Components

The simulator makes use of multiple global components:

1. **TimeComponent (required)**: A component to track the time and date in the simulation.
2. **TemperatureComponent (optional/output)**: A component that contains the temperature in degrees Celsius.
3. **PrecipitationComponent (optional/output)**: A component that contains the amount of precipitation in mm/hour.
4. **WindSpeedComponent (optional/output)**: A component representing the wind speed in m/s.
5. **WindDirectionComponent (optional/output)**: A component to represent the wind direction in degrees.
6. **IrradianceComponent (optional/output)**: A component representing solar irradiance in W/m^2.

## Explanation

The weather simulator makes use of weather data captured by registered global sensors. Data captured
for quantities 'Temperature', 'Rainfall', 'WindSpeed', 'WindDirection' and 'Irradiance' is considered relevant
for the weather simulator. For each weather related component that the simulation manager passes to the weather
simulator,
the associated sensor data is retrieved. This data is then used by a prediction model in order to predict the weather
variables for the current frame.
The components are then updated using the predicted values and returned to the simulation manager.
In case no sensor data is found for the component, the value initially provided by the user will be returned.

The components 'TemperatureComponent', 'PrecipitationComponent', 'WindSpeedComponent' and 'IrradianceComponent' contain,
besides the weather variable they represent, also a scalar. This scalar is used to multiply the predicted value before
updating the component.


