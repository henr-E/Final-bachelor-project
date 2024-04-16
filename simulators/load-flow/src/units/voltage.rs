use num_complex::Complex;

/// Electrical potential difference between two points, driving the flow of electric current
#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub struct Voltage {
    /// Amplitude (|V|) in volts (V), the RMS value indicating the effective voltage level.
    pub amplitude: f64,
    /// Phase angle (δ) of voltage in radians, indicating the shift from a reference phase.
    pub angle: f64,
}

/// Electrical potential difference, the force that drives electric current around a circuit.
impl Voltage {
    /// Creates a new instance of `Voltage` with given amplitude and angle.
    pub fn new(amplitude: f64, angle: f64) -> Self {
        Voltage { amplitude, angle }
    }

    /// Returns the complex representation of this voltage instance.
    /// Converts polar form (amplitude and angle) to rectangular form (real and imaginary).
    pub fn to_complex(self) -> Complex<f64> {
        Complex::from_polar(self.amplitude, self.angle)
    }

    /// Creates a `Voltage` instance from a complex number representation.
    /// Converts rectangular form (real and imaginary) to polar form (amplitude and angle).
    pub fn from_complex(complex: Complex<f64>) -> Self {
        Voltage {
            amplitude: complex.norm(), // Magnitude of the complex number.
            angle: complex.arg(),      // Angle of the complex number in radians.
        }
    }
}
