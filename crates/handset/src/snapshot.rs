//! Ref assignment, compact rendering and snapshot diffs.
//!
//! **Stable refs.** An element keeps its ref (`e7`) across snapshots as long
//! as its role, label and id stay the same — so an agent can act on `e7`
//! after scrolling or after a field updates. Identical siblings (three
//! "Delete" buttons) are told apart by order.
//!
//! **Collapsing.** Android wraps most tappable text in a clickable container
//! with an unlabelled child `TextView`. The child's text is moved onto the
//! container and the child dropped, so one thing on screen is one line.

use std::collections::HashMap;

use serde::Serialize;

use crate::node::{Node, Rect, Role};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Element {
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(flatten)]
    pub node: Node,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Snapshot {
    pub elements: Vec<Element>,
}

impl Snapshot {
    pub fn get(&self, reference: &str) -> Option<&Element> {
        let r = reference.trim().trim_start_matches('@');
        self.elements.iter().find(|e| e.reference == r)
    }

    /// Elements whose label, value or id contains `needle`, case-insensitively.
    pub fn find(&self, needle: &str) -> Vec<&Element> {
        let n = needle.to_lowercase();
        self.elements
            .iter()
            .filter(|e| {
                e.node.label.to_lowercase().contains(&n)
                    || e.node.value.as_deref().is_some_and(|v| v.to_lowercase().contains(&n))
                    || e.node.id.as_deref().is_some_and(|v| v.to_lowercase().contains(&n))
            })
            .collect()
    }
}

fn contains(outer: &Rect, inner: &Rect) -> bool {
    outer.x1 <= inner.x1 && outer.y1 <= inner.y1 && outer.x2 >= inner.x2 && outer.y2 >= inner.y2
}

fn area(r: &Rect) -> i64 {
    i64::from(r.x2 - r.x1) * i64::from(r.y2 - r.y1)
}

/// Merge label-only text children into their smallest enclosing interactive
/// element, then keep only meaningful elements in reading order.
pub fn collapse(nodes: Vec<Node>) -> Vec<Node> {
    let mut nodes: Vec<Node> = nodes.into_iter().filter(Node::is_meaningful).collect();
    let mut drop = vec![false; nodes.len()];
    for i in 0..nodes.len() {
        if nodes[i].role != Role::Text || nodes[i].interactive {
            continue;
        }
        let parent = (0..nodes.len())
            .filter(|&j| j != i && nodes[j].interactive && nodes[j].role != Role::List && contains(&nodes[j].bounds, &nodes[i].bounds))
            .min_by_key(|&j| area(&nodes[j].bounds));
        if let Some(j) = parent {
            if nodes[j].label.is_empty() {
                nodes[j].label = nodes[i].label.clone();
                drop[i] = true;
            } else if nodes[j].label == nodes[i].label {
                drop[i] = true;
            }
        }
    }
    let mut kept: Vec<Node> = nodes.into_iter().zip(drop).filter(|(_, d)| !d).map(|(n, _)| n).collect();
    // Containers with nothing to say (unlabelled, non-interactive, non-scrolling) add noise.
    kept.retain(|n| !(n.role == Role::Other && n.label.is_empty() && !n.interactive && !n.scrollable));
    kept.sort_by_key(|n| (n.bounds.y1, n.bounds.x1));
    kept
}

type Key = (Role, String, Option<String>, usize);

fn keys(nodes: &[Node]) -> Vec<Key> {
    let mut seen: HashMap<(Role, String, Option<String>), usize> = HashMap::new();
    nodes
        .iter()
        .map(|n| {
            let base = (n.role, n.label.clone(), n.id.clone());
            let ordinal = seen.entry(base.clone()).or_default();
            let k = (base.0, base.1, base.2, *ordinal);
            *ordinal += 1;
            k
        })
        .collect()
}

/// Hands out refs and remembers the last snapshot.
#[derive(Debug, Default)]
pub struct Registry {
    next: u32,
    known: HashMap<Key, String>,
    pub last: Snapshot,
}

impl Registry {
    /// Collapse raw nodes, assign stable refs, remember and return the result.
    pub fn assign(&mut self, raw: Vec<Node>) -> Snapshot {
        let nodes = collapse(raw);
        let ks = keys(&nodes);
        let elements = nodes
            .into_iter()
            .zip(ks)
            .map(|(node, key)| {
                let reference = match self.known.get(&key) {
                    Some(r) => r.clone(),
                    None => {
                        self.next += 1;
                        let r = format!("e{}", self.next);
                        self.known.insert(key, r.clone());
                        r
                    }
                };
                Element { reference, node }
            })
            .collect();
        self.last = Snapshot { elements };
        self.last.clone()
    }
}

/// One element as a single line, e.g.
/// `e4 checkbox "Remember me" [ ]` or `e2 input "Email" = "jo@x" (focused)`.
pub fn line(e: &Element) -> String {
    let n = &e.node;
    let mut s = format!("{} {}", e.reference, n.role.as_str());
    if !n.label.is_empty() {
        s.push_str(&format!(" {:?}", n.label));
    } else if let Some(id) = &n.id {
        s.push_str(&format!(" #{id}"));
    }
    if let Some(v) = &n.value {
        s.push_str(&format!(" = {v:?}"));
    }
    match (n.role, n.checked) {
        (Role::Switch, Some(on)) => s.push_str(if on { " [on]" } else { " [off]" }),
        (_, Some(true)) => s.push_str(" [x]"),
        (_, Some(false)) => s.push_str(" [ ]"),
        _ => {}
    }
    let mut flags = Vec::new();
    if !n.enabled {
        flags.push("disabled");
    }
    if n.focused {
        flags.push("focused");
    }
    if n.scrollable {
        flags.push("scrollable");
    }
    if !flags.is_empty() {
        s.push_str(&format!(" ({})", flags.join(", ")));
    }
    s
}

