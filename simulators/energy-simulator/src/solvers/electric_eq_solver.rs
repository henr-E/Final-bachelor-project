use num_complex::Complex;
use std::collections::HashMap;

use crate::nodes::electric::generic::BusType;
use crate::units::electrical::admittance::Admittance;
use crate::units::electrical::power::Power;
use crate::units::electrical::voltage::Voltage;

/// Single round of Gauss-Seidel, to be used in a distributed fashion
pub fn distributed_round(
    node: (Power, Voltage, BusType),
    neighbours: HashMap<usize, Voltage>,
    lines: HashMap<usize, Admittance>,

    //id of line, voltage magnitude and phase angle difference
    slack_neighbours: Option<HashMap<usize, (Voltage, f64)>>,
) -> Result<(Power, Voltage, BusType), String> {
    let mut sum_yv = Complex::new(0.0, 0.0);
    for (&key, voltage) in &neighbours {
        let line = lines
            .get(&key)
            .ok_or_else(|| format!("Line with key {} not found", key))?;
        sum_yv += line.to_complex() * voltage.to_complex();
    }
    if let Some(neighbours) = slack_neighbours {
        for (&key, (_voltage, _angle)) in &neighbours {
            let _line = lines
                .get(&key)
                .ok_or_else(|| format!("Line with key {} not found", key))?;
        }
    }

    let s_i = node.0.to_complex();
    let v_i_old = node.1.to_complex();
    let y_ii = lines
        .values()
        .fold(Complex::new(0.0, 0.0), |acc, x| acc + x.to_complex());

    let v_i_new = (s_i / v_i_old.conj() - sum_yv) / y_ii;

    Ok((
        Power::from_complex(s_i),
        Voltage::from_complex(v_i_new),
        node.clone().2,
    ))
}

pub fn converge_gauss_seidel(
    tolerance: f64,
    max_iter: u32,
    lines: HashMap<usize, Admittance>,
    neighbours: HashMap<usize, Voltage>,
    node: (Power, Voltage, BusType),
    recursion: u32,
) -> (Power, Voltage, BusType, u32) {
    if recursion >= max_iter {
        // Maximum iterations reached, return the current result
        (node.0, node.1, node.2, recursion)
    } else {
        // Call the function
        let new_result = distributed_round(
            node.clone(),
            neighbours.clone(),
            lines.clone(),
            Some(HashMap::new()),
        )
        .unwrap();
        // Check the termination condition

        if (new_result.1.to_complex().re - node.clone().1.to_complex().re).abs() < tolerance
            && (new_result.1.to_complex().im - node.clone().1.to_complex().im).abs() < tolerance
            && recursion > 0
        {
            // Termination condition met, return the current result

            (new_result.0, new_result.1, new_result.2, recursion)
        } else {
            // Recursively call the function with updated values
            converge_gauss_seidel(
                tolerance,
                max_iter,
                lines,
                neighbours,
                new_result,
                recursion + 1,
            )
        }
    }
}
