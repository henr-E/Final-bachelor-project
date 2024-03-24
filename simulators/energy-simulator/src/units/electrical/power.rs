use num_complex::Complex;

/// Rate at which electrical energy is transferred by an electric circuit (P = active power, Q = reactive power).
#[derive(Clone, Debug, Copy)]
pub struct Power {
    /// Active power (P) in watts (W), representing the real power used by the system.
    pub active: f64,
    /// Reactive power (Q) in volt-amperes reactive (var), representing power stored and released by the system.
    pub reactive: f64,
}

impl Power {
    /// Creates a new instance of `Power` with given active and reactive component.
    pub fn new(active: f64, reactive: f64) -> Self {
        Power { active, reactive }
    }

    /// Returns the complex representation of power (S = P + jQ).
    pub fn to_complex(self) -> Complex<f64> {
        Complex::new(self.active, self.reactive)
    }

    /// Creates a Power instance from a complex number (S = P + jQ).
    pub fn from_complex(complex: Complex<f64>) -> Self {
        Power {
            active: complex.re,
            reactive: complex.im,
        }
    }
}
