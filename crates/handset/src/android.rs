//! Parse `uiautomator dump` XML.

use quick_xml::{Reader, events::Event};

use crate::node::{Node, Rect, Role};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("UI dump is not valid XML: {0}")]
    Xml(String),
    #[error("UI dump contained no hierarchy (is the screen locked or the app secure?)")]
    Empty,
}

/// `[0,63][1080,210]` → Rect.
pub fn parse_bounds(s: &str) -> Option<Rect> {
    let nums: Vec<i32> = s
        .split(|c: char| !(c.is_ascii_digit() || c == '-'))
        .filter(|t| !t.is_empty())
        .filter_map(|t| t.parse().ok())
        .collect();
    match nums.as_slice() {
        [x1, y1, x2, y2] => Some(Rect { x1: *x1, y1: *y1, x2: *x2, y2: *y2 }),
        _ => None,
    }
}

fn role(class: &str, clickable: bool, checkable: bool) -> Role {
    let short = class.rsplit('.').next().unwrap_or(class);
    match short {
        "Button" | "ImageButton" | "FloatingActionButton" | "MaterialButton" | "AppCompatButton" => {
            Role::Button
        }
        "EditText" | "AutoCompleteTextView" | "TextInputEditText" | "AppCompatEditText" | "SearchView" => {
            Role::Input
        }
        "CheckBox" | "AppCompatCheckBox" | "RadioButton" => Role::Checkbox,
        "Switch" | "SwitchCompat" | "SwitchMaterial" | "ToggleButton" => Role::Switch,
        "ImageView" | "AppCompatImageView" => {
            if clickable {
                Role::Button
            } else {
                Role::Image
            }
        }
        "RecyclerView"
        | "ListView"
        | "ScrollView"
        | "HorizontalScrollView"
        | "NestedScrollView"
        | "GridView" => Role::List,
        "TabView" | "TabLayout$TabView" => Role::Tab,
        "TextView" | "AppCompatTextView" | "MaterialTextView" => {
            if clickable {
                Role::Button
            } else {
                Role::Text
            }
        }
        _ if checkable => Role::Checkbox,
        _ if clickable => Role::Button,
        _ => Role::Other,
    }
}

