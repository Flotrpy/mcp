//! Cut lists: grouped parts with material totals.

use serde::Serialize;

use crate::model::{MM_PER_INCH, Part};

/// One line of a cut list.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CutLine {
    pub name: String,
    pub quantity: u32,
    pub length: f64,
    pub width: f64,
    pub thickness: f64,
}

/// Totals for one stock thickness.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StockTotal {
    pub thickness_mm: f64,
    pub area_m2: f64,
    /// Board feet (1 ft × 1 ft × 1 in) for buying rough lumber.
    pub board_feet: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CutList {
    pub units: &'static str,
    pub lines: Vec<CutLine>,
    pub totals: Vec<StockTotal>,
    pub piece_count: u32,
}

/// Build a cut list. Identical parts (same name and size) are merged; lines
/// are ordered thickest stock first, then longest, which is the order you
/// would break down boards in.
pub fn build(parts: &[Part], inches: bool) -> CutList {
    let mut merged: Vec<Part> = Vec::new();
    for p in parts {
        if let Some(existing) = merged.iter_mut().find(|m| {
            m.name == p.name
                && (m.length_mm - p.length_mm).abs() < 0.05
                && (m.width_mm - p.width_mm).abs() < 0.05
                && (m.thickness_mm - p.thickness_mm).abs() < 0.05
        }) {
            existing.quantity += p.quantity;
        } else {
            merged.push(p.clone());
        }
    }
    merged.sort_by(|a, b| {
        b.thickness_mm
            .total_cmp(&a.thickness_mm)
            .then(b.length_mm.total_cmp(&a.length_mm))
    });

    let conv = |mm: f64| if inches { round(mm / MM_PER_INCH, 3) } else { round(mm, 1) };
    let lines = merged
        .iter()
        .map(|p| CutLine {
            name: p.name.clone(),
            quantity: p.quantity,
            length: conv(p.length_mm),
            width: conv(p.width_mm),
            thickness: conv(p.thickness_mm),
        })
        .collect();

    let mut totals: Vec<StockTotal> = Vec::new();
    for p in &merged {
        let area_mm2 = p.area_mm2() * f64::from(p.quantity);
        let volume_in3 = area_mm2 * p.thickness_mm / MM_PER_INCH.powi(3);
        match totals.iter_mut().find(|t| (t.thickness_mm - p.thickness_mm).abs() < 0.05) {
            Some(t) => {
                t.area_m2 += area_mm2 / 1e6;
                t.board_feet += volume_in3 / 144.0;
            }
            None => totals.push(StockTotal {
                thickness_mm: p.thickness_mm,
                area_m2: area_mm2 / 1e6,
                board_feet: volume_in3 / 144.0,
            }),
        }
    }
    for t in &mut totals {
        t.area_m2 = round(t.area_m2, 3);
        t.board_feet = round(t.board_feet, 2);
    }

    CutList {
        units: if inches { "in" } else { "mm" },
        piece_count: merged.iter().map(|p| p.quantity).sum(),
        lines,
        totals,
    }
}

fn round(v: f64, places: i32) -> f64 {
    let f = 10f64.powi(places);
    (v * f).round() / f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_identical_parts_and_orders_thickest_first() {
        let parts = vec![
            Part::new("back", 1000.0, 500.0, 6.0, 1),
            Part::new("shelf", 800.0, 300.0, 18.0, 2),
            Part::new("shelf", 800.0, 300.0, 18.0, 1),
        ];
        let cl = build(&parts, false);
        assert_eq!(cl.lines.len(), 2);
        assert_eq!(cl.lines[0].name, "shelf");
        assert_eq!(cl.lines[0].quantity, 3);
        assert_eq!(cl.piece_count, 4);
    }

    #[test]
    fn board_feet_of_one_board_foot() {
        let one_bf = Part::new("test", 12.0 * MM_PER_INCH, 12.0 * MM_PER_INCH, MM_PER_INCH, 1);
        let cl = build(&[one_bf], true);
        assert_eq!(cl.totals[0].board_feet, 1.0);
        assert_eq!(cl.lines[0].length, 12.0);
    }
}
