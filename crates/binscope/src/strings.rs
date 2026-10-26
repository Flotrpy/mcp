//! Embedded string extraction, ranked so the informative ones come first.
//!
//! Raw `strings` output on a release binary runs to tens of thousands of
//! lines, most of them noise. binscope classifies each run and sorts by
//! category, so a small token budget still surfaces URLs, file paths, version
//! banners and error messages.

use serde::Serialize;

use crate::inspect::Section;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Url,
    Path,
    Version,
    Message,
    FormatString,
    Identifier,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Encoding {
    Ascii,
    Utf16le,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Found {
    pub offset: u64,
    pub encoding: Encoding,
    pub category: Category,
    pub section: Option<String>,
    pub text: String,
}

/// Longest text returned for one string; longer runs are cut with `…`.
pub const MAX_TEXT: usize = 160;

fn printable(b: u8) -> bool {
    b == b'\t' || (0x20..0x7f).contains(&b)
}

/// Classify a string by what it most likely is.
pub fn classify(s: &str) -> Category {
    let lower = s.to_ascii_lowercase();
    if lower.contains("://") {
        return Category::Url;
    }
    let looks_like_path = (s.starts_with('/') && s.len() > 1 && s[1..].contains('/'))
        || s.contains(":\\")
        || s.starts_with("\\\\");
    if looks_like_path && !s.contains(' ') {
        return Category::Path;
    }
    let has_version_number = s
        .split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .any(|t| t.matches('.').count() >= 1 && t.split('.').filter(|p| !p.is_empty()).count() >= 2);
    if has_version_number && (lower.contains("version") || lower.contains(" v") || lower.starts_with('v') || lower.contains("build")) {
        return Category::Version;
    }
    if s.contains('%') && s.chars().any(|c| "sdxufplc".contains(c)) && s.matches('%').count() <= 8 {
        return Category::FormatString;
    }
    let words = s.split_whitespace().count();
    if words >= 3 && s.chars().filter(|c| c.is_ascii_alphabetic()).count() * 2 > s.len() {
        return Category::Message;
    }
    if words == 1 && s.chars().all(|c| c.is_ascii_alphanumeric() || "_:.$@".contains(c)) {
        return Category::Identifier;
    }
    Category::Other
}

fn section_for(sections: &[Section], offset: u64) -> Option<String> {
    sections
        .iter()
        .find(|s| s.file_size > 0 && offset >= s.file_offset && offset < s.file_offset + s.file_size)
        .map(|s| s.name.clone())
}

fn push(out: &mut Vec<Found>, sections: &[Section], offset: usize, encoding: Encoding, mut text: String) {
    if text.chars().count() > MAX_TEXT {
        text = text.chars().take(MAX_TEXT - 1).collect::<String>() + "…";
    }
    out.push(Found {
        offset: offset as u64,
        encoding,
        category: classify(&text),
        section: section_for(sections, offset as u64),
        text,
    });
}

/// Extract printable ASCII and UTF-16LE runs of at least `min_len`
/// characters, ranked by category then length (longest first).
pub fn extract(bytes: &[u8], sections: &[Section], min_len: usize) -> Vec<Found> {
    let min_len = min_len.max(2);
    let mut out = Vec::new();

    let mut start = 0;
    for (i, &b) in bytes.iter().chain(std::iter::once(&0)).enumerate() {
        if !(i < bytes.len() && printable(b)) {
            if i - start >= min_len {
                push(&mut out, sections, start, Encoding::Ascii, String::from_utf8_lossy(&bytes[start..i]).into_owned());
            }
            start = i + 1;
        }
    }

    // UTF-16LE: printable ASCII byte followed by 0x00, repeated.
    let mut i = 0;
    while i + 1 < bytes.len() {
        let run_start = i;
        let mut chars = String::new();
        while i + 1 < bytes.len() && printable(bytes[i]) && bytes[i + 1] == 0 {
            chars.push(char::from(bytes[i]));
            i += 2;
        }
        if chars.len() >= min_len {
            push(&mut out, sections, run_start, Encoding::Utf16le, chars);
        }
        if i == run_start {
            i += 1;
        }
    }

    out.sort_by(|a, b| a.category.cmp(&b.category).then(b.text.len().cmp(&a.text.len())).then(a.offset.cmp(&b.offset)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_shapes() {
        assert_eq!(classify("https://example.com/update"), Category::Url);
        assert_eq!(classify("/usr/lib/libssl.so.3"), Category::Path);
        assert_eq!(classify("C:\\Program Files\\App"), Category::Other); // has a space
        assert_eq!(classify("C:\\Windows\\System32"), Category::Path);
        assert_eq!(classify("myapp version 2.4.1"), Category::Version);
        assert_eq!(classify("failed to open %s: %d"), Category::FormatString);
        assert_eq!(classify("could not read the configuration file"), Category::Message);
        assert_eq!(classify("_ZN4core3fmt5write"), Category::Identifier);
    }

    #[test]
    fn extracts_ascii_and_utf16_and_ranks_urls_first() {
        // The extra NUL keeps "x\0" from being read as the start of the wide run.
        let mut bytes = b"\x00\x01noise_identifier\x00\x02https://example.org/x\x00\x00".to_vec();
        for c in "Wide Message Here".bytes() {
            bytes.push(c);
            bytes.push(0);
        }
        let found = extract(&bytes, &[], 6);
        assert_eq!(found[0].category, Category::Url);
        assert!(found.iter().any(|f| f.encoding == Encoding::Utf16le && f.text == "Wide Message Here"));
        assert!(found.iter().any(|f| f.text == "noise_identifier"));
    }

    #[test]
    fn long_runs_are_truncated_with_an_ellipsis() {
        let bytes = vec![b'a'; 500];
        let found = extract(&bytes, &[], 4);
        assert_eq!(found[0].text.chars().count(), MAX_TEXT);
        assert!(found[0].text.ends_with('…'));
    }
}
