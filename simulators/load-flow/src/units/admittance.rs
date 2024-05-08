use crate::units::impedance::Impedance;
use num_complex::Complex;
use num_complex::ComplexFloat;

/// Measure of how easily a circuit allows electrical current to flow (inverse of impedance).
#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub struct Admittance {
    /// Conductance (G) in siemens (S), indicating the real part of admittance facilitating power flow.
    pub conductance: f64,
    /// Susceptance (B) in siemens (S), indicating the imaginary part of admittance storing and releasing energy.
    pub susceptance: f64,
}

#[allow(dead_code)]
impl Admittance {
    /// Creates a new instance of `Admittance` with given conductance and susceptance.
    pub fn new(conductance: f64, susceptance: f64) -> Self {
        Admittance {
            conductance,
            susceptance,
        }
    }

    /// Returns the complex representation of this admittance instance (Y = G + jB).
    pub fn to_complex(self) -> Complex<f64> {
        Complex::new(self.conductance, self.susceptance)
    }

    /// Creates a new instance of `Admittance` from complex representation (Y = G + jB)
    pub fn from_complex(complex: Complex<f64>) -> Self {
        Admittance {
            conductance: complex.re,
            susceptance: complex.im,
        }
    }

    /// Creates a new instance of `Admittance` from and `Impedance` object (Y = 1/Z)
    pub fn from_impedance(impedance: Impedance) -> Self {
        Self::from_complex(impedance.to_complex().recip())
    }
}

impl std::ops::Add for Admittance {
    type Output = Self;

    /// Performs the `+` operation
    fn add(self, other: Self) -> Self {
        Admittance {
            conductance: self.conductance + other.conductance,
            susceptance: self.susceptance + other.susceptance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admittance_to_complex() {
        let admittance = Admittance::new(1.0, 2.0);
        let complex = admittance.to_complex();
        assert_eq!(complex, Complex::new(1.0, 2.0));
    }

    #[test]
    fn test_admittance_from_complex() {
        let complex = Complex::new(1.0, 2.0);
        let admittance = Admittance::from_complex(complex);
        assert_eq!(admittance, Admittance::new(1.0, 2.0));
    }

    #[test]
    fn test_admittance_from_impedance() {
        let complex = Complex::new(1.0, 2.0).recip();
        let impedance = Impedance::from_complex(complex);
        let admittance = Admittance::from_impedance(impedance);
        assert_eq!(admittance.to_complex(), complex.recip());
    }
}
