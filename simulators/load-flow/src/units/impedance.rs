use num_complex::Complex;

#[allow(dead_code)]
/// Measure of opposition that a circuit presents to a current when a voltage is applied (combines resistance and reactance).
#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub struct Impedance {
    /// Resistance (R) in ohms (Ω), representing the real part of impedance that dissipates energy.
    pub resistance: f64,
    /// Reactance (X) in ohms (Ω), representing the imaginary part of impedance that stores energy.
    pub reactance: f64,
}

impl Impedance {
    /// Returns the complex representation of this impedance instance (Z = R + jX).
    pub fn new(resistance: f64, reactance: f64) -> Self {
        Impedance {
            resistance,
            reactance,
        }
    }

    /// Returns the complex representation of this impedance instance (Z = R + jX).
    pub fn to_complex(self) -> Complex<f64> {
        Complex::new(self.resistance, self.reactance)
    }

    #[allow(dead_code)]
    /// Creates an Impedance instance from a complex number (Z = R + jX).
    pub fn from_complex(complex: Complex<f64>) -> Self {
        Impedance {
            resistance: complex.re,
            reactance: complex.im,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impedance_to_complex() {
        let impedance = Impedance::new(1.0, 2.0);
        let complex = impedance.to_complex();
        assert_eq!(complex, Complex::new(1.0, 2.0));
    }

    #[test]
    fn test_impedance_from_complex() {
        let complex = Complex::new(1.0, 2.0);
        let impedance = Impedance::from_complex(complex);
        assert_eq!(complex.re, impedance.resistance);
        assert_eq!(complex.im, impedance.reactance);
    }
}
