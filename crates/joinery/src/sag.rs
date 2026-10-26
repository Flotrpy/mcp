//! Shelf sag (deflection) checks.
//!
//! A shelf resting on pins or dados at both ends is modelled as a simply
//! supported beam under a uniformly distributed load:
//!
//! ```text
//! δ = 5·W·L³ / (384·E·I),   I = b·t³ / 12
//! ```
//!
//! where `W` is the total load, `L` the span, `E` the species' modulus of
//! elasticity, `b` the shelf depth and `t` its thickness. The traditional
//! woodworking limit for sag that the eye will not notice is 1/32″ per foot of
//! span.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::{MM_PER_INCH, Species};

/// Pounds per kilogram.
pub const LB_PER_KG: f64 = 2.204_622_6;
/// Acceptable sag in inches per foot of span.
pub const LIMIT_IN_PER_FT: f64 = 1.0 / 32.0;

/// Typical loads, in pounds per foot of shelf length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LoadPreset {
    /// Hardcover and paperback books, packed tight (~20 lb/ft).
    Books,
    /// Stacked plates and bowls (~25 lb/ft).
    Dishes,
    /// Records / LPs on edge (~35 lb/ft).
    Records,
    /// Light decor and photo frames (~5 lb/ft).
    Decor,
}

impl LoadPreset {
    pub fn lb_per_ft(self) -> f64 {
        match self {
            LoadPreset::Books => 20.0,
            LoadPreset::Dishes => 25.0,
            LoadPreset::Records => 35.0,
            LoadPreset::Decor => 5.0,
        }
    }
}

/// Inputs to a sag check, all in millimetres and kilograms.
#[derive(Debug, Clone, Copy)]
pub struct Shelf {
    pub span_mm: f64,
    pub depth_mm: f64,
    pub thickness_mm: f64,
    pub load_kg: f64,
}

/// Result of a sag check.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SagReport {
    pub species: &'static str,
    pub deflection_mm: f64,
    /// Deflection after long-term creep under constant load.
    pub long_term_deflection_mm: f64,
    pub limit_mm: f64,
    pub passes: bool,
    /// Long-term deflection as a fraction of the limit (1.0 = right at it).
    pub utilisation: f64,
    /// The longest span that would pass with the same load density.
    pub max_span_mm: f64,
    /// The thinnest stock that would pass at this span and load.
    pub min_thickness_mm: f64,
    pub advice: Vec<String>,
}

fn immediate_deflection_in(
    span_in: f64,
    depth_in: f64,
    thickness_in: f64,
    load_lb: f64,
    moe_psi: f64,
) -> f64 {
    let moment_of_inertia = depth_in * thickness_in.powi(3) / 12.0;
    5.0 * load_lb * span_in.powi(3) / (384.0 * moe_psi * moment_of_inertia)
}

/// Check a shelf for sag.
pub fn check(shelf: Shelf, species: &'static Species) -> SagReport {
    let span_in = shelf.span_mm / MM_PER_INCH;
    let depth_in = shelf.depth_mm / MM_PER_INCH;
    let thickness_in = shelf.thickness_mm / MM_PER_INCH;
    let load_lb = shelf.load_kg * LB_PER_KG;

    let deflection_in = immediate_deflection_in(span_in, depth_in, thickness_in, load_lb, species.moe_psi);
    let long_term_in = deflection_in * species.creep_factor;
    let limit_in = LIMIT_IN_PER_FT * span_in / 12.0;
    let utilisation = long_term_in / limit_in;

    // Deflection/limit scales with L⁴/L = L³ when load density is constant,
    // so the passing span is span · (1/u)^(1/3).
    let max_span_in = span_in * utilisation.recip().cbrt();
    // Deflection scales with 1/t³, so the passing thickness is t · u^(1/3).
    let min_thickness_in = thickness_in * utilisation.cbrt();

    let passes = utilisation <= 1.0;
    let mut advice = Vec::new();
    if passes {
        if utilisation < 0.5 {
            advice.push("Comfortable margin; thinner stock or a longer span would still pass.".to_string());
        }
    } else {
        advice.push(format!(
            "Shorten the span to {:.0} mm or less, or add a centre support.",
            max_span_in * MM_PER_INCH
        ));
        advice.push(format!("Or use stock at least {:.1} mm thick.", min_thickness_in * MM_PER_INCH));
        advice.push(
            "Or glue a hardwood edge strip on edge under the front (it adds stiffness far faster than thickness)."
                .to_string(),
        );
        if species.sheet_good && species.moe_psi < 1_000_000.0 {
            advice.push(
                "MDF and particleboard creep badly; plywood or solid wood will hold up longer.".to_string(),
            );
        }
    }

    SagReport {
        species: species.id,
        deflection_mm: deflection_in * MM_PER_INCH,
        long_term_deflection_mm: long_term_in * MM_PER_INCH,
        limit_mm: limit_in * MM_PER_INCH,
        passes,
        utilisation,
        max_span_mm: max_span_in * MM_PER_INCH,
        min_thickness_mm: min_thickness_in * MM_PER_INCH,
        advice,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::species;

    fn shelf(span_in: f64, thickness_in: f64, load_lb: f64) -> Shelf {
        Shelf {
            span_mm: span_in * MM_PER_INCH,
            depth_mm: 10.0 * MM_PER_INCH,
            thickness_mm: thickness_in * MM_PER_INCH,
            load_kg: load_lb / LB_PER_KG,
        }
    }

    #[test]
    fn matches_hand_calculation() {
        // 36" x 10" x 0.75" red oak, 60 lb: I = 10·0.75³/12 = 0.3515625
        // δ = 5·60·36³ / (384·1.82e6·0.3515625) ≈ 0.0569"
        let r = check(shelf(36.0, 0.75, 60.0), species("red-oak").unwrap());
        assert!((r.deflection_mm / MM_PER_INCH - 0.0569).abs() < 0.0005, "{r:?}");
    }

    #[test]
    fn long_bookshelf_in_mdf_fails_and_suggests_fixes() {
        let r = check(shelf(36.0, 0.75, 60.0), species("mdf").unwrap());
        assert!(!r.passes);
        assert!(r.max_span_mm < 36.0 * MM_PER_INCH);
        assert!(r.min_thickness_mm > 0.75 * MM_PER_INCH);
        assert!(r.advice.iter().any(|a| a.contains("creep")));
    }

    #[test]
    fn short_oak_shelf_passes() {
        let r = check(shelf(24.0, 0.75, 20.0), species("red-oak").unwrap());
        assert!(r.passes, "{r:?}");
    }

    #[test]
    fn suggested_dimensions_land_exactly_on_the_limit() {
        let sp = species("poplar").unwrap();
        let base = shelf(40.0, 0.75, 70.0);
        let r = check(base, sp);
        let at_max_span = check(
            Shelf { span_mm: r.max_span_mm, load_kg: base.load_kg * r.max_span_mm / base.span_mm, ..base },
            sp,
        );
        assert!((at_max_span.utilisation - 1.0).abs() < 1e-6);
        let at_min_thickness = check(Shelf { thickness_mm: r.min_thickness_mm, ..base }, sp);
        assert!((at_min_thickness.utilisation - 1.0).abs() < 1e-6);
    }
}
