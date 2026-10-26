//! Shannon entropy over bytes.
//!
//! Plain code and data usually sit between 4 and 6.5 bits per byte;
//! compressed or encrypted payloads approach 8. A section that is both large
//! and near 8 is worth a look in release review ("why is 3 MB of this binary
//! incompressible?").

use serde::Serialize;

/// Bits of entropy per byte, 0.0 (all one value) to 8.0 (uniform).
pub fn shannon(bytes: &[u8]) -> f64 {
    if bytes.is_empty() {
        return 0.0;
    }
    let mut counts = [0u64; 256];
    for &b in bytes {
        counts[usize::from(b)] += 1;
    }
    let len = bytes.len() as f64;
    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

/// A coarse label for an entropy value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Band {
    /// Below 1: padding, zero-filled tables.
    Sparse,
    /// 1–6.5: typical code, text and structured data.
    Typical,
    /// 6.5–7.5: dense data such as images or resources.
    Dense,
    /// Above 7.5: compressed or encrypted.
    Packed,
}

pub fn band(bits: f64) -> Band {
    match bits {
        b if b < 1.0 => Band::Sparse,
        b if b < 6.5 => Band::Typical,
        b if b < 7.5 => Band::Dense,
        _ => Band::Packed,
    }
}

/// Entropy of consecutive fixed-size windows across `bytes`, for a
/// whole-file map. The last window may be shorter.
pub fn windows(bytes: &[u8], window: usize) -> Vec<f64> {
    if window == 0 {
        return Vec::new();
    }
    bytes.chunks(window).map(shannon).collect()
}

/// Render entropy values as a one-line sparkline (▁ … █), cheap in tokens.
pub fn sparkline(values: &[f64]) -> String {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    values
        .iter()
        .map(|v| BARS[((v / 8.0) * 7.0).round().clamp(0.0, 7.0) as usize])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_bytes_have_zero_entropy() {
        assert_eq!(shannon(&[7; 1000]), 0.0);
        assert_eq!(shannon(&[]), 0.0);
    }

    #[test]
    fn every_byte_once_is_eight_bits() {
        let all: Vec<u8> = (0..=255).collect();
        assert!((shannon(&all) - 8.0).abs() < 1e-12);
    }

    #[test]
    fn two_symbols_evenly_is_one_bit() {
        assert!((shannon(b"abababab") - 1.0).abs() < 1e-12);
    }

    #[test]
    fn bands_and_sparkline() {
        assert_eq!(band(0.2), Band::Sparse);
        assert_eq!(band(5.0), Band::Typical);
        assert_eq!(band(7.0), Band::Dense);
        assert_eq!(band(7.9), Band::Packed);
        assert_eq!(sparkline(&[0.0, 8.0]), "▁█");
        assert_eq!(windows(&[0; 10], 4).len(), 3);
    }
}
