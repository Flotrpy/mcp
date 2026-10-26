//! Derive a glue-up order from a design's joint graph.
//!
//! Rules a woodworker follows without thinking:
//! 1. Dry-fit everything first.
//! 2. Housed joints (dados, grooves, rabbets for case parts) go together
//!    before anything that closes the case, because a back panel or bottom
//!    locks the geometry.
//! 3. Backs go in last, after checking diagonals for square.
//! 4. Loose parts on shelf pins are installed after finishing.

use serde::Serialize;

use crate::furniture::{Design, JointKind};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Step {
    pub number: usize,
    pub instruction: String,
}

fn stage(kind: JointKind, member: &str) -> u8 {
    match kind {
        JointKind::Groove => 1,
        JointKind::Dado | JointKind::Butt => 2,
        JointKind::Rabbet if member == "back" => 4,
        JointKind::Rabbet => 2,
        JointKind::ShelfPins => 5,
    }
}

fn verb(kind: JointKind) -> &'static str {
    match kind {
        JointKind::Dado => "into the dados in",
        JointKind::Rabbet => "into the rabbets in",
        JointKind::Groove => "into the grooves in",
        JointKind::Butt => "against",
        JointKind::ShelfPins => "onto shelf pins in",
    }
}

/// Produce numbered assembly steps for `design`.
pub fn steps(design: &Design) -> Vec<Step> {
    let mut joints: Vec<_> = design.joints.iter().collect();
    joints.sort_by_key(|j| stage(j.kind, &j.member));

    let mut lines = vec![format!(
        "Dry-fit the whole {} without glue; label every part and mark faces.",
        design.kind
    )];
    let mut squared = false;
    for j in joints {
        let s = stage(j.kind, &j.member);
        if s >= 4 && !squared {
            lines.push("Check both diagonals match before the glue sets; clamp across the long diagonal to correct.".into());
            squared = true;
        }
        let glue = if j.kind == JointKind::ShelfPins { "" } else { "Glue and fit " };
        let what = if glue.is_empty() { "Set the " } else { "the " };
        lines.push(format!("{glue}{what}{} {} the {}.", j.member, verb(j.kind), j.host));
    }
    if !squared {
        lines.push("Check both diagonals match before the glue sets.".into());
    }
    lines.push("Let the glue cure, then sand and finish.".into());

    // Shelf pins come after finishing.
    let (pins, rest): (Vec<String>, Vec<String>) = lines.into_iter().partition(|l| l.contains("shelf pins"));
    rest.into_iter()
        .chain(pins)
        .enumerate()
        .map(|(i, instruction)| Step { number: i + 1, instruction })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::furniture::{Carcass, bookcase, drawer};

    fn index_of(steps: &[Step], needle: &str) -> usize {
        steps.iter().position(|s| s.instruction.contains(needle)).unwrap()
    }

    #[test]
    fn bookcase_back_goes_in_after_squaring_and_pins_last() {
        let c = Carcass { width_mm: 900.0, height_mm: 1800.0, depth_mm: 300.0, stock_mm: 18.0, back_mm: 6.0 };
        let s = steps(&bookcase(c, 1, 2));
        assert!(s[0].instruction.starts_with("Dry-fit"));
        assert!(index_of(&s, "the top") < index_of(&s, "diagonals"));
        assert!(index_of(&s, "diagonals") < index_of(&s, "the back"));
        assert!(index_of(&s, "the back") < index_of(&s, "finish"));
        assert_eq!(index_of(&s, "shelf pins"), s.len() - 1);
        assert!(s.iter().enumerate().all(|(i, st)| st.number == i + 1));
    }

    #[test]
    fn drawer_bottom_goes_in_before_the_box_closes() {
        let s = steps(&drawer(400.0, 120.0, 450.0, 12.0, 6.0));
        assert!(index_of(&s, "bottom") < index_of(&s, "drawer front/back into the rabbets"));
    }
}
