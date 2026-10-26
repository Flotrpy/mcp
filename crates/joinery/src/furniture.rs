//! Parametric furniture generators.
//!
//! Each generator turns outside dimensions into a list of parts sized for the
//! chosen joinery, plus the joints between them (used to derive an assembly
//! order). All dimensions are millimetres.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::Part;

/// How two parts are joined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum JointKind {
    /// A housing groove across the grain.
    Dado,
    /// An L-shaped step along an edge.
    Rabbet,
    /// A groove along the grain, e.g. for a drawer bottom.
    Groove,
    /// Glue and screws or pocket screws, no housing.
    Butt,
    /// Loose part resting on shelf pins.
    ShelfPins,
}

/// A joint between two named parts. `host` is cut to receive `member`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Joint {
    pub kind: JointKind,
    pub host: String,
    pub member: String,
}

/// A complete design: parts plus how they go together.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Design {
    pub kind: String,
    pub parts: Vec<Part>,
    pub joints: Vec<Joint>,
    pub notes: Vec<String>,
}

/// Common carcass inputs.
#[derive(Debug, Clone, Copy)]
pub struct Carcass {
    pub width_mm: f64,
    pub height_mm: f64,
    pub depth_mm: f64,
    /// Thickness of the case stock.
    pub stock_mm: f64,
    /// Thickness of the back panel.
    pub back_mm: f64,
}

impl Carcass {
    /// Default dado depth: one third of the stock, the usual rule of thumb
    /// that keeps the side strong.
    pub fn dado_depth(&self) -> f64 {
        self.stock_mm / 3.0
    }

    /// Distance between the inside faces of the sides.
    pub fn inside_width(&self) -> f64 {
        self.width_mm - 2.0 * self.stock_mm
    }

    /// Depth of horizontal parts, leaving room for the back in a rabbet.
    pub fn shelf_depth(&self) -> f64 {
        self.depth_mm - self.back_mm
    }
}

/// Clearance taken off adjustable shelves so they drop in without binding.
pub const ADJUSTABLE_SHELF_CLEARANCE_MM: f64 = 2.0;

fn carcass_parts(c: &Carcass, kind: &str) -> Design {
    let dado = c.dado_depth();
    let horizontal_len = c.inside_width() + 2.0 * dado;
    let mut parts = vec![
        Part::new("side", c.height_mm, c.depth_mm, c.stock_mm, 2),
        Part::new("top", horizontal_len, c.shelf_depth(), c.stock_mm, 1),
        Part::new("bottom", horizontal_len, c.shelf_depth(), c.stock_mm, 1),
    ];
    // The back sits in rabbets in both sides, so it is wider than the inside.
    let back_width = c.inside_width() + 2.0 * (c.stock_mm / 2.0);
    let mut back = Part::new("back", c.height_mm, back_width, c.back_mm, 1);
    // Plywood backs have no meaningful grain direction for nesting.
    back.grain_locked = false;
    parts.push(back);

    let joints = vec![
        Joint { kind: JointKind::Dado, host: "side".into(), member: "top".into() },
        Joint { kind: JointKind::Dado, host: "side".into(), member: "bottom".into() },
        Joint { kind: JointKind::Rabbet, host: "side".into(), member: "back".into() },
    ];
    Design {
        kind: kind.to_string(),
        parts,
        joints,
        notes: vec![format!(
            "Dados are {dado:.1} mm deep (one third of the {:.1} mm stock); horizontal parts include that depth at both ends.",
            c.stock_mm
        )],
    }
}

