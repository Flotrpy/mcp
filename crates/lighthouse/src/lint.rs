//! Lint tool definitions for how well an *agent* can use them.
//!
//! These rules come from the failure modes reported in MCP server reviews:
//! vague descriptions the model cannot choose between, parameters with no
//! explanation, free-text inputs with no shape, and schemas that contradict
//! themselves.

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Finding {
    pub severity: Severity,
    pub rule: &'static str,
    pub tool: String,
    pub message: String,
}

/// Shortest description that can plausibly say what a tool does and when.
pub const MIN_DESCRIPTION: usize = 20;
/// Descriptions longer than this crowd the context window.
pub const MAX_DESCRIPTION: usize = 1024;

const VAGUE: &[&str] =
    &["does stuff", "handles things", "utility function", "helper", "misc", "todo", "various"];

fn finding(severity: Severity, rule: &'static str, tool: &str, message: String) -> Finding {
    Finding { severity, rule, tool: tool.to_string(), message }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name.chars().all(|c| c.is_ascii_alphanumeric() || "_-.".contains(c))
}

/// Lint one tool definition (as returned by `tools/list`).
pub fn lint_tool(tool: &Value) -> Vec<Finding> {
    let name = tool["name"].as_str().unwrap_or("");
    let label = if name.is_empty() { "<unnamed>" } else { name };
    let mut out = Vec::new();

    if !valid_name(name) {
        out.push(finding(
            Severity::Error,
            "tool-name",
            label,
            "Tool names must be 1–128 characters of letters, digits, `_`, `-` or `.`.".into(),
        ));
    }

    let description = tool["description"].as_str().unwrap_or("").trim();
    if description.is_empty() {
        out.push(finding(
            Severity::Error,
            "description-missing",
            label,
            "No description: the model has nothing to choose this tool by.".into(),
        ));
    } else {
        if description.len() < MIN_DESCRIPTION {
            out.push(finding(
                Severity::Warning,
                "description-short",
                label,
                format!(
                    "Description is {} characters; say what the tool returns and when to use it.",
                    description.len()
                ),
            ));
        }
        if description.len() > MAX_DESCRIPTION {
            out.push(finding(
                Severity::Warning,
                "description-long",
                label,
                format!(
                    "Description is {} characters; move detail into parameter descriptions.",
                    description.len()
                ),
            ));
        }
        let lower = description.to_ascii_lowercase();
        if let Some(word) = VAGUE.iter().find(|w| lower.contains(*w)) {
            out.push(finding(
                Severity::Warning,
                "description-vague",
                label,
                format!("Vague wording (\"{word}\") gives the model nothing to act on."),
            ));
        }
    }

    let schema = &tool["inputSchema"];
    if !schema.is_object() {
        out.push(finding(
            Severity::Error,
            "schema-missing",
            label,
            "inputSchema is required and must be an object schema.".into(),
        ));
        return out;
    }
    if schema["type"] != "object" {
        out.push(finding(
            Severity::Error,
            "schema-not-object",
            label,
            "inputSchema must have \"type\": \"object\".".into(),
        ));
    }

    let props = schema["properties"].as_object();
    if let Some(required) = schema["required"].as_array() {
        for r in required.iter().filter_map(Value::as_str) {
            if !props.is_some_and(|p| p.contains_key(r)) {
                out.push(finding(
                    Severity::Error,
                    "required-unknown",
                    label,
                    format!("`{r}` is required but not defined in properties."),
                ));
            }
        }
    }

    if let Some(props) = props {
        let undocumented: Vec<&str> = props
            .iter()
            .filter(|(_, p)| p.get("description").and_then(Value::as_str).is_none_or(|d| d.trim().is_empty()))
            .filter(|(_, p)| p.get("$ref").is_none() && p.get("enum").is_none())
            .map(|(k, _)| k.as_str())
            .collect();
        if !undocumented.is_empty() && props.len() > 1 {
            out.push(finding(
                Severity::Warning,
                "param-undocumented",
                label,
                format!("Parameters without a description: {}.", undocumented.join(", ")),
            ));
        }
        for (k, p) in props {
            let is_string = p["type"] == "string"
                || p["type"].as_array().is_some_and(|t| t.iter().any(|x| x == "string"));
            let shaped =
                ["enum", "maxLength", "pattern", "format", "const"].iter().any(|key| p.get(*key).is_some());
            if is_string && !shaped && p.get("description").is_none() {
                out.push(finding(
                    Severity::Info,
                    "string-unshaped",
                    label,
                    format!("`{k}` is free text with no enum, pattern or description; the model will guess its format."),
                ));
            }
        }
    }

    if tool.get("annotations").is_none_or(|a| a.get("readOnlyHint").is_none()) {
        out.push(finding(
            Severity::Info,
            "annotations-missing",
            label,
            "No readOnlyHint annotation: clients cannot tell a safe lookup from a write.".into(),
        ));
    }
    out
}

/// Lint a whole tool list, including cross-tool rules.
pub fn lint_all(tools: &[Value]) -> Vec<Finding> {
    let mut out: Vec<Finding> = tools.iter().flat_map(lint_tool).collect();
    let mut names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    names.sort();
    for pair in names.windows(2) {
        if pair[0] == pair[1] {
            out.push(finding(
                Severity::Error,
                "tool-duplicate",
                pair[0],
                "Two tools share this name; clients will only see one.".into(),
            ));
        }
    }
    if tools.len() > 40 {
        out.push(finding(
            Severity::Warning,
            "too-many-tools",
            "*",
            format!("{} tools; models choose worse past ~40. Consider fewer, coarser tools.", tools.len()),
        ));
    }
    out.sort_by(|a, b| a.severity.cmp(&b.severity).then(a.tool.cmp(&b.tool)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rules(f: &[Finding]) -> Vec<&'static str> {
        f.iter().map(|x| x.rule).collect()
    }

    #[test]
    fn a_well_formed_tool_is_clean() {
        let t = json!({
            "name": "search_notes",
            "description": "Full-text search over the user's notes; returns the 10 best matches with snippets.",
            "inputSchema": {"type": "object", "properties": {"query": {"type": "string", "description": "Words to find"}}, "required": ["query"]},
            "annotations": {"readOnlyHint": true}
        });
        assert!(lint_tool(&t).is_empty(), "{:?}", lint_tool(&t));
    }

    #[test]
    fn catches_the_classic_mistakes() {
        let t = json!({
            "name": "do it!",
            "description": "helper",
            "inputSchema": {"type": "object", "properties": {"a": {"type": "string"}, "b": {"type": "number"}}, "required": ["c"]}
        });
        let r = rules(&lint_tool(&t));
        for expected in [
            "tool-name",
            "description-short",
            "description-vague",
            "required-unknown",
            "param-undocumented",
            "string-unshaped",
            "annotations-missing",
        ] {
            assert!(r.contains(&expected), "missing {expected} in {r:?}");
        }
    }

    #[test]
    fn missing_schema_and_duplicates_are_errors() {
        let t = json!({"name": "x", "description": "Returns the current time in UTC as ISO 8601."});
        let all = lint_all(&[t.clone(), t]);
        assert!(rules(&all).contains(&"schema-missing"));
        assert!(rules(&all).contains(&"tool-duplicate"));
        assert_eq!(all[0].severity, Severity::Error, "errors sort first");
    }
}
