//! Render a nesting layout as a standalone SVG cut diagram.

use std::fmt::Write;

use crate::nesting::Layout;

const GAP_MM: f64 = 80.0;
const LABEL_MIN_MM: f64 = 120.0;

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// One SVG with every sheet stacked vertically. Units are millimetres, so the
/// `viewBox` matches real dimensions and the drawing prints to scale.
pub fn render(layout: &Layout) -> String {
    let sheet = layout.sheet;
    let sheets = layout.sheets_used.max(1) as f64;
    let height = sheets * sheet.width_mm + (sheets - 1.0) * GAP_MM;
    let mut out = String::new();
    let _ = write!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="-10 -10 {w} {h}" font-family="system-ui, sans-serif">"#,
        w = sheet.length_mm + 20.0,
        h = height + 20.0
    );
    for s in 0..layout.sheets_used {
        let top = s as f64 * (sheet.width_mm + GAP_MM);
        let _ = write!(
            out,
            r##"<rect x="0" y="{top}" width="{l}" height="{w}" fill="#f5f1ea" stroke="#8a7a66" stroke-width="2"/>"##,
            l = sheet.length_mm,
            w = sheet.width_mm
        );
        for p in layout.placements.iter().filter(|p| p.sheet == s) {
            let y = top + p.y_mm;
            let _ = write!(
                out,
                r##"<rect x="{x}" y="{y}" width="{l}" height="{w}" fill="#d9c3a0" stroke="#5c4a36" stroke-width="1.5"/>"##,
                x = p.x_mm,
                l = p.length_mm,
                w = p.width_mm
            );
            if p.length_mm >= LABEL_MIN_MM && p.width_mm >= 40.0 {
                let size = (p.width_mm / 4.0).clamp(14.0, 48.0);
                let _ = write!(
                    out,
                    r##"<text x="{cx}" y="{cy}" font-size="{size}" text-anchor="middle" dominant-baseline="middle" fill="#2e2418">{name} {l:.0}×{w:.0}</text>"##,
                    cx = p.x_mm + p.length_mm / 2.0,
                    cy = y + p.width_mm / 2.0,
                    name = escape(&p.name),
                    l = p.length_mm,
                    w = p.width_mm
                );
            }
        }
    }
    out.push_str("</svg>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Part;
    use crate::nesting::{Sheet, nest};

    #[test]
    fn renders_one_rect_per_sheet_and_part_and_escapes_names() {
        let parts = vec![Part::new("left & right <side>", 1800.0, 300.0, 18.0, 2)];
        let layout = nest(&parts, Sheet { length_mm: 2440.0, width_mm: 1220.0 }, 3.0);
        let svg = render(&layout);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert_eq!(svg.matches("<rect").count(), 1 + 2);
        assert!(svg.contains("left &amp; right &lt;side&gt;"));
    }
}
