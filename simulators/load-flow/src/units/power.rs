use num_complex::Complex;
/// Rate at which electrical energy is transferred by an electric circuit (P = active power, Q = reactive power).
#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
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

    #[allow(dead_code)]
    /// Creates a Power instance from a complex number (S = P + jQ).
    pub fn from_complex(complex: Complex<f64>) -> Self {
        Power {
            active: complex.re,
            reactive: complex.im,
        }
    }

    /// Returns the apparent power (S) in volt-amperes (VA) (S = sqrt(P^2 + Q^2) )
    pub fn apparent(&self) -> f64 {
        (self.active.powi(2) + self.reactive.powi(2)).sqrt()
    }

    #[allow(dead_code)]
    /// Calculates and returns the power factor. (Pf = P/S)
    pub fn power_factor(&self) -> f64 {
        self.active / self.apparent()
    }

    #[allow(dead_code)]
    /// Calculates and returns the phase angle in radians.
    pub fn phase_angle(&self) -> f64 {
        self.reactive.atan2(self.active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_to_complex() {
        let power = Power::new(1.0, 2.0);
        let complex = power.to_complex();
        assert_eq!(complex, Complex::new(1.0, 2.0));
    }

    #[test]
    fn test_power_from_complex() {
        let complex = Complex::new(1.0, 2.0);
        let power = Power::from_complex(complex);
        assert_eq!(complex.re, power.active);
        assert_eq!(complex.im, power.reactive);
    }

    #[test]
    fn test_power_apparent() {
        let power = Power::new(3.0, 4.0);
        assert_eq!(power.apparent(), 5.0);
    }

    #[test]
    fn test_power_power_factor() {
        let power = Power::new(3.0, 4.0);
        assert_eq!(power.power_factor(), 0.6);
    }

    #[test]
    fn test_power_phase_angle() {
        let power = Power::new(3.0, 4.0);
        assert_eq!(power.phase_angle(), 0.9272952180016122);
        let p = Power::from_complex(Complex::new(-0.25, -7.96));
        assert_eq!(p.active, -0.25);
        assert_eq!(p.reactive, -7.96);
    }
}
