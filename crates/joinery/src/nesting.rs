//! Guillotine sheet nesting with saw kerf and grain direction.
//!
//! Guillotine layouts are the ones you can actually cut on a table saw or
//! track saw: every cut runs edge to edge of the current piece. Parts are
//! placed largest first into the free rectangle that wastes the least area;
//! after each placement the leftover space is split along whichever axis keeps
//! the larger offcut intact.

use serde::Serialize;

use crate::model::Part;

/// A stock sheet, e.g. 2440 × 1220 mm plywood. `length_mm` runs with the
/// face grain.
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Sheet {
    pub length_mm: f64,
    pub width_mm: f64,
}

/// A part instance placed on a sheet. Coordinates are the top-left corner,
/// `x` along the sheet length.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Placement {
    pub sheet: usize,
    pub name: String,
    pub x_mm: f64,
    pub y_mm: f64,
    pub length_mm: f64,
    pub width_mm: f64,
    /// True if the part was turned 90° (only ever for grain-free parts).
    pub rotated: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Layout {
    pub sheet: Sheet,
    pub kerf_mm: f64,
    pub sheets_used: usize,
    pub placements: Vec<Placement>,
    /// Parts larger than a whole sheet in every allowed orientation.
    pub unplaced: Vec<String>,
    /// Part area over total area of the sheets used, 0–100.
    pub yield_percent: f64,
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    l: f64,
    w: f64,
}

impl Rect {
    fn area(&self) -> f64 {
        self.l * self.w
    }
}

struct Instance<'a> {
    part: &'a Part,
}

/// Nest `parts` onto as few `sheet`s as this greedy heuristic manages.
pub fn nest(parts: &[Part], sheet: Sheet, kerf_mm: f64) -> Layout {
    let mut instances: Vec<Instance> = parts
        .iter()
        .flat_map(|p| (0..p.quantity).map(move |_| Instance { part: p }))
        .collect();
    instances.sort_by(|a, b| {
        let ka = a.part.length_mm.max(a.part.width_mm);
        let kb = b.part.length_mm.max(b.part.width_mm);
        kb.total_cmp(&ka).then(b.part.area_mm2().total_cmp(&a.part.area_mm2()))
    });

    // Free rectangles per sheet. Each part reserves its size plus one kerf on
    // the right and bottom, so the sheet is treated as one kerf larger to let
    // a part run flush to the far edge.
    let mut free: Vec<Vec<Rect>> = Vec::new();
    let mut placements = Vec::new();
    let mut unplaced = Vec::new();
    let full = Rect { x: 0.0, y: 0.0, l: sheet.length_mm + kerf_mm, w: sheet.width_mm + kerf_mm };

    for inst in instances {
        let p = inst.part;
        let mut orientations = vec![(p.length_mm, p.width_mm, false)];
        if !p.grain_locked && (p.length_mm - p.width_mm).abs() > f64::EPSILON {
            orientations.push((p.width_mm, p.length_mm, true));
        }

        let fits_sheet = orientations
            .iter()
            .any(|&(l, w, _)| l + kerf_mm <= full.l + 1e-9 && w + kerf_mm <= full.w + 1e-9);
        if !fits_sheet {
            unplaced.push(p.name.clone());
            continue;
        }

        let best = best_spot(&free, &orientations, kerf_mm);
        let (sheet_idx, rect_idx, (l, w, rotated)) = match best {
            Some(found) => found,
            None => {
                free.push(vec![full]);
                let idx = free.len() - 1;
                best_spot(&free[idx..], &orientations, kerf_mm)
                    .map(|(_, r, o)| (idx, r, o))
                    .expect("part fits an empty sheet")
            }
        };

        let rect = free[sheet_idx].swap_remove(rect_idx);
        placements.push(Placement {
            sheet: sheet_idx,
            name: p.name.clone(),
            x_mm: rect.x,
            y_mm: rect.y,
            length_mm: l,
            width_mm: w,
            rotated,
        });
        free[sheet_idx].extend(split(rect, l + kerf_mm, w + kerf_mm));
    }

    let used_area: f64 = placements.iter().map(|p| p.length_mm * p.width_mm).sum();
    let sheets_used = free.len();
    let yield_percent = if sheets_used == 0 {
        0.0
    } else {
        100.0 * used_area / (sheets_used as f64 * sheet.length_mm * sheet.width_mm)
    };

    Layout { sheet, kerf_mm, sheets_used, placements, unplaced, yield_percent }
}