/// A bookcase with `fixed` shelves in dados and `adjustable` shelves on pins.
pub fn bookcase(c: Carcass, fixed: u32, adjustable: u32) -> Design {
    let mut d = carcass_parts(&c, "bookcase");
    if fixed > 0 {
        let len = c.inside_width() + 2.0 * c.dado_depth();
        d.parts.push(Part::new("fixed shelf", len, c.shelf_depth(), c.stock_mm, fixed));
        d.joints.push(Joint { kind: JointKind::Dado, host: "side".into(), member: "fixed shelf".into() });
    }
    if adjustable > 0 {
        let len = c.inside_width() - ADJUSTABLE_SHELF_CLEARANCE_MM;
        d.parts.push(Part::new("adjustable shelf", len, c.shelf_depth(), c.stock_mm, adjustable));
        d.joints.push(Joint {
            kind: JointKind::ShelfPins,
            host: "side".into(),
            member: "adjustable shelf".into(),
        });
        d.notes.push(format!(
            "Adjustable shelves are {ADJUSTABLE_SHELF_CLEARANCE_MM} mm short of the inside width so they drop in without binding."
        ));
    }
    let openings = fixed + adjustable + 1;
    let clear =
        (c.height_mm - 2.0 * c.stock_mm - f64::from(fixed + adjustable) * c.stock_mm) / f64::from(openings);
    d.notes.push(format!("Evenly spaced, each opening is about {clear:.0} mm tall."));
    d
}

/// A plain cabinet carcass (no shelves).
pub fn cabinet(c: Carcass) -> Design {
    carcass_parts(&c, "cabinet")
}

/// A drawer box with a bottom captured in grooves.
pub fn drawer(width_mm: f64, height_mm: f64, depth_mm: f64, stock_mm: f64, bottom_mm: f64) -> Design {
    let groove_depth = stock_mm / 2.0;
    let front_back_len = width_mm - 2.0 * stock_mm + 2.0 * (stock_mm / 2.0);
    let mut bottom = Part::new(
        "bottom",
        depth_mm - 2.0 * stock_mm + 2.0 * groove_depth,
        width_mm - 2.0 * stock_mm + 2.0 * groove_depth,
        bottom_mm,
        1,
    );
    bottom.grain_locked = false;
    Design {
        kind: "drawer".into(),
        parts: vec![
            Part::new("drawer side", depth_mm, height_mm, stock_mm, 2),
            Part::new("drawer front/back", front_back_len, height_mm, stock_mm, 2),
            bottom,
        ],
        joints: vec![
            Joint { kind: JointKind::Rabbet, host: "drawer side".into(), member: "drawer front/back".into() },
            Joint { kind: JointKind::Groove, host: "drawer side".into(), member: "bottom".into() },
            Joint { kind: JointKind::Groove, host: "drawer front/back".into(), member: "bottom".into() },
        ],
        notes: vec![format!(
            "Front and back sit in half-depth rabbets in the sides; the bottom floats in {groove_depth:.1} mm grooves."
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case() -> Carcass {
        Carcass { width_mm: 900.0, height_mm: 1800.0, depth_mm: 300.0, stock_mm: 18.0, back_mm: 6.0 }
    }

    fn part<'a>(d: &'a Design, name: &str) -> &'a Part {
        d.parts.iter().find(|p| p.name == name).unwrap()
    }

    #[test]
    fn bookcase_horizontals_span_inside_plus_dados() {
        let d = bookcase(case(), 1, 3);
        // inside 864 + 2 × 6 mm dados
        assert!((part(&d, "top").length_mm - 876.0).abs() < 1e-9);
        assert!((part(&d, "fixed shelf").length_mm - 876.0).abs() < 1e-9);
        assert!((part(&d, "adjustable shelf").length_mm - 862.0).abs() < 1e-9);
        assert_eq!(part(&d, "adjustable shelf").quantity, 3);
        assert!((part(&d, "top").width_mm - 294.0).abs() < 1e-9);
    }

    #[test]
    fn back_is_not_grain_locked() {
        assert!(!part(&cabinet(case()), "back").grain_locked);
    }

    #[test]
    fn drawer_parts_close_the_box() {
        let d = drawer(400.0, 120.0, 450.0, 12.0, 6.0);
        // Front/back 400 - 24 + 12 = 388, so with two 12 mm sides minus two
        // 6 mm rabbets the outside width is 388 + 24 - 12 = 400.
        let fb = part(&d, "drawer front/back");
        assert!((fb.length_mm + 2.0 * 12.0 - 2.0 * 6.0 - 400.0).abs() < 1e-9);
        assert_eq!(d.joints.len(), 3);
    }
}
