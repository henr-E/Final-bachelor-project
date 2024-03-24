use num_complex::Complex;

/// Measure of how easily a circuit allows electrical current to flow (inverse of impedance).
#[derive(Clone, Debug, Copy)]
pub struct Admittance {
    /// Conductance (G) in siemens (S), indicating the real part of admittance facilitating power flow.
    pub conductance: f64,
    /// Susceptance (B) in siemens (S), indicating the imaginary part of admittance storing and releasing energy.
    pub susceptance: f64,
}

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
}
impl std::ops::Add for Admittance {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Admittance {
            conductance: self.conductance + other.conductance,
            susceptance: self.susceptance + other.susceptance,
        }
    }
}