/// Parse a dump into a flat list of nodes in document order. The trailing
/// "UI hierchary dumped to: …" line that `uiautomator` prints is ignored.
pub fn parse(xml: &str) -> Result<Vec<Node>, ParseError> {
    let start = xml.find('<').ok_or(ParseError::Empty)?;
    let end = xml.rfind('>').map_or(xml.len(), |i| i + 1);
    let mut reader = Reader::from_str(&xml[start..end]);
    let mut nodes = Vec::new();
    let mut saw_hierarchy = false;
    loop {
        match reader.read_event().map_err(|e| ParseError::Xml(e.to_string()))? {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => {
                let tag = e.name();
                if tag.as_ref() == b"hierarchy" {
                    saw_hierarchy = true;
                    continue;
                }
                if tag.as_ref() != b"node" {
                    continue;
                }
                let mut get = std::collections::HashMap::new();
                for a in e.attributes().flatten() {
                    let k = String::from_utf8_lossy(a.key.as_ref()).into_owned();
                    let v = a.unescape_value().map(|c| c.into_owned()).unwrap_or_default();
                    get.insert(k, v);
                }
                let flag = |k: &str| get.get(k).is_some_and(|v| v == "true");
                let Some(bounds) = get.get("bounds").and_then(|b| parse_bounds(b)) else { continue };
                let class = get.get("class").cloned().unwrap_or_default();
                let clickable = flag("clickable") || flag("long-clickable");
                let checkable = flag("checkable");
                let r = role(&class, clickable, checkable);
                let text = get.get("text").cloned().unwrap_or_default();
                let desc = get.get("content-desc").cloned().unwrap_or_default();
                let password = flag("password");
                let (label, value) = if r == Role::Input {
                    // For inputs, text is the value; the hint lives in content-desc.
                    let v = if password && !text.is_empty() {
                        Some("••••".to_string())
                    } else {
                        (!text.is_empty()).then_some(text.clone())
                    };
                    (desc, v)
                } else if text.is_empty() {
                    (desc, None)
                } else {
                    (text, None)
                };
                let id = get
                    .get("resource-id")
                    .filter(|s| !s.is_empty())
                    .map(|s| s.rsplit('/').next().unwrap_or(s).to_string());
                nodes.push(Node {
                    role: r,
                    label: label.trim().to_string(),
                    value,
                    id,
                    bounds,
                    interactive: clickable
                        || checkable
                        || r == Role::Input
                        || flag("focusable") && r != Role::Other && r != Role::Text,
                    enabled: get.get("enabled").is_none_or(|v| v == "true"),
                    checked: checkable.then(|| flag("checked")),
                    focused: flag("focused"),
                    scrollable: flag("scrollable"),
                });
            }
            _ => {}
        }
    }
    if !saw_hierarchy {
        return Err(ParseError::Empty);
    }
    Ok(nodes)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub const LOGIN: &str = r#"<?xml version='1.0' encoding='UTF-8' standalone='yes' ?>
<hierarchy rotation="0">
  <node index="0" text="" resource-id="" class="android.widget.FrameLayout" package="com.example" content-desc="" checkable="false" checked="false" clickable="false" enabled="true" focusable="false" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[0,0][1080,2400]">
    <node index="0" text="Welcome back" resource-id="com.example:id/title" class="android.widget.TextView" package="com.example" content-desc="" checkable="false" checked="false" clickable="false" enabled="true" focusable="false" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[96,300][984,400]" />
    <node index="1" text="jo@example.com" resource-id="com.example:id/email" class="android.widget.EditText" package="com.example" content-desc="Email" checkable="false" checked="false" clickable="true" enabled="true" focusable="true" focused="true" scrollable="false" long-clickable="true" password="false" selected="false" bounds="[96,500][984,640]" />
    <node index="2" text="hunter2" resource-id="com.example:id/password" class="android.widget.EditText" package="com.example" content-desc="Password" checkable="false" checked="false" clickable="true" enabled="true" focusable="true" focused="false" scrollable="false" long-clickable="true" password="true" selected="false" bounds="[96,680][984,820]" />
    <node index="3" text="Remember me" resource-id="com.example:id/remember" class="android.widget.CheckBox" package="com.example" content-desc="" checkable="true" checked="false" clickable="true" enabled="true" focusable="true" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[96,860][600,960]" />
    <node index="4" text="Sign in" resource-id="com.example:id/sign_in" class="android.widget.Button" package="com.example" content-desc="" checkable="false" checked="false" clickable="true" enabled="false" focusable="true" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[96,1000][984,1140]" />
    <node index="5" text="" resource-id="" class="android.view.View" package="com.example" content-desc="" checkable="false" checked="false" clickable="false" enabled="true" focusable="false" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[0,1200][1080,1300]" />
  </node>
</hierarchy>
UI hierchary dumped to: /dev/tty"#;

    #[test]
    fn parses_bounds() {
        assert_eq!(parse_bounds("[0,63][1080,210]"), Some(Rect { x1: 0, y1: 63, x2: 1080, y2: 210 }));
        assert_eq!(parse_bounds("garbage"), None);
    }

    #[test]
    fn parses_a_login_screen() {
        let nodes = parse(LOGIN).unwrap();
        assert_eq!(nodes.len(), 7);
        let email = &nodes[2];
        assert_eq!(
            (email.role, email.label.as_str(), email.value.as_deref()),
            (Role::Input, "Email", Some("jo@example.com"))
        );
        assert!(email.focused);
        assert_eq!(nodes[3].value.as_deref(), Some("••••"), "passwords are never echoed");
        assert_eq!(nodes[4].checked, Some(false));
        assert_eq!(nodes[5].role, Role::Button);
        assert!(!nodes[5].enabled);
        assert_eq!(nodes[5].id.as_deref(), Some("sign_in"));
        assert!(!nodes[6].is_meaningful(), "empty decorative view");
    }

    #[test]
    fn empty_or_broken_dumps_are_errors() {
        assert_eq!(parse("ERROR: could not get idle state."), Err(ParseError::Empty));
    }
}
