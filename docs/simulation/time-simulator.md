# Time simulator

This simulator is responsible for progressing time in the simulation.

## Components

The simulator makes use of one global component:

1. **TimeComponent (required/output)**: A component to track the time and date in the simulation. The time is
   represented as a unix timestamp in milliseconds.

## Explanation

The time simulator receives a graph with one 'TimeComponent' component from the simulation manager.
For every frame, the time simulator increases the value found inside the component by the frame time duration specified
by the user.
This updated component is then returned to the simulation manager.