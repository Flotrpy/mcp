//! The `overview` report: a compact first look plus suggested next calls.
//!
//! Instead of exposing dozens of narrow tools, binscope returns
//! `next_steps` — concrete follow-up calls chosen from what this particular
//! file contains. The model discovers depth only when it needs it.

use serde::Serialize;

use crate::{
    entropy::{self, Band},
    inspect::Binary,
};

/// Windows in the whole-file entropy sparkline.
pub const MAP_WIDTH: usize = 48;

#[derive(Debug, Clone, Serialize)]
pub struct SectionLine {
    pub name: String,
    pub size: u64,
    pub entropy: f64,
    pub band: Band,
    /// e.g. `r-x`.
    pub permissions: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Overview {
    pub format: String,
    pub architecture: String,
    pub bits: u8,
    pub endianness: &'static str,
    pub entry_point: String,
    pub file_size: u64,
    pub stripped: bool,
    pub counts: Counts,
    /// Entropy across the file, left to right, ▁ (0 bits) to █ (8 bits).
    pub entropy_map: String,
    /// Largest sections first.
    pub largest_sections: Vec<SectionLine>,
    pub notes: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Counts {
    pub sections: usize,
    pub libraries: usize,
    pub imports: usize,
    pub exports: usize,
}

fn perms(r: bool, w: bool, x: bool) -> String {
    [(r, 'r'), (w, 'w'), (x, 'x')]
        .iter()
        .map(|&(on, c)| if on { c } else { '-' })
        .collect()
}

pub fn overview(bin: &Binary, bytes: &[u8]) -> Overview {
    let window = bytes.len().div_ceil(MAP_WIDTH).max(1);
    let mut sections: Vec<_> = bin
        .sections
        .iter()
        .filter(|s| s.file_size > 0)
        .map(|s| SectionLine {
            name: s.name.clone(),
            size: s.file_size,
            entropy: (s.entropy * 100.0).round() / 100.0,
            band: entropy::band(s.entropy),
            permissions: perms(s.readable, s.writable, s.executable),
        })
        .collect();
    sections.sort_by_key(|s| std::cmp::Reverse(s.size));
    sections.truncate(8);

    let mut notes = Vec::new();
    for s in &bin.sections {
        if s.writable && s.executable {
            notes.push(format!("{} is writable and executable (W^X violation).", s.name));
        }
        if s.file_size >= 4096 && entropy::band(s.entropy) == Band::Packed {
            notes.push(format!(
                "{} ({} bytes) is near-random at {:.2} bits/byte: compressed or encrypted data.",
                s.name, s.file_size, s.entropy
            ));
        }
    }
    if bin.stripped {
        notes.push("Stripped: no local symbol table, only dynamic symbols remain.".into());
    }
    if !bin.universal_architectures.is_empty() {
        notes.push(format!(
            "Universal binary containing {}; describing the first.",
            bin.universal_architectures.join(", ")
        ));
    }

    let mut next_steps = Vec::new();
    if !bin.libraries.is_empty() || !bin.imports.is_empty() {
        next_steps.push(format!(
            "inspect view=imports — {} imported symbol(s) from {} librar(y/ies) show what the program relies on.",
            bin.imports.len(),
            bin.libraries.len()
        ));
    }
    if !bin.exports.is_empty() {
        next_steps.push(format!("inspect view=exports — {} exported symbol(s): the public API.", bin.exports.len()));
    }
    next_steps.push("inspect view=strings — URLs, paths, version banners and messages ranked first.".into());
    if let Some(packed) = bin.sections.iter().find(|s| entropy::band(s.entropy) == Band::Packed && s.file_size > 0) {
        next_steps.push(format!(
            "hexdump offset={} — peek at the start of high-entropy section {}.",
            packed.file_offset, packed.name
        ));
    }
    next_steps.push("diff old_path=… new_path=… — compare against another build of the same program.".into());

    Overview {
        format: format!("{:?}", bin.format),
        architecture: bin.architecture.clone(),
        bits: bin.bits,
        endianness: if bin.little_endian { "little" } else { "big" },
        entry_point: format!("{:#x}", bin.entry_point),
        file_size: bin.file_size,
        stripped: bin.stripped,
        counts: Counts {
            sections: bin.sections.len(),
            libraries: bin.libraries.len(),
            imports: bin.imports.len(),
            exports: bin.exports.len(),
        },
        entropy_map: entropy::sparkline(&entropy::windows(bytes, window)),
        largest_sections: sections,
        notes,
        next_steps,
    }
}

/// Classic 16-bytes-per-line hex dump with an ASCII gutter.
pub fn hexdump(bytes: &[u8], base_offset: u64) -> String {
    let mut out = String::new();
    for (i, chunk) in bytes.chunks(16).enumerate() {
        let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02x}")).collect();
        let ascii: String = chunk
            .iter()
            .map(|&b| if (0x20..0x7f).contains(&b) { char::from(b) } else { '.' })
            .collect();
        out.push_str(&format!("{:08x}  {:<47}  |{ascii}|\n", base_offset + (i as u64) * 16, hex.join(" ")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::parse;

    #[test]
    fn overview_of_this_test_binary_has_a_full_width_map_and_next_steps() {
        let bytes = std::fs::read(std::env::current_exe().unwrap()).unwrap();
        let bin = parse(&bytes).unwrap();
        let o = overview(&bin, &bytes);
        assert!(o.entropy_map.chars().count() <= MAP_WIDTH);
        assert!(o.entropy_map.chars().count() >= MAP_WIDTH - 1);
        assert!(o.next_steps.iter().any(|s| s.starts_with("inspect view=strings")));
        assert!(o.largest_sections.len() <= 8);
        assert!(o.entry_point.starts_with("0x"));
    }

    #[test]
    fn hexdump_formats_offsets_and_ascii() {
        let dump = hexdump(b"MZ\x90\x00hello", 0x40);
        assert!(dump.starts_with("00000040  4d 5a 90 00 68 65 6c 6c 6f"));
        assert!(dump.trim_end().ends_with("|MZ..hello|"));
    }
}