pub fn render(snapshot: &Snapshot) -> String {
    snapshot.elements.iter().map(line).collect::<Vec<_>>().join("\n")
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Diff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
}

impl Diff {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    pub fn render(&self) -> String {
        if self.is_empty() {
            return "No visible change.".into();
        }
        let mut out: Vec<String> = Vec::new();
        out.extend(self.added.iter().map(|l| format!("+ {l}")));
        out.extend(self.removed.iter().map(|l| format!("- {l}")));
        out.extend(self.changed.iter().map(|l| format!("~ {l}")));
        out.join("\n")
    }
}

/// What changed between two snapshots, by ref.
pub fn diff(before: &Snapshot, after: &Snapshot) -> Diff {
    let old: HashMap<&str, &Element> = before.elements.iter().map(|e| (e.reference.as_str(), e)).collect();
    let new: HashMap<&str, &Element> = after.elements.iter().map(|e| (e.reference.as_str(), e)).collect();
    let mut d = Diff::default();
    for e in &after.elements {
        match old.get(e.reference.as_str()) {
            None => d.added.push(line(e)),
            Some(o) => {
                let (a, b) = (&o.node, &e.node);
                if a.value != b.value || a.checked != b.checked || a.enabled != b.enabled || a.focused != b.focused {
                    d.changed.push(line(e));
                }
            }
        }
    }
    for e in &before.elements {
        if !new.contains_key(e.reference.as_str()) {
            d.removed.push(line(e));
        }
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::android;

    fn node(role: Role, label: &str, y: i32, interactive: bool) -> Node {
        Node {
            role,
            label: label.into(),
            value: None,
            id: None,
            bounds: Rect { x1: 0, y1: y, x2: 100, y2: y + 50 },
            interactive,
            enabled: true,
            checked: None,
            focused: false,
            scrollable: false,
        }
    }

    #[test]
    fn login_screen_renders_one_line_per_thing() {
        let mut reg = Registry::default();
        let snap = reg.assign(android::parse(android::tests::LOGIN).unwrap());
        let text = render(&snap);
        assert_eq!(
            text,
            "e1 text \"Welcome back\"\n\
             e2 input \"Email\" = \"jo@example.com\" (focused)\n\
             e3 input \"Password\" = \"••••\"\n\
             e4 checkbox \"Remember me\" [ ]\n\
             e5 button \"Sign in\" (disabled)"
        );
        assert!(text.len() * 10 < android::tests::LOGIN.len(), "≥10× smaller than the raw dump");
    }

    #[test]
    fn unlabelled_wrapper_takes_its_childs_text() {
        let mut wrapper = node(Role::Button, "", 0, true);
        wrapper.bounds = Rect { x1: 0, y1: 0, x2: 300, y2: 100 };
        let mut child = node(Role::Text, "Continue", 10, false);
        child.bounds = Rect { x1: 10, y1: 10, x2: 200, y2: 90 };
        let out = collapse(vec![wrapper, child]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].label, "Continue");
    }

    #[test]
    fn refs_survive_reordering_and_identical_siblings_stay_distinct() {
        let mut reg = Registry::default();
        let first = reg.assign(vec![node(Role::Button, "Delete", 0, true), node(Role::Button, "Delete", 100, true), node(Role::Text, "Inbox", 200, false)]);
        assert_eq!(first.elements.iter().map(|e| e.reference.as_str()).collect::<Vec<_>>(), ["e1", "e2", "e3"]);
        // Scroll: the header moves to the top; same elements keep their refs.
        let second = reg.assign(vec![node(Role::Text, "Inbox", 0, false), node(Role::Button, "Delete", 50, true), node(Role::Button, "Delete", 150, true)]);
        assert_eq!(second.get("e3").unwrap().node.label, "Inbox");
        assert_eq!(second.elements[1].reference, "e1");
        assert_eq!(second.elements[2].reference, "e2");
    }

    #[test]
    fn diff_reports_added_removed_and_changed() {
        let mut reg = Registry::default();
        let before = reg.assign(vec![node(Role::Button, "Sign in", 0, true), node(Role::Input, "Email", 100, true)]);
        let mut email = node(Role::Input, "Email", 100, true);
        email.value = Some("jo@example.com".into());
        let after = reg.assign(vec![email, node(Role::Text, "Wrong password", 200, false)]);
        let d = diff(&before, &after);
        assert_eq!(d.added, vec!["e3 text \"Wrong password\""]);
        assert_eq!(d.removed, vec!["e1 button \"Sign in\""]);
        assert_eq!(d.changed, vec!["e2 input \"Email\" = \"jo@example.com\""]);
        assert!(d.render().starts_with("+ e3"));
        assert_eq!(diff(&after, &after).render(), "No visible change.");
    }

    #[test]
    fn find_matches_label_value_and_id() {
        let mut reg = Registry::default();
        let mut n = node(Role::Input, "Email", 0, true);
        n.id = Some("email_field".into());
        let snap = reg.assign(vec![n]);
        assert_eq!(snap.find("EMAIL").len(), 1);
        assert_eq!(snap.find("_field").len(), 1);
        assert!(snap.get("@e1").is_some());
    }
}
