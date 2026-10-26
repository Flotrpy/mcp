//! Core types: lengths, wood species and parts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Millimetres per inch.
pub const MM_PER_INCH: f64 = 25.4;

/// Unit system used for input and output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    #[default]
    Mm,
    In,
}

impl Units {
    /// Convert a value in these units to millimetres.
    pub fn to_mm(self, value: f64) -> f64 {
        match self {
            Units::Mm => value,
            Units::In => value * MM_PER_INCH,
        }
    }

    /// Convert millimetres to these units.
    pub fn from_mm(self, mm: f64) -> f64 {
        match self {
            Units::Mm => mm,
            Units::In => mm / MM_PER_INCH,
        }
    }
}

/// A wood species or sheet good with the stiffness needed for sag checks.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Species {
    pub id: &'static str,
    pub name: &'static str,
    /// Modulus of elasticity in psi (US Forest Products Lab, Wood Handbook,
    /// 12% moisture content; sheet goods are typical manufacturer figures).
    pub moe_psi: f64,
    /// Multiplier applied to immediate deflection to approximate long-term
    /// creep under a constant load.
    pub creep_factor: f64,
    pub sheet_good: bool,
}

/// Species known to the server. Values are approximate and intended for
/// furniture-scale decisions, not engineering sign-off.
pub const SPECIES: &[Species] = &[
    Species { id: "red-oak", name: "Red oak", moe_psi: 1_820_000.0, creep_factor: 1.5, sheet_good: false },
    Species { id: "white-oak", name: "White oak", moe_psi: 1_780_000.0, creep_factor: 1.5, sheet_good: false },
    Species { id: "hard-maple", name: "Hard maple", moe_psi: 1_830_000.0, creep_factor: 1.5, sheet_good: false },
    Species { id: "cherry", name: "Black cherry", moe_psi: 1_490_000.0, creep_factor: 1.5, sheet_good: false },
    Species { id: "walnut", name: "Black walnut", moe_psi: 1_680_000.0, creep_factor: 1.5, sheet_good: false },
    Species { id: "poplar", name: "Yellow poplar", moe_psi: 1_580_000.0, creep_factor: 1.5, sheet_good: false },
    Species { id: "white-pine", name: "Eastern white pine", moe_psi: 1_240_000.0, creep_factor: 1.5, sheet_good: false },
    Species { id: "douglas-fir", name: "Douglas fir", moe_psi: 1_950_000.0, creep_factor: 1.5, sheet_good: false },
    Species { id: "birch-ply", name: "Birch plywood", moe_psi: 1_200_000.0, creep_factor: 1.75, sheet_good: true },
    Species { id: "mdf", name: "MDF", moe_psi: 400_000.0, creep_factor: 2.0, sheet_good: true },
    Species { id: "particleboard", name: "Particleboard", moe_psi: 300_000.0, creep_factor: 2.0, sheet_good: true },
];

/// Look a species up by id (case-insensitive), accepting spaces for dashes.
pub fn species(id: &str) -> Option<&'static Species> {
    let wanted = id.trim().to_ascii_lowercase().replace([' ', '_'], "-");
    SPECIES.iter().find(|s| s.id == wanted)
}

/// One rectangular part, sized in millimetres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Part {
    pub name: String,
    /// Along the grain.
    pub length_mm: f64,
    pub width_mm: f64,
    pub thickness_mm: f64,
    pub quantity: u32,
    /// If true the part must keep its grain running along the sheet's length
    /// during nesting (it may not be rotated).
    #[serde(default = "default_true")]
    pub grain_locked: bool,
}

fn default_true() -> bool {
    true
}

impl Part {
    pub fn new(name: impl Into<String>, length_mm: f64, width_mm: f64, thickness_mm: f64, quantity: u32) -> Self {
        Self {
            name: name.into(),
            length_mm,
            width_mm,
            thickness_mm,
            quantity,
            grain_locked: true,
        }
    }

    pub fn area_mm2(&self) -> f64 {
        self.length_mm * self.width_mm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_inches_round_trip() {
        let mm = Units::In.to_mm(12.0);
        assert!((mm - 304.8).abs() < 1e-9);
        assert!((Units::In.from_mm(mm) - 12.0).abs() < 1e-9);
    }

    #[test]
    fn species_lookup_is_forgiving() {
        assert_eq!(species("Red Oak").unwrap().id, "red-oak");
        assert_eq!(species("birch_ply").unwrap().id, "birch-ply");
        assert!(species("unobtainium").is_none());
    }
}
