//! Compare two builds: what changed between v1 and v2 of a binary?

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::inspect::{Binary, Import};

/// Entropy changes smaller than this are noise between ordinary rebuilds.
pub const ENTROPY_DELTA: f64 = 0.5;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SectionChange {
    pub name: String,
    pub old_size: u64,
    pub new_size: u64,
    pub old_entropy: f64,
    pub new_entropy: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct Added<T> {
    pub added: Vec<T>,
    pub removed: Vec<T>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Diff {
    pub size_delta: i64,
    /// Human-readable headline changes, most important first.
    pub highlights: Vec<String>,
    pub sections: Added<String>,
    pub resized_sections: Vec<SectionChange>,
    pub libraries: Added<String>,
    pub imports: Added<String>,
    pub exports: Added<String>,
}

fn set_diff<T: Ord + Clone>(old: &BTreeSet<T>, new: &BTreeSet<T>) -> Added<T> {
    Added {
        added: new.difference(old).cloned().collect(),
        removed: old.difference(new).cloned().collect(),
    }
}

fn import_label(i: &Import) -> String {
    match &i.library {
        Some(lib) => format!("{lib}!{}", i.name),
        None => i.name.clone(),
    }
}

pub fn diff(old: &Binary, new: &Binary) -> Diff {
    let mut highlights = Vec::new();
    if old.format != new.format || old.architecture != new.architecture || old.bits != new.bits {
        highlights.push(format!(
            "Target changed: {:?} {} {}-bit → {:?} {} {}-bit.",
            old.format, old.architecture, old.bits, new.format, new.architecture, new.bits
        ));
    }
    if old.stripped != new.stripped {
        highlights.push(if new.stripped {
            "The new build is stripped (symbol table removed).".to_string()
        } else {
            "The new build ships a symbol table (it was stripped before).".to_string()
        });
    }

    let old_secs: BTreeMap<_, _> = old.sections.iter().map(|s| (s.name.clone(), s)).collect();
    let new_secs: BTreeMap<_, _> = new.sections.iter().map(|s| (s.name.clone(), s)).collect();
    let sections = set_diff(&old_secs.keys().cloned().collect(), &new_secs.keys().cloned().collect());
    let mut resized_sections: Vec<SectionChange> = old_secs
        .iter()
        .filter_map(|(name, o)| {
            let n = new_secs.get(name)?;
            let changed = o.file_size != n.file_size || (o.entropy - n.entropy).abs() >= ENTROPY_DELTA;
            changed.then(|| SectionChange {
                name: name.clone(),
                old_size: o.file_size,
                new_size: n.file_size,
                old_entropy: o.entropy,
                new_entropy: n.entropy,
            })
        })
        .collect();
    resized_sections.sort_by_key(|c| std::cmp::Reverse(c.new_size.abs_diff(c.old_size)));

    for c in &resized_sections {
        if c.new_entropy - c.old_entropy >= ENTROPY_DELTA && c.new_entropy > 7.5 {
            highlights.push(format!(
                "Section {} became near-random ({:.1} → {:.1} bits/byte): newly compressed or encrypted content.",
                c.name, c.old_entropy, c.new_entropy
            ));
        }
    }
    for name in &sections.added {
        if let Some(s) = new_secs.get(name)
            && s.writable
            && s.executable
        {
            highlights.push(format!("New section {name} is both writable and executable."));
        }
    }

    let libraries = set_diff(
        &old.libraries.iter().cloned().collect(),
        &new.libraries.iter().cloned().collect(),
    );
    if !libraries.added.is_empty() {
        highlights.push(format!("Links {} new librar(y/ies): {}.", libraries.added.len(), libraries.added.join(", ")));
    }
    let imports = set_diff(
        &old.imports.iter().map(import_label).collect(),
        &new.imports.iter().map(import_label).collect(),
    );
    let exports = set_diff(
        &old.exports.iter().cloned().collect(),
        &new.exports.iter().cloned().collect(),
    );
    if !exports.removed.is_empty() {
        highlights.push(format!(
            "{} exported symbol(s) removed — callers linking against them will break.",
            exports.removed.len()
        ));
    }

    Diff {
        size_delta: new.file_size as i64 - old.file_size as i64,
        highlights,
        sections,
        resized_sections,
        libraries,
        imports,
        exports,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::{Format, Section};

    fn sec(name: &str, size: u64, entropy: f64, w: bool, x: bool) -> Section {
        Section {
            name: name.into(),
            file_offset: 0,
            file_size: size,
            virtual_address: 0,
            readable: true,
            writable: w,
            executable: x,
            entropy,
        }
    }

    fn bin(sections: Vec<Section>, libs: &[&str], exports: &[&str]) -> Binary {
        Binary {
            format: Format::Elf,
            architecture: "X86_64".into(),
            bits: 64,
            little_endian: true,
            entry_point: 0,
            file_size: sections.iter().map(|s| s.file_size).sum(),
            stripped: true,
            libraries: libs.iter().map(|s| s.to_string()).collect(),
            sections,
            imports: vec![],
            exports: exports.iter().map(|s| s.to_string()).collect(),
            universal_architectures: vec![],
        }
    }

    #[test]
    fn reports_added_libraries_removed_exports_and_packed_sections() {
        let old = bin(vec![sec(".text", 1000, 6.0, false, true), sec(".data", 100, 3.0, true, false)], &["libc.so.6"], &["api_v1", "api_v2"]);
        let new = bin(
            vec![sec(".text", 1500, 6.1, false, true), sec(".data", 4000, 7.9, true, false), sec(".jit", 10, 1.0, true, true)],
            &["libc.so.6", "libcurl.so.4"],
            &["api_v2"],
        );
        let d = diff(&old, &new);
        assert_eq!(d.size_delta, 4410);
        assert_eq!(d.libraries.added, vec!["libcurl.so.4".to_string()]);
        assert_eq!(d.exports.removed, vec!["api_v1".to_string()]);
        assert_eq!(d.sections.added, vec![".jit".to_string()]);
        assert_eq!(d.resized_sections[0].name, ".data", "largest change first");
        let h = d.highlights.join("\n");
        assert!(h.contains("near-random"), "{h}");
        assert!(h.contains("writable and executable"), "{h}");
        assert!(h.contains("libcurl"), "{h}");
        assert!(h.contains("will break"), "{h}");
    }

    #[test]
    fn identical_builds_have_no_highlights() {
        let a = bin(vec![sec(".text", 10, 5.0, false, true)], &[], &[]);
        let d = diff(&a, &a.clone());
        assert!(d.highlights.is_empty());
        assert!(d.resized_sections.is_empty());
        assert_eq!(d.size_delta, 0);
    }
}
