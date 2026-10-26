//! Actions, waits and snapshots for one or more devices, independent of MCP.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::{
    device::{self, Backend, DeviceError, DeviceInfo, Runner},
    snapshot::{self, Diff, Element, Registry, Snapshot},
};

/// What to act on.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    /// A ref from the latest snapshot, e.g. `e7`.
    Ref(String),
    /// Visible text, label or id; must match exactly one element.
    Text(String),
    Point(i32, i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Tap(Target),
    /// Optionally focus `target` first, type, optionally press enter.
    Type { target: Option<Target>, text: String, submit: bool },
    /// Scroll the content in `direction` (swipes the opposite way).
    Scroll(Direction),
    Key(String),
    Launch(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ActError {
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error("{0} is not on screen any more; call snapshot to get fresh refs")]
    StaleRef(String),
    #[error("nothing on screen matches \"{0}\"")]
    NoMatch(String),
    #[error("\"{needle}\" matches {count} elements ({refs}); act on a ref instead")]
    Ambiguous { needle: String, count: usize, refs: String },
    #[error("timed out after {0:.1}s waiting for {1}")]
    Timeout(f64, String),
}

#[derive(Debug, Serialize)]
pub struct Outcome {
    pub device: DeviceInfo,
    pub did: String,
    pub diff: Diff,
    /// The whole new screen, included when most of it changed (navigation).
    pub screen: Option<String>,
}

/// When more than this share of elements changed, return the full screen.
const NAVIGATION_SHARE: f64 = 0.6;

pub struct Session {
    runner: Arc<dyn Runner>,
    registries: Mutex<HashMap<String, Registry>>,
    /// Pause after an action before re-reading the screen.
    pub settle: Duration,
    pub poll: Duration,
}

impl Session {
    pub fn new(runner: Arc<dyn Runner>) -> Self {
        Self { runner, registries: Mutex::new(HashMap::new()), settle: Duration::from_millis(400), poll: Duration::from_millis(500) }
    }

    pub fn devices(&self) -> (Vec<DeviceInfo>, Vec<String>) {
        device::list(&*self.runner)
    }

    fn device(&self, wanted: Option<&str>) -> Result<DeviceInfo, DeviceError> {
        let (devices, _) = self.devices();
        device::choose(&devices, wanted)
    }

    fn read(&self, dev: &DeviceInfo) -> Result<Snapshot, DeviceError> {
        let nodes = Backend { runner: &*self.runner, device: dev.clone() }.dump()?;
        let mut regs = self.registries.lock().unwrap();
        Ok(regs.entry(dev.id.clone()).or_default().assign(nodes))
    }

    fn last(&self, dev: &DeviceInfo) -> Option<Snapshot> {
        let regs = self.registries.lock().unwrap();
        regs.get(&dev.id).map(|r| r.last.clone()).filter(|s| !s.elements.is_empty())
    }

    pub fn snapshot(&self, wanted: Option<&str>) -> Result<(DeviceInfo, Snapshot), DeviceError> {
        let dev = self.device(wanted)?;
        let snap = self.read(&dev)?;
        Ok((dev, snap))
    }

    fn resolve(snap: &Snapshot, target: &Target) -> Result<((i32, i32), String), ActError> {
        let describe = |e: &Element| format!("{} {:?}", e.reference, e.node.label);
        match target {
            Target::Point(x, y) => Ok(((*x, *y), format!("({x}, {y})"))),
            Target::Ref(r) => snap.get(r).map(|e| (e.node.bounds.center(), describe(e))).ok_or_else(|| ActError::StaleRef(r.clone())),
            Target::Text(t) => {
                let matches = snap.find(t);
                // Prefer something tappable when text also appears as a label.
                let interactive: Vec<&&Element> = matches.iter().filter(|e| e.node.interactive).collect();
                let pick: Vec<&Element> = if interactive.len() == 1 { vec![*interactive[0]] } else { matches };
                match pick.as_slice() {
                    [] => Err(ActError::NoMatch(t.clone())),
                    [one] => Ok((one.node.bounds.center(), describe(one))),
                    many => Err(ActError::Ambiguous {
                        needle: t.clone(),
                        count: many.len(),
                        refs: many.iter().map(|e| e.reference.as_str()).collect::<Vec<_>>().join(", "),
                    }),
                }
            }
        }
    }

    /// Perform `action` and report only what changed on screen.
    pub fn act(&self, wanted: Option<&str>, action: Action) -> Result<Outcome, ActError> {
        let dev = self.device(wanted)?;
        let before = match self.last(&dev) {
            Some(s) => s,
            None => self.read(&dev)?,
        };
        let backend = Backend { runner: &*self.runner, device: dev.clone() };
        let did = match &action {
            Action::Tap(t) => {
                let ((x, y), what) = Self::resolve(&before, t)?;
                backend.tap(x, y)?;
                format!("Tapped {what}")
            }
            Action::Type { target, text, submit } => {
                let mut what = String::from("the focused field");
                if let Some(t) = target {
                    let ((x, y), w) = Self::resolve(&before, t)?;
                    backend.tap(x, y)?;
                    what = w;
                }
                backend.type_text(text)?;
                if *submit {
                    backend.key("enter")?;
                }
                format!("Typed {} characters into {what}{}", text.chars().count(), if *submit { " and pressed enter" } else { "" })
            }
            Action::Scroll(dir) => {
                let screen = before
                    .elements
                    .iter()
                    .map(|e| e.node.bounds)
                    .fold((0, 0, 1080, 1920), |(a, b, c, d), r| (a.min(r.x1), b.min(r.y1), c.max(r.x2), d.max(r.y2)));
                let (cx, cy) = ((screen.0 + screen.2) / 2, (screen.1 + screen.3) / 2);
                let (dx, dy) = ((screen.2 - screen.0) / 3, (screen.3 - screen.1) / 3);
                // Content moves opposite to the finger.
                let (from, to) = match dir {
                    Direction::Down => ((cx, cy + dy), (cx, cy - dy)),
                    Direction::Up => ((cx, cy - dy), (cx, cy + dy)),
                    Direction::Right => ((cx + dx, cy), (cx - dx, cy)),
                    Direction::Left => ((cx - dx, cy), (cx + dx, cy)),
                };
                backend.swipe(from, to, 300)?;
                format!("Scrolled {dir:?}").to_lowercase()
            }
            Action::Key(k) => {
                backend.key(k)?;
                format!("Pressed {k}")
            }
            Action::Launch(app) => {
                backend.launch(app)?;
                format!("Launched {app}")
            }
        };
        std::thread::sleep(self.settle);
        let after = self.read(&dev)?;
        let diff = snapshot::diff(&before, &after);
        let changed = diff.added.len() + diff.removed.len();
        let base = before.elements.len().max(after.elements.len()).max(1);
        let screen = (changed as f64 / base as f64 > NAVIGATION_SHARE).then(|| snapshot::render(&after));
        Ok(Outcome { device: dev, did, diff, screen })
    }

    /// Poll until something matching `needle` appears (or disappears).
    pub fn wait_for(&self, wanted: Option<&str>, needle: &str, gone: bool, timeout: Duration) -> Result<(DeviceInfo, Snapshot, f64), ActError> {
        let dev = self.device(wanted)?;
        let started = Instant::now();
        loop {
            let snap = self.read(&dev)?;
            let present = snap.get(needle).is_some() || !snap.find(needle).is_empty();
            if present != gone {
                return Ok((dev, snap, started.elapsed().as_secs_f64()));
            }
            if started.elapsed() >= timeout {
                let what = if gone { format!("\"{needle}\" to disappear") } else { format!("\"{needle}\"") };
                return Err(ActError::Timeout(timeout.as_secs_f64(), what));
            }
            std::thread::sleep(self.poll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{android, device::tests::FakeRunner};

    const DEVICES: &str = "List of devices attached\nemulator-5554 device model:Pixel_8\n";

    fn session(runner: &Arc<FakeRunner>) -> Session {
        runner.reply("adb devices", DEVICES);
        runner.reply("adb -s emulator-5554 exec-out uiautomator dump", android::tests::LOGIN);
        let mut s = Session::new(runner.clone());
        s.settle = Duration::ZERO;
        s.poll = Duration::from_millis(1);
        s
    }

    #[test]
    fn tap_by_ref_uses_the_element_centre_and_reports_no_change() {
        let r = Arc::new(FakeRunner::default());
        let s = session(&r);
        let (_, snap) = s.snapshot(None).unwrap();
        let sign_in = snap.find("Sign in")[0].reference.clone();
        let out = s.act(None, Action::Tap(Target::Ref(sign_in))).unwrap();
        assert!(out.did.starts_with("Tapped e5"));
        assert!(r.calls.lock().unwrap().iter().any(|c| c == "adb -s emulator-5554 shell input tap 540 1070"));
        assert!(out.diff.is_empty());
        assert!(out.screen.is_none());
    }

    #[test]
    fn type_by_text_focuses_then_types_then_submits() {
        let r = Arc::new(FakeRunner::default());
        let s = session(&r);
        let out = s.act(None, Action::Type { target: Some(Target::Text("Password".into())), text: "pw 1".into(), submit: true }).unwrap();
        assert!(out.did.contains("Typed 4 characters"));
        let calls = r.calls.lock().unwrap();
        let tail: Vec<&String> = calls.iter().filter(|c| c.contains("input")).collect();
        assert_eq!(tail[0], "adb -s emulator-5554 shell input tap 540 750");
        assert_eq!(tail[1], "adb -s emulator-5554 shell input text pw%s1");
        assert_eq!(tail[2], "adb -s emulator-5554 shell input keyevent KEYCODE_ENTER");
    }

    #[test]
    fn a_screen_change_is_reported_as_a_diff_and_navigation_returns_the_screen() {
        let r = Arc::new(FakeRunner::default());
        let s = session(&r);
        s.snapshot(None).unwrap();
        let home = LOGIN_TO_HOME;
        r.reply("adb -s emulator-5554 exec-out uiautomator dump", home);
        let out = s.act(None, Action::Tap(Target::Text("Sign in".into()))).unwrap();
        assert!(out.diff.added.iter().any(|l| l.contains("Inbox")));
        assert!(out.screen.as_deref().unwrap().contains("Inbox"));
    }

    const LOGIN_TO_HOME: &str = r#"<hierarchy><node class="android.widget.FrameLayout" bounds="[0,0][1080,2400]"><node text="Inbox" class="android.widget.TextView" bounds="[40,100][400,180]"/><node text="Compose" class="android.widget.Button" clickable="true" bounds="[700,2200][1040,2340]"/></node></hierarchy>"#;

    #[test]
    fn stale_and_ambiguous_targets_explain_how_to_recover() {
        let r = Arc::new(FakeRunner::default());
        let s = session(&r);
        s.snapshot(None).unwrap();
        let err = s.act(None, Action::Tap(Target::Ref("e99".into()))).unwrap_err().to_string();
        assert!(err.contains("call snapshot"), "{err}");
        let err = s.act(None, Action::Tap(Target::Text("e".into()))).unwrap_err().to_string();
        assert!(err.contains("act on a ref"), "{err}");
    }

    #[test]
    fn wait_for_returns_when_text_appears_or_times_out() {
        let r = Arc::new(FakeRunner::default());
        let s = session(&r);
        let (_, snap, _) = s.wait_for(None, "Welcome", false, Duration::from_millis(50)).unwrap();
        assert!(!snap.elements.is_empty());
        let err = s.wait_for(None, "Never shows up", false, Duration::from_millis(20)).unwrap_err();
        assert!(matches!(err, ActError::Timeout(..)));
        assert!(s.wait_for(None, "Never shows up", true, Duration::from_millis(20)).is_ok());
    }

    #[test]
    fn scroll_swipes_against_the_direction() {
        let r = Arc::new(FakeRunner::default());
        let s = session(&r);
        s.act(None, Action::Scroll(Direction::Down)).unwrap();
        let calls = r.calls.lock().unwrap();
        let swipe = calls.iter().find(|c| c.contains("swipe")).unwrap();
        let nums: Vec<i32> = swipe.split_whitespace().filter_map(|t| t.parse().ok()).collect();
        assert!(nums[1] > nums[3], "finger moves up to scroll content down: {swipe}");
    }
}
