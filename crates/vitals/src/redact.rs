//! Redaction for results that leave the warehouse.
//!
//! With `VITALS_REDACT=1` (or `redact: true` on a call) source and device
//! names are replaced with stable aliases ("Source A", "Source B" …) and only
//! calendar days — never times of day — are reported. Aliases are assigned in
//! sorted order of the real names, so they stay stable across calls for the
//! same data.

use std::collections::BTreeMap;

pub struct Redactor {
    aliases: BTreeMap<String, String>,
    pub enabled: bool,
}

fn letter_name(i: usize) -> String {
    let mut n = i;
    let mut s = String::new();
    loop {
        s.insert(0, char::from(b'A' + (n % 26) as u8));
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    format!("Source {s}")
}

impl Redactor {
    /// `known` is every source name in the warehouse.
    pub fn new(enabled: bool, known: impl IntoIterator<Item = String>) -> Self {
        let mut names: Vec<String> = known.into_iter().collect();
        names.sort();
        names.dedup();
        let aliases = names.into_iter().enumerate().map(|(i, n)| (n, letter_name(i))).collect();
        Self { aliases, enabled }
    }

    pub fn source(&self, name: &str) -> String {
        if !self.enabled {
            return name.to_string();
        }
        self.aliases.get(name).cloned().unwrap_or_else(|| "Source ?".into())
    }

    pub fn sources(&self, names: &[String]) -> Vec<String> {
        names.iter().map(|n| self.source(n)).collect()
    }
}

/// Whether the environment asks for redaction by default.
pub fn from_env() -> bool {
    matches!(std::env::var("VITALS_REDACT").as_deref(), Ok("1" | "true" | "yes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_are_stable_and_sorted() {
        let r = Redactor::new(
            true,
            ["Jo's iPhone".to_string(), "Jo's Apple Watch".to_string(), "Oura".to_string()],
        );
        assert_eq!(r.source("Jo's Apple Watch"), "Source A");
        assert_eq!(r.source("Jo's iPhone"), "Source B");
        assert_eq!(r.source("Oura"), "Source C");
        assert_eq!(r.source("unknown"), "Source ?");
    }

    #[test]
    fn disabled_passes_names_through_and_letters_extend_past_z() {
        let r = Redactor::new(false, ["x".to_string()]);
        assert_eq!(r.source("x"), "x");
        assert_eq!(letter_name(25), "Source Z");
        assert_eq!(letter_name(26), "Source AA");
    }
}
