# Load flow analysis

## Table of contents
1. [Introduction](#introduction)
2. [Theory](#theory)
   - [Three Node Types in Power-Flow Studies](#three-node-types-in-power-flow-studies)
   - [Gauss-Seidel Iterative Method](#gauss-seidel-iterative-method)
   - [Newton-Raphson Iterative Method](#newton-raphson-iterative-method)
3. [Usage](#usage)

## Introduction
In power engineering, the power-flow study, or load-flow study, is a numerical analysis of the flow of electric power in an interconnected system. A power-flow study usually uses simplified notations such as a one-line diagram, and focuses on various aspects of AC power parameters, such as voltages, voltage angles, real power and reactive power. It analyzes the power systems in normal steady-state operation.

Power-flow or load-flow studies are important for planning future expansion of power systems as well as in determining the best operation of existing systems. The principal information obtained from the power-flow study is the magnitude and phase angle of the voltage at each bus, and the real and reactive power flowing in each line.

Wikipedia contributors. (2024, January 2). Power-flow study [Wikipedia](https://en.wikipedia.org/wiki/Power-flow_study)

## Theory

### Three Node Types in Power-Flow Studies

In the context of power-flow studies, nodes (or buses) in an electrical power system are categorized into three main types based on the parameters that are specified and those that are calculated:

1.  **Slack (or Swing) Bus**: This is typically the reference bus for all calculations within the system, serving as the balance point for the power system. The voltage magnitude and angle at this bus are assumed to be known and fixed. The slack bus compensates for system losses and maintains the balance between the generated power and the load within the system by adjusting its output accordingly. In essence, it provides the active power and reactive power necessary to maintain the system's balance.

2.  **PQ Bus (Load Bus)**: For these buses, the active (P) and reactive (Q) power loads are known quantities. The voltage magnitude and angle are to be determined through the load flow calculations. PQ buses represent the majority of buses in a power system and are where consumers (residential, commercial, and industrial loads) are connected.
    
3.  **PV Bus (Generator Bus)**: At these buses, the active power generated (P) and the voltage magnitude (V) are known, while the reactive power (Q) and the voltage angle are to be calculated. These buses typically represent generator connections within the power system, where the generator output voltage is controlled to a set value, but the reactive power can vary.

### Gauss-Seidel Iterative Method
The Gauss-Seidel method is an iterative technique used to solve the power-flow problem in an electrical network. It's particularly useful for solving the set of nonlinear equations that arise in load-flow studies. The basic steps of the Gauss-Seidel method are as follows:

1.  **Initialization**: Start by initializing the voltage at all buses. A common initial guess is to set all voltage magnitudes to 1.0 p.u. (per unit) and all angles to zero, except for the slack bus, which is set to its specified value. (Which can also be zero)

2.  **Iteration**: For each iteration, the method calculates the new values of the voltage at each PQ and PV bus based on the power balance equations and the current estimates of all other bus voltages. The Gauss-Seidel formula for updating the voltage at each bus $i$ is given by:

$$
V_i^{(\text{new})} = \frac{1}{Y_{ii}} \left( P_i + jQ_i - \sum_{k \neq i} Y_{ik}V_k^{(\text{old})} \right)
$$

Where:
- $V_i^{(\text{new})}$ is the new voltage at bus $i$,
- $Y_{ii}$ is the admittance of bus $i$ to itself,
- $P_i$ and $Q_i$ are the real and reactive power at bus $i$,
- $V_k^{(\text{old})}$ are the voltages at other buses $k$ from the previous iteration.

3.  **Convergence Check**: After updating the voltages at all buses, the differences between the old and new voltages are checked. If the differences for all buses are below a specified tolerance, the process is considered to have converged, and the current values are taken as the solution. If not, the process is repeated.

### Notes
We chose Gauss-Seidel for its simplicity and ease of implementation, especially for small to medium-sized systems. However, for very large systems or systems with high R/X ratios, the method may converge slowly or not at all. This is a problem that needs to be addressed in future versions.

## Newton-Raphson Iterative Method
1.  **Initialization**: Just Like the Gauss-Seidel Method, one starts by initializing the voltage at all buses. A common initial guess is to set all voltage magnitudes to 1.0 p.u. (per unit) and all angles to zero, except for the slack bus, which is set to its specified value. (Which can also be zero)

2.  **Iteration**: The Newton-Raphson works solving the mismatch power equations and finding the Jacobian matrix for a given graph.
The general power equations for the load and generator node are:
Active Power:

$$P_{i}=\sum_{k=1}^{n}\mid V_i \mid .\mid V_k \mid G_{ik} \cos {\theta_{ik}}+B_{ik} \sin {\theta_{ik}}=P{Gi}-P{Di}$$

Reactive Power:

$$Q_{i}=\sum_{k=1}^{n}\mid V_i \mid .\mid V_k \mid G_{ik} \sin {\theta_{ik}}-B_{ik} \cos {\theta_{ik}}=Q{Gi}-Q{Di}$$


Where:
- $V_i$ is the voltage at bus $i$,
- $\theta_{ik}$ is the the difference in voltage angle between bus i and k,
- $G_{ik}$ is the real part of the admittance matrix at position ik,
- $B_{ik}$ is the imaginary part of the admittance matrix at position ik.
- $P{Gi}-P{Di}$ is the difference between the "guess" active power of node i and determined or calculated active power of node i.
- $Q{Gi}-Q{Di}$ is the difference between the "guess" reactive power of node i and the determined or calculated reactive power of node i.

Next we construct the Jacobian matrix as follows:

$$
\begin{bmatrix}
\frac{\partial P_1}{\partial \theta_1} & \cdots & \frac{\partial P_1}{\partial \theta_N} & \frac{\partial P_1}{\partial V_1} & \cdots & \frac{\partial P_1}{\partial V_N} \\
\vdots & \ddots & \vdots & \vdots & \ddots & \vdots \\
\frac{\partial P_N}{\partial \theta_1} & \cdots & \frac{\partial P_N}{\partial \theta_N} & \frac{\partial P_N}{\partial V_1} & \cdots & \frac{\partial P_N}{\partial V_N} \\
\frac{\partial Q_1}{\partial \theta_1} & \cdots & \frac{\partial Q_1}{\partial \theta_N} & \frac{\partial Q_1}{\partial V_1} & \cdots & \frac{\partial Q_1}{\partial V_N} \\
\vdots & \ddots & \vdots & \vdots & \ddots & \vdots \\
\frac{\partial Q_N}{\partial \theta_1} & \cdots & \frac{\partial Q_N}{\partial \theta_N} & \frac{\partial Q_N}{\partial V_1} & \cdots & \frac{\partial Q_N}{\partial V_N}
\end{bmatrix}
$$

This matrix consists of:
- The top-left submatrix: The partial derivatives of real power flows $P_{i}$ with respect to voltage angles ${\theta_i}$,
- The top-right submatrix: The partial derivatives of real power flows $P_{i}$ with respect to voltage magnitudes $V_{i}$,
- The bottom-left submatrix: The partial derivatives of reactive power flows $Q_{i}$ with respect to voltage angles $\theta_{i}$,
- The bottom-right submatrix: The partial derivatives of reactive power flows $Q_{i}$ with respect to voltage magnitudes $V_{i}$.

Finally the new voltages can be calculated as such:

$$
\begin{bmatrix}
\Delta \theta \\
\Delta |V|
\end{bmatrix} = -J^{-1} \begin{bmatrix} \Delta P \\ \Delta Q \end{bmatrix}
$$

In the equation , the inverse of the Jacobian $J^{-1}$ is used to calculate the new voltages $i$. Just like in the Gauss-Seidel method, we check for convergence. We say the algorithm has converged when the difference between the old value and the calculated value is less than a specified tolerance. In case the determinant is zero (that is we can't inverse it) we apply damping to the diagonal elements to make it invertible.
 
## Notes
This linear equations solver is more complex than the Gauss-Seidel method. However, we have seen a huge performance improvement when testing out new graphs.
- Larger graphs converge more often and faster than the Gauss-Seidel method.
- Even when the initial guesses for the voltages are completely off, The Newton-Raphson method will (often) find a solution in a reasonable amount of iterations.


## Usage
The simulation requires pairing between sensor nodes and their corresponding real nodes to function correctly. Below are the guidelines to properly configure and use the simulation system:

### Configuration Requirements

1.  **Node Pairing**: Each LoadNode or GeneratorNode must have a corresponding SensorLoadNode or SensorGeneratorNode. The sensor nodes will contain either measured data or simulated values, which the simulation uses to compute outputs for the real nodes.
    
2.  **Initial Values**: Initialize real nodes with any placeholder values. These values will be overwritten with the results from the simulation.
    
3.  **Sensor Attachment**: It's essential to attach sensors to the nodes. Without sensors, the simulation will only use the initial static values. Sensors enable the use of regression models to enhance simulation accuracy.