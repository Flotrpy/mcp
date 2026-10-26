//! MCP tool surface for `handset`.

use std::{sync::Arc, time::Duration};

use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerConfig},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::{
    device::{Runner, SystemRunner},
    session::{Action, Direction, Session, Target},
    snapshot,
};
use mcp_kit::result::{structured, tool_error};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeviceArg {
    /// Device serial (Android) or Simulator UDID (iOS). Optional when exactly one is connected.
    pub device: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ActKind {
    Tap,
    Type,
    Scroll,
    Key,
    Launch,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ActArgs {
    /// What to do.
    pub action: ActKind,
    /// Device serial or UDID. Optional when exactly one is connected.
    pub device: Option<String>,
    /// tap/type: element ref from the latest snapshot, e.g. `e7`.
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    /// tap/type: visible text or label to target when you have no ref.
    pub target_text: Option<String>,
    /// tap: x coordinate when targeting a point.
    pub x: Option<i32>,
    /// tap: y coordinate when targeting a point.
    pub y: Option<i32>,
    /// type: the text to enter.
    pub text: Option<String>,
    /// type: press enter afterwards.
    #[serde(default)]
    pub submit: bool,
    /// scroll: which way the content should move.
    pub direction: Option<Direction>,
    /// key: back, home, enter, delete, tab or recents.
    pub key: Option<String>,
    /// launch: Android package or iOS bundle id.
    pub app: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WaitArgs {
    /// Text, label, id or ref to wait for.
    pub target: String,
    /// Wait for it to disappear instead of appear.
    #[serde(default)]
    pub gone: bool,
    /// Give up after this many seconds. Default 10, at most 60.
    pub timeout_seconds: Option<f64>,
    /// Device serial or UDID. Optional when exactly one is connected.
    pub device: Option<String>,
}

#[derive(Clone)]
pub struct Handset {
    session: Arc<Session>,
    tool_router: ToolRouter<Self>,
}

impl Handset {
    pub fn new(runner: Arc<dyn Runner>) -> Self {
        Self { session: Arc::new(Session::new(runner)), tool_router: Self::tool_router() }
    }

    pub fn with_session(session: Session) -> Self {
        Self { session: Arc::new(session), tool_router: Self::tool_router() }
    }
}

impl Default for Handset {
    fn default() -> Self {
        Self::new(Arc::new(SystemRunner::default()))
    }
}

fn target(a: &ActArgs) -> Option<Target> {
    if let Some(r) = &a.reference {
        return Some(Target::Ref(r.clone()));
    }
    if let Some(t) = &a.target_text {
        return Some(Target::Text(t.clone()));
    }
    a.x.zip(a.y).map(|(x, y)| Target::Point(x, y))
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Result<T, ErrorData> {
    tokio::task::spawn_blocking(f).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))
}

#[tool_router]
impl Handset {
    #[tool(
        description = "List connected Android devices/emulators (adb) and booted iOS Simulators, with the ids to pass as `device`.",
        annotations(title = "List devices", read_only_hint = true, open_world_hint = false)
    )]
    pub async fn list_devices(&self) -> Result<CallToolResult, ErrorData> {
        let session = self.session.clone();
        let (devices, notes) = blocking(move || session.devices()).await?;
        let summary = if devices.is_empty() {
            format!("No devices. {}", notes.join(" "))
        } else {
            devices
                .iter()
                .map(|d| format!("{} ({:?}, {})", d.id, d.platform, d.name))
                .collect::<Vec<_>>()
                .join("; ")
        };
        structured(summary, &json!({"devices": devices, "notes": notes}))
    }

    #[tool(
        description = "Read the screen as compact lines, one per meaningful element: `e7 button \"Sign in\" (disabled)`. Refs stay stable across snapshots; act on them with `act`.",
        annotations(title = "Snapshot screen", read_only_hint = true, open_world_hint = false)
    )]
    pub async fn snapshot(&self, Parameters(a): Parameters<DeviceArg>) -> Result<CallToolResult, ErrorData> {
        let session = self.session.clone();
        match blocking(move || session.snapshot(a.device.as_deref())).await? {
            Ok((dev, snap)) => {
                let text = snapshot::render(&snap);
                structured(
                    if text.is_empty() { "The screen has no readable elements.".to_string() } else { text },
                    &json!({"device": dev, "elements": snap.elements}),
                )
            }
            Err(e) => Ok(tool_error(e.to_string())),
        }
    }

    #[tool(
        description = "Tap, type, scroll, press a key or launch an app, then return only what changed on screen (+ added, - removed, ~ changed). Target by `ref` (best), `target_text`, or `x`/`y`.",
        annotations(
            title = "Act on the device",
            read_only_hint = false,
            destructive_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn act(&self, Parameters(a): Parameters<ActArgs>) -> Result<CallToolResult, ErrorData> {
        let action = match a.action {
            ActKind::Tap => match target(&a) {
                Some(t) => Action::Tap(t),
                None => return Ok(tool_error("tap needs `ref`, `target_text`, or both `x` and `y`.")),
            },
            ActKind::Type => match &a.text {
                Some(text) => Action::Type { target: target(&a), text: text.clone(), submit: a.submit },
                None => {
                    return Ok(tool_error(
                        "type needs `text` (and optionally a `ref` or `target_text` to focus first).",
                    ));
                }
            },
            ActKind::Scroll => Action::Scroll(a.direction.unwrap_or(Direction::Down)),
            ActKind::Key => match &a.key {
                Some(k) => Action::Key(k.clone()),
                None => return Ok(tool_error("key needs `key`: back, home, enter, delete, tab or recents.")),
            },
            ActKind::Launch => match &a.app {
                Some(app) => Action::Launch(app.clone()),
                None => return Ok(tool_error("launch needs `app`: an Android package or iOS bundle id.")),
            },
        };
        let session = self.session.clone();
        let device = a.device.clone();
        match blocking(move || session.act(device.as_deref(), action)).await? {
            Ok(out) => {
                let mut text = format!("{}.\n{}", out.did, out.diff.render());
                if let Some(screen) = &out.screen {
                    text.push_str("\n\nNew screen:\n");
                    text.push_str(screen);
                }
                structured(text, &out)
            }
            Err(e) => Ok(tool_error(e.to_string())),
        }
    }

    #[tool(
        description = "Wait until text, a label, an id or a ref appears (or with gone=true, disappears), polling the screen. Returns the screen when it does. Use instead of sleeping.",
        annotations(title = "Wait for element", read_only_hint = true, open_world_hint = false)
    )]
    pub async fn wait_for(&self, Parameters(a): Parameters<WaitArgs>) -> Result<CallToolResult, ErrorData> {
        let timeout = Duration::from_secs_f64(a.timeout_seconds.unwrap_or(10.0).clamp(0.5, 60.0));
        let session = self.session.clone();
        let (needle, gone, device) = (a.target.clone(), a.gone, a.device.clone());
        match blocking(move || session.wait_for(device.as_deref(), &needle, gone, timeout)).await? {
            Ok((dev, snap, secs)) => structured(
                format!(
                    "\"{}\" {} after {secs:.1}s.\n{}",
                    a.target,
                    if a.gone { "disappeared" } else { "appeared" },
                    snapshot::render(&snap)
                ),
                &json!({"device": dev, "waited_seconds": secs, "elements": snap.elements}),
            ),
            Err(e) => Ok(tool_error(e.to_string())),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Handset {
    fn get_info(&self) -> ServerConfig {
        mcp_kit::server_config(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            "Handset — phone and Simulator control",
            "Call snapshot to read the screen, then act using element refs (e.g. ref=e7). Every act returns only \
             what changed, so you rarely need another snapshot. Use wait_for instead of sleeping. Android needs adb; \
             the iOS Simulator needs idb and Xcode.",
        )
    }
}
