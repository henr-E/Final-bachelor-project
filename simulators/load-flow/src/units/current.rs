use num_complex::Complex;

/// Flow of electrical charge carriers, typically measured in amperes (A).
#[derive(Clone, Debug, Copy)]
pub struct Current {
    /// Magnitude (|I|) in amperes (A), the RMS value indicating the effective current level.
    pub magnitude: f64,
    /// Phase angle (θ) of current in radians, showing its shift from a reference phase.
    pub angle: f64,
}

#[allow(dead_code)]
impl Current {
    /// Creates a new instance of `Current` with given magnitude and angle.
    pub fn new(magnitude: f64, angle: f64) -> Self {
        Current { magnitude, angle }
    }

    /// Creates a `Current` instance from a complex number representation.
    /// Converts rectangular form (real and imaginary) to polar form (magnitude and angle).
    pub fn from_complex(complex: Complex<f64>) -> Self {
        Current {
            magnitude: complex.norm(), // Magnitude of the complex number.
            angle: complex.arg(),      // Angle of the complex number in radians.
        }
    }

    /// Converts the current to a complex number representation.
    pub fn to_complex(self) -> Complex<f64> {
        Complex::new(
            self.magnitude * self.angle.cos(),
            self.magnitude * self.angle.sin(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let current = Current::new(1.0, 0.0);
        assert_eq!(current.magnitude, 1.0);
        assert_eq!(current.angle, 0.0);
    }
}
