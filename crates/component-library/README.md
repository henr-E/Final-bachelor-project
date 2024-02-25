# Energycomponent library for simulators

## Overview

This module contains a series of components, that are made to simulate the different aspects of energy-management and distribution. This is only an initial library as we might add more components in the future.

## Components

The module contains the following energy-related nodes:

- **EnergyConsumerNode**: Simulates energy demand and how it responds to price changes.
- **EnergyStorageNode**: Represents storage facilities that can store and release energy, taking into account capacity, charge/discharge rates and efficiency.
- **EnergyProducerNode**: Models different types of energy producers, such as fossil fuels, nuclear or renewable energy sources, including their production capacity.
- **EnergyTransmissionNode**: Simulates the transmission of energy over long distances, including operating voltage, capacitance and line resistance.

Each node contains a `GenericEnergyNode` structure that contains common data for all types of energy nodes, such as the active status and the current energy supply.
