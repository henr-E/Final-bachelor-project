use num_complex::Complex;

/// Measure of opposition that a circuit presents to a current when a voltage is applied (combines resistance and reactance).
#[derive(Clone, Debug, Copy)]
pub struct Impedance {
    /// Resistance (R) in ohms (Ω), representing the real part of impedance that dissipates energy.
    pub resistance: f64,
    /// Reactance (X) in ohms (Ω), representing the imaginary part of impedance that stores energy.
    pub reactance: f64,
}

impl Impedance {
    /// Returns the complex representation of this impedance instance (Z = R + jX).
    pub fn to_complex(self) -> Complex<f64> {
        Complex::new(self.resistance, self.reactance)
    }

    /// Creates an Impedance instance from a complex number (Z = R + jX).
    pub fn from_complex(complex: Complex<f64>) -> Self {
        Impedance {
            resistance: complex.re,
            reactance: complex.im,
        }
    }
}
