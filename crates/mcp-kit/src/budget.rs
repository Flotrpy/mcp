//! Token budgets.
//!
//! Tools in this workspace accept an optional `max_tokens` argument. Output is
//! trimmed *structurally* (whole records are dropped from the tail of a
//! ranked list) rather than cut mid-string, so the model never sees half a
//! JSON object.

use serde::Serialize;

/// Default budget when a caller does not pass `max_tokens`.
pub const DEFAULT_MAX_TOKENS: usize = 2_000;

/// Rough token estimate for English text and JSON.
///
/// Uses the common ~4 characters per token heuristic, counting each
/// whitespace-separated word as at least one token so that short dense
/// strings are not under-counted.
pub fn estimate_tokens(text: &str) -> usize {
    let by_chars = text.chars().count().div_ceil(4);
    let by_words = text.split_whitespace().count();
    by_chars.max(by_words)
}

/// Estimate tokens for any serializable value as compact JSON.
pub fn estimate_json_tokens<T: Serialize>(value: &T) -> usize {
    serde_json::to_string(value)
        .map(|s| estimate_tokens(&s))
        .unwrap_or(0)
}

/// The outcome of fitting a list into a budget.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Fitted<T> {
    pub items: Vec<T>,
    /// How many items were dropped to stay under budget.
    pub omitted: usize,
    /// Estimated tokens used by `items`.
    pub tokens: usize,
}

/// Keep items from the front of `items` (which should already be ranked most
/// important first) until the next one would exceed `max_tokens`.
pub fn fit<T: Serialize>(items: Vec<T>, max_tokens: usize) -> Fitted<T> {
    let total = items.len();
    let mut kept = Vec::new();
    let mut tokens = 0;
    for item in items {
        let cost = estimate_json_tokens(&item);
        if tokens + cost > max_tokens && !kept.is_empty() {
            break;
        }
        tokens += cost;
        kept.push(item);
    }
    let omitted = total - kept.len();
    Fitted {
        items: kept,
        omitted,
        tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimates_by_characters_for_long_words() {
        assert_eq!(estimate_tokens("abcdefgh"), 2);
    }

    #[test]
    fn estimates_by_words_for_short_words() {
        assert_eq!(estimate_tokens("a b c d e"), 5);
    }

    #[test]
    fn fit_drops_tail_items_over_budget() {
        let items: Vec<String> = (0..100).map(|i| format!("item number {i}")).collect();
        let fitted = fit(items, 20);
        assert!(fitted.items.len() < 100);
        assert_eq!(fitted.items.len() + fitted.omitted, 100);
        assert!(fitted.tokens <= 20);
        assert_eq!(fitted.items[0], "item number 0");
    }

    #[test]
    fn fit_always_keeps_at_least_one_item() {
        let fitted = fit(vec!["x".repeat(400)], 1);
        assert_eq!(fitted.items.len(), 1);
        assert_eq!(fitted.omitted, 0);
    }
}
