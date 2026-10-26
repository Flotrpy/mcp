//! What a server's tool list costs in context tokens, per tool.
//!
//! Every tool definition is sent to the model on every turn, so a heavy list
//! is a tax on every conversation that connects the server.

use serde::Serialize;
use serde_json::Value;

use mcp_kit::budget::estimate_tokens;

use crate::canonical::to_canonical_string;

/// Tool lists above this many tokens start to crowd out real work.
pub const HEAVY_TOTAL: usize = 8_000;
/// A single tool above this is usually carrying documentation it should not.
pub const HEAVY_TOOL: usize = 800;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ToolCost {
    pub tool: String,
    pub tokens: usize,
    /// Share of the whole list, 0–100.
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CostReport {
    pub total_tokens: usize,
    /// Heaviest first.
    pub tools: Vec<ToolCost>,
    pub heavy: bool,
}

pub fn measure(tools: &[Value]) -> CostReport {
    let mut costs: Vec<ToolCost> = tools
        .iter()
        .map(|t| ToolCost {
            tool: t["name"].as_str().unwrap_or("<unnamed>").to_string(),
            tokens: estimate_tokens(&to_canonical_string(t)),
            percent: 0.0,
        })
        .collect();
    let total: usize = costs.iter().map(|c| c.tokens).sum();
    for c in &mut costs {
        c.percent = if total == 0 { 0.0 } else { (1000.0 * c.tokens as f64 / total as f64).round() / 10.0 };
    }
    costs.sort_by(|a, b| b.tokens.cmp(&a.tokens).then(a.tool.cmp(&b.tool)));
    CostReport { total_tokens: total, heavy: total > HEAVY_TOTAL, tools: costs }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ranks_heaviest_first_and_percentages_sum_to_100() {
        let tools = vec![
            json!({"name": "small", "description": "x"}),
            json!({"name": "big", "description": "y".repeat(2000)}),
        ];
        let r = measure(&tools);
        assert_eq!(r.tools[0].tool, "big");
        let sum: f64 = r.tools.iter().map(|c| c.percent).sum();
        assert!((sum - 100.0).abs() < 0.2);
        assert!(!r.heavy);
    }

    #[test]
    fn empty_list_costs_nothing() {
        let r = measure(&[]);
        assert_eq!(r.total_tokens, 0);
        assert!(r.tools.is_empty());
    }
}
