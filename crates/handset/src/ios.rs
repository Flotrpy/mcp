//! Parse `idb ui describe-all --json` output from the iOS Simulator.
//!
//! idb prints either a JSON array of elements or one JSON object per line,
//! depending on version; both are accepted.

use serde::Deserialize;

use crate::node::{Node, Rect, Role};

#[derive(Debug, Deserialize)]
struct Frame {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug, Deserialize)]
struct Element {
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(rename = "AXLabel", default)]
    label: Option<String>,
    #[serde(rename = "AXValue", default)]
    value: Option<String>,
    #[serde(rename = "AXUniqueId", default)]
    id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    frame: Option<Frame>,
    #[serde(default = "yes")]
    enabled: bool,
}

fn yes() -> bool {
    true
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("idb output is not JSON: {0}")]
    Json(String),
}

fn role(kind: &str) -> Role {
    match kind {
        "Button" | "Cell" | "MenuItem" | "Key" => Role::Button,
        "StaticText" | "Heading" => Role::Text,
        "TextField" | "SecureTextField" | "SearchField" | "TextView" => Role::Input,
        "Image" | "Icon" => Role::Image,
        "Switch" | "Toggle" => Role::Switch,
        "CheckBox" => Role::Checkbox,
        "Link" => Role::Link,
        "Tab" | "TabBar" | "SegmentedControl" => Role::Tab,
        "ScrollView" | "Table" | "CollectionView" | "List" => Role::List,
        _ => Role::Other,
    }
}

pub fn parse(output: &str) -> Result<Vec<Node>, ParseError> {
    let trimmed = output.trim();
    let elements: Vec<Element> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).map_err(|e| ParseError::Json(e.to_string()))?
    } else {
        trimmed
            .lines()
            .filter(|l| l.trim_start().starts_with('{'))
            .map(|l| serde_json::from_str(l).map_err(|e| ParseError::Json(e.to_string())))
            .collect::<Result<_, _>>()?
    };
    Ok(elements
        .into_iter()
        .filter_map(|e| {
            let f = e.frame?;
            let r = role(&e.kind);
            let secure = e.kind == "SecureTextField";
            let label = e.label.or(e.title).unwrap_or_default();
            let value = e.value.filter(|v| !v.is_empty()).map(|v| if secure { "••••".to_string() } else { v });
            let (checked, value) = if r == Role::Switch {
                (value.as_deref().map(|v| v == "1"), None)
            } else {
                (None, value)
            };
            Some(Node {
                role: r,
                label: label.trim().to_string(),
                value,
                id: e.id.filter(|s| !s.is_empty()),
                bounds: Rect {
                    x1: f.x.round() as i32,
                    y1: f.y.round() as i32,
                    x2: (f.x + f.width).round() as i32,
                    y2: (f.y + f.height).round() as i32,
                },
                interactive: matches!(r, Role::Button | Role::Input | Role::Switch | Role::Checkbox | Role::Link | Role::Tab),
                enabled: e.enabled,
                checked,
                focused: false,
                scrollable: r == Role::List,
            })
        })
        .collect())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub const SETTINGS: &str = r#"[
 {"type":"Application","AXLabel":"Settings","frame":{"x":0,"y":0,"width":390,"height":844},"enabled":true},
 {"type":"Heading","AXLabel":"Wi-Fi","frame":{"x":16,"y":100,"width":200,"height":40},"enabled":true},
 {"type":"Switch","AXLabel":"Wi-Fi","AXValue":"1","AXUniqueId":"wifi-switch","frame":{"x":320,"y":100,"width":51,"height":31},"enabled":true},
 {"type":"SecureTextField","AXLabel":"Password","AXValue":"secret","frame":{"x":16,"y":200,"width":358,"height":44},"enabled":true},
 {"type":"Button","AXLabel":"Join","frame":{"x":16,"y":260,"width":358,"height":44},"enabled":false}
]"#;

    #[test]
    fn parses_array_output() {
        let nodes = parse(SETTINGS).unwrap();
        assert_eq!(nodes.len(), 5);
        assert_eq!(nodes[2].role, Role::Switch);
        assert_eq!(nodes[2].checked, Some(true));
        assert_eq!(nodes[2].id.as_deref(), Some("wifi-switch"));
        assert_eq!(nodes[3].value.as_deref(), Some("••••"));
        assert!(!nodes[4].enabled);
        assert_eq!(nodes[4].bounds.center(), (195, 282));
    }

    #[test]
    fn parses_line_delimited_output() {
        let lines = r#"{"type":"Button","AXLabel":"OK","frame":{"x":0,"y":0,"width":10,"height":10}}
{"type":"StaticText","AXLabel":"Done","frame":{"x":0,"y":20,"width":10,"height":10}}"#;
        let nodes = parse(lines).unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[1].role, Role::Text);
    }

    #[test]
    fn non_json_is_an_error() {
        assert!(parse("[not json").is_err());
    }
}
