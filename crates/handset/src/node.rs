//! The platform-independent UI element model.

use serde::Serialize;

/// What an element is, in words a model already knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Button,
    Text,
    Input,
    Image,
    Checkbox,
    Switch,
    Link,
    List,
    Tab,
    Other,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Button => "button",
            Role::Text => "text",
            Role::Input => "input",
            Role::Image => "image",
            Role::Checkbox => "checkbox",
            Role::Switch => "switch",
            Role::Link => "link",
            Role::List => "list",
            Role::Tab => "tab",
            Role::Other => "other",
        }
    }
}

/// Screen rectangle in device pixels (Android) or points (iOS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Rect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Rect {
    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    pub fn is_empty(&self) -> bool {
        self.x2 <= self.x1 || self.y2 <= self.y1
    }
}

/// One element after parsing, before refs are assigned.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Node {
    pub role: Role,
    /// Visible text, or the accessibility label when there is none.
    pub label: String,
    /// Current value for inputs, sliders and toggles.
    pub value: Option<String>,
    /// Platform identifier (Android resource-id, iOS accessibility identifier).
    pub id: Option<String>,
    pub bounds: Rect,
    pub interactive: bool,
    pub enabled: bool,
    pub checked: Option<bool>,
    pub focused: bool,
    pub scrollable: bool,
}

impl Node {
    /// Elements worth showing to the model: anything it can act on, or any
    /// on-screen text.
    pub fn is_meaningful(&self) -> bool {
        !self.bounds.is_empty() && (self.interactive || self.scrollable || !self.label.is_empty() || self.value.is_some())
    }
}
