use std::f64::consts::PI;

use crate::units::current::Current;
use crate::units::impedance::Impedance;
use crate::units::voltage::Voltage;

/// Frequency in hertz, for calculating reactance
const FREQUENCY: f64 = 50.0;

/// An enum containing line types based on the most relevant materials used in power nets
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum LineType {
    /// Overhead Lines
    /// Aluminum Conductor Steel Reinforced
    ACSRConductor,
    /// All Aluminum Conductor
    AACConductor,
    /// All Aluminum Alloy Conductor
    AAACConductor,

    /// Underground Cables
    /// Cross-Linked Polyethylene insulated cable
    XLPECable,
    /// Paper-Insulated Lead-Covered Cable
    PILCCable,
}

#[allow(dead_code)]
impl LineType {
    /// Returns `true` if the line type is an overhead line, `false` if it is an underground cable.
    pub fn is_overhead(&self) -> bool {
        match self {
            LineType::ACSRConductor | LineType::AACConductor | LineType::AAACConductor => true,
            LineType::XLPECable | LineType::PILCCable => false,
        }
    }

    /// Returns default values for resistance (Ω/km), inductance (mH/km), and capacitance (nF/km)
    /// for the line type.
    fn impedance_values(&self) -> (f64, f64, f64) {
        match self {
            // Values are illustrative examples; actual values can vary based on the specific line configuration, e.g. temperature
            LineType::ACSRConductor => (0.6082, 0.603, 0.80),
            LineType::AACConductor => (0.3104, 0.533, 9.0),
            LineType::AAACConductor => (0.6163, 0.603, 0.85),
            //cable info
            //https://www.gulfcable.com/Product_Subdetails?key=353
            LineType::XLPECable => (0.08, 0.7, 500.0),
            LineType::PILCCable => (0.1, 0.6, 300.0),
        }
    }
}

/// Transmission Line: Represents the transmission line that carries electrical power,
/// linking power sources with consumption areas.
#[derive(Clone, Debug, Copy)]
pub struct Transmission {
    /// The type of the transmission line
    line_type: LineType,

    /// Length of the transmission line in meters (m)
    length: f64,
}

impl Transmission {
    /// Constructs a new `Transmission` line with a specified type and length.
    ///
    /// # Parameters
    /// - `line_type`: The type of the transmission line, defined by `LineType`.
    /// - `length`: The length of the transmission line in kilometers.
    ///
    /// # Returns
    /// A new `Transmission` instance.
    pub fn new(line_type: LineType, length: f64) -> Self {
        Transmission { line_type, length }
    }

    /// Calculates the current flowing through the transmission line given the sending and receiving voltages.
    ///
    /// Formula: I = (Vi - Vj)/Z
    ///
    /// # Parameters
    /// - `v_sending`: The voltage at the sending end of the transmission line.
    /// - `v_receiving`: The voltage at the receiving end of the transmission line.
    /// - `z_base`: The base impedance value used for per-unit calculations.
    ///
    /// # Returns
    /// The current as a `Current` object, calculated using the difference in voltage divided by the impedance of the line.
    pub fn current(self, v_sending: Voltage, v_receiving: Voltage, z_base: f64) -> Current {
        Current::from_complex(
            (v_sending.to_complex() - v_receiving.to_complex())
                / self.impedance(z_base).to_complex(),
        )
    }

    /// Calculates the impedance of the transmission line.
    ///
    /// # Parameters
    /// - `z_base`: The base impedance value used for per-unit calculations.
    ///
    /// # Returns
    /// The impedance of the line as an `Impedance` object, scaled to per-unit system based on `z_base`.
    pub fn impedance(&self, z_base: f64) -> Impedance {
        //impedance =(resistance,reactance)
        let (_resistance_per_km, reactance_per_km, _) = self.line_type.impedance_values();
        //division by 1000 mHz->Hz
        let reactance = 2.0 * PI * reactance_per_km * self.length() * FREQUENCY / 1000.0;
        //set in p.u impedance
        Impedance::new(self.resistance() / z_base, reactance / z_base)
    }

    /// Returns the total resistance of the transmission line.
    ///
    /// The resistance is calculated based on the resistance component of the line type's impedance and the line's length.
    ///
    /// # Returns
    /// The total resistance of the line as a floating-point number.
    pub fn resistance(&self) -> f64 {
        self.line_type.impedance_values().0 * self.length
    }

    /// Returns the length of the transmission line.
    ///
    /// # Returns
    /// The length of the line in kilometers as a floating-point number.
    pub fn length(&self) -> f64 {
        self.length
    }

    /// Returns the type of the transmission line.
    ///
    /// # Returns
    /// The `LineType` enum indicating the type of the transmission line.
    pub fn line_type(&self) -> LineType {
        self.line_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_edge() {
        let l1 = Transmission::new(LineType::ACSRConductor, 200.0);
        assert_eq!(l1.length(), 200.0);
        assert!(l1.line_type.is_overhead());
        assert_eq!(l1.resistance(), 121.63999999999999);
        let l2 = Transmission::new(LineType::XLPECable, 200.0);
        let l3 = Transmission::new(LineType::PILCCable, 200.0);
        let l4 = Transmission::new(LineType::AACConductor, 200.0);
        assert!(!l2.line_type.is_overhead());
        assert!(!l3.line_type.is_overhead());
        assert!(l4.line_type.is_overhead());
    }
}
