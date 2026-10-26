//! One-line sparklines (▁▂▃▄▅▆▇█): a whole series for a handful of tokens.

const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Map each value onto eight bar heights between `min` and `max`.
/// A flat series (min == max) renders as mid-height bars.
pub fn scaled(values: &[f64], min: f64, max: f64) -> String {
    values
        .iter()
        .map(|v| {
            if max > min {
                BARS[(((v - min) / (max - min)) * 7.0).round().clamp(0.0, 7.0) as usize]
            } else {
                BARS[3]
            }
        })
        .collect()
}

/// Sparkline scaled to the series' own minimum and maximum.
pub fn auto(values: &[f64]) -> String {
    let (min, max) = values.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    scaled(values, min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_and_auto_ranges() {
        assert_eq!(scaled(&[0.0, 4.0, 8.0], 0.0, 8.0), "▁▅█");
        assert_eq!(scaled(&[-1.0, 99.0], 0.0, 8.0), "▁█", "out-of-range values clamp");
        assert_eq!(auto(&[10.0, 20.0]), "▁█");
        assert_eq!(auto(&[5.0, 5.0]), "▄▄");
        assert_eq!(auto(&[]), "");
    }
}