/// Best-area-fit search across every free rectangle of every sheet.
fn best_spot(
    free: &[Vec<Rect>],
    orientations: &[(f64, f64, bool)],
    kerf: f64,
) -> Option<(usize, usize, (f64, f64, bool))> {
    let mut best: Option<(f64, usize, usize, (f64, f64, bool))> = None;
    for (si, rects) in free.iter().enumerate() {
        for (ri, r) in rects.iter().enumerate() {
            for &o in orientations {
                let (l, w, _) = o;
                if l + kerf <= r.l + 1e-9 && w + kerf <= r.w + 1e-9 {
                    let waste = r.area() - (l + kerf) * (w + kerf);
                    if best.as_ref().is_none_or(|b| waste < b.0) {
                        best = Some((waste, si, ri, o));
                    }
                }
            }
        }
    }
    best.map(|(_, s, r, o)| (s, r, o))
}

/// Split the space left after placing an `l × w` block in the top-left of
/// `r`, keeping the larger of the two possible offcuts whole.
fn split(r: Rect, l: f64, w: f64) -> Vec<Rect> {
    let right_full = Rect { x: r.x + l, y: r.y, l: r.l - l, w: r.w };
    let below_short = Rect { x: r.x, y: r.y + w, l, w: r.w - w };
    let right_short = Rect { x: r.x + l, y: r.y, l: r.l - l, w };
    let below_full = Rect { x: r.x, y: r.y + w, l: r.l, w: r.w - w };

    let vertical_cut = right_full.area().max(below_short.area());
    let horizontal_cut = right_short.area().max(below_full.area());
    let (a, b) = if vertical_cut >= horizontal_cut {
        (right_full, below_short)
    } else {
        (right_short, below_full)
    };
    [a, b].into_iter().filter(|x| x.l > 1e-6 && x.w > 1e-6).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLY: Sheet = Sheet { length_mm: 2440.0, width_mm: 1220.0 };

    fn overlaps(a: &Placement, b: &Placement) -> bool {
        a.sheet == b.sheet
            && a.x_mm < b.x_mm + b.length_mm
            && b.x_mm < a.x_mm + a.length_mm
            && a.y_mm < b.y_mm + b.width_mm
            && b.y_mm < a.y_mm + a.width_mm
    }

    #[test]
    fn four_quarters_fill_one_sheet() {
        let q = Part::new("quarter", 1220.0, 610.0, 18.0, 4);
        let layout = nest(&[q], PLY, 0.0);
        assert_eq!(layout.sheets_used, 1);
        assert!((layout.yield_percent - 100.0).abs() < 1e-9);
    }

    #[test]
    fn kerf_means_exact_quarters_no_longer_share_a_sheet() {
        // 1220 + 3 mm kerf + 1220 = 2443 mm > 2440 mm, and likewise across the
        // width, so every exact quarter needs its own sheet once kerf counts.
        let q = Part::new("quarter", 1220.0, 610.0, 18.0, 4);
        let layout = nest(&[q], PLY, 3.0);
        assert_eq!(layout.sheets_used, 4);

        // Trimmed by a kerf, they fit four to a sheet again.
        let trimmed = Part::new("quarter", 1218.5, 608.5, 18.0, 4);
        assert_eq!(nest(&[trimmed], PLY, 3.0).sheets_used, 1);
    }

    #[test]
    fn placements_never_overlap_and_stay_on_the_sheet() {
        let parts = vec![
            Part::new("side", 1800.0, 300.0, 18.0, 2),
            Part::new("shelf", 862.0, 294.0, 18.0, 5),
            Part::new("top", 876.0, 294.0, 18.0, 2),
        ];
        let layout = nest(&parts, PLY, 3.2);
        assert!(layout.unplaced.is_empty());
        assert_eq!(layout.placements.len(), 9);
        for (i, a) in layout.placements.iter().enumerate() {
            assert!(a.x_mm + a.length_mm <= PLY.length_mm + 1e-6, "{a:?}");
            assert!(a.y_mm + a.width_mm <= PLY.width_mm + 1e-6, "{a:?}");
            for b in &layout.placements[i + 1..] {
                assert!(!overlaps(a, b), "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn grain_locked_parts_are_never_rotated() {
        let tall = Part::new("tall", 1000.0, 1200.0, 18.0, 1);
        let layout = nest(&[tall], PLY, 0.0);
        assert!(!layout.placements[0].rotated);

        let mut free = Part::new("free", 1300.0, 1000.0, 6.0, 1);
        free.grain_locked = false;
        let narrow = Sheet { length_mm: 1100.0, width_mm: 1400.0 };
        let layout = nest(&[free], narrow, 0.0);
        assert!(layout.placements[0].rotated);
    }

    #[test]
    fn oversize_parts_are_reported_not_placed() {
        let huge = Part::new("huge", 3000.0, 500.0, 18.0, 1);
        let layout = nest(&[huge], PLY, 3.0);
        assert_eq!(layout.unplaced, vec!["huge".to_string()]);
        assert_eq!(layout.sheets_used, 0);
    }
}
