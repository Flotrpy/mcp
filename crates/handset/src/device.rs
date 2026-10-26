//! Device backends: Android over `adb`, iOS Simulator over `idb` + `simctl`.
//!
//! All process execution goes through [`Runner`], so the command lines can be
//! tested without a device attached.

use std::time::Duration;

use serde::Serialize;

use crate::{android, ios, node::Node};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Android,
    Ios,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub platform: Platform,
    pub name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("`{program}` is not installed or not on PATH ({hint})")]
    Missing { program: String, hint: &'static str },
    #[error("`{command}` failed: {stderr}")]
    Failed { command: String, stderr: String },
    #[error("{0}")]
    Parse(String),
    #[error("no device connected; start an emulator or Simulator, or plug in a phone with USB debugging on")]
    NoDevice,
    #[error("several devices connected ({0}); pass `device` to choose one")]
    Ambiguous(String),
    #[error("no device with id `{0}`")]
    UnknownDevice(String),
}

/// Runs external commands. The real implementation shells out; tests record.
pub trait Runner: Send + Sync {
    fn run(&self, program: &str, args: &[String]) -> Result<String, DeviceError>;
}

pub struct SystemRunner {
    pub timeout: Duration,
}

impl Default for SystemRunner {
    fn default() -> Self {
        Self { timeout: Duration::from_secs(20) }
    }
}

fn hint(program: &str) -> &'static str {
    match program {
        "adb" => "install Android platform-tools",
        "idb" => "install idb: brew install facebook/fb/idb-companion && pip install fb-idb",
        "xcrun" => "install Xcode command-line tools",
        _ => "check your PATH",
    }
}

impl Runner for SystemRunner {
    fn run(&self, program: &str, args: &[String]) -> Result<String, DeviceError> {
        let mut child = std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|_| DeviceError::Missing { program: program.into(), hint: hint(program) })?;
        let deadline = std::time::Instant::now() + self.timeout;
        loop {
            if child.try_wait().ok().flatten().is_some() {
                break;
            }
            if std::time::Instant::now() >= deadline {
                let _ = child.kill();
                return Err(DeviceError::Failed {
                    command: format!("{program} {}", args.join(" ")),
                    stderr: format!("timed out after {}s", self.timeout.as_secs()),
                });
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let out = child
            .wait_with_output()
            .map_err(|e| DeviceError::Failed { command: program.into(), stderr: e.to_string() })?;
        if !out.status.success() {
            return Err(DeviceError::Failed {
                command: format!("{program} {}", args.join(" ")),
                stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
            });
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

/// Escape text for `adb shell input text`: spaces become `%s` and shell
/// metacharacters are backslash-escaped.
pub fn adb_text(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            ' ' => out.push_str("%s"),
            '\'' | '"' | '\\' | '&' | '|' | ';' | '<' | '>' | '(' | ')' | '$' | '`' | '*' | '?' | '#'
            | '~' | '!' | '%' => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out
}

/// Keys every platform understands.
pub fn android_keycode(key: &str) -> Option<&'static str> {
    Some(match key.to_ascii_lowercase().as_str() {
        "back" => "KEYCODE_BACK",
        "home" => "KEYCODE_HOME",
        "enter" | "return" => "KEYCODE_ENTER",
        "delete" | "backspace" => "KEYCODE_DEL",
        "tab" => "KEYCODE_TAB",
        "app_switch" | "recents" => "KEYCODE_APP_SWITCH",
        _ => return None,
    })
}

pub struct Backend<'a> {
    pub runner: &'a dyn Runner,
    pub device: DeviceInfo,
}

impl Backend<'_> {
    fn adb(&self, args: &[&str]) -> Result<String, DeviceError> {
        let mut full = s(&["-s", &self.device.id]);
        full.extend(s(args));
        self.runner.run("adb", &full)
    }

    fn idb(&self, args: &[&str]) -> Result<String, DeviceError> {
        let mut full = s(args);
        full.extend(s(&["--udid", &self.device.id]));
        self.runner.run("idb", &full)
    }

    pub fn dump(&self) -> Result<Vec<Node>, DeviceError> {
        match self.device.platform {
            Platform::Android => {
                let xml = self.adb(&["exec-out", "uiautomator", "dump", "/dev/tty"])?;
                android::parse(&xml).map_err(|e| DeviceError::Parse(e.to_string()))
            }
            Platform::Ios => {
                let json = self.idb(&["ui", "describe-all", "--json"])?;
                ios::parse(&json).map_err(|e| DeviceError::Parse(e.to_string()))
            }
        }
    }

    pub fn tap(&self, x: i32, y: i32) -> Result<(), DeviceError> {
        let (x, y) = (x.to_string(), y.to_string());
        match self.device.platform {
            Platform::Android => self.adb(&["shell", "input", "tap", &x, &y]).map(drop),
            Platform::Ios => self.idb(&["ui", "tap", &x, &y]).map(drop),
        }
    }

    pub fn type_text(&self, text: &str) -> Result<(), DeviceError> {
        match self.device.platform {
            Platform::Android => self.adb(&["shell", "input", "text", &adb_text(text)]).map(drop),
            Platform::Ios => self.idb(&["ui", "text", text]).map(drop),
        }
    }

    pub fn swipe(&self, from: (i32, i32), to: (i32, i32), duration_ms: u32) -> Result<(), DeviceError> {
        let v: Vec<String> = [from.0, from.1, to.0, to.1].iter().map(i32::to_string).collect();
        let ms = duration_ms.to_string();
        match self.device.platform {
            Platform::Android => {
                self.adb(&["shell", "input", "swipe", &v[0], &v[1], &v[2], &v[3], &ms]).map(drop)
            }
            Platform::Ios => {
                let secs = format!("{:.2}", f64::from(duration_ms) / 1000.0);
                self.idb(&["ui", "swipe", &v[0], &v[1], &v[2], &v[3], "--duration", &secs]).map(drop)
            }
        }
    }

    pub fn key(&self, key: &str) -> Result<(), DeviceError> {
        match self.device.platform {
            Platform::Android => {
                let code = android_keycode(key).ok_or_else(|| {
                    DeviceError::Parse(format!(
                        "unknown key `{key}`; use back, home, enter, delete, tab or recents"
                    ))
                })?;
                self.adb(&["shell", "input", "keyevent", code]).map(drop)
            }
            Platform::Ios => match key.to_ascii_lowercase().as_str() {
                "home" => self.idb(&["ui", "button", "HOME"]).map(drop),
                "enter" | "return" => self.idb(&["ui", "key", "40"]).map(drop),
                "delete" | "backspace" => self.idb(&["ui", "key", "42"]).map(drop),
                _ => Err(DeviceError::Parse(format!(
                    "key `{key}` is not available on iOS; use home, enter or delete"
                ))),
            },
        }
    }

    pub fn launch(&self, app: &str) -> Result<(), DeviceError> {
        match self.device.platform {
            Platform::Android => self
                .adb(&["shell", "monkey", "-p", app, "-c", "android.intent.category.LAUNCHER", "1"])
                .map(drop),
            Platform::Ios => {
                self.runner.run("xcrun", &s(&["simctl", "launch", &self.device.id, app])).map(drop)
            }
        }
    }
}

/// Parse `adb devices -l`.
pub fn parse_adb_devices(out: &str) -> Vec<DeviceInfo> {
    out.lines()
        .skip_while(|l| !l.starts_with("List of devices"))
        .skip(1)
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            let id = parts.next()?;
            let state = parts.next()?;
            if state != "device" {
                return None;
            }
            let name = parts.find_map(|p| p.strip_prefix("model:")).unwrap_or(id).replace('_', " ");
            Some(DeviceInfo { id: id.into(), platform: Platform::Android, name })
        })
        .collect()
}

/// Parse `xcrun simctl list devices booted -j`.
pub fn parse_simctl_devices(out: &str) -> Vec<DeviceInfo> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(out) else { return Vec::new() };
    let Some(runtimes) = v["devices"].as_object() else { return Vec::new() };
    runtimes
        .values()
        .flat_map(|list| list.as_array().cloned().unwrap_or_default())
        .filter(|d| d["state"] == "Booted")
        .filter_map(|d| {
            Some(DeviceInfo {
                id: d["udid"].as_str()?.into(),
                platform: Platform::Ios,
                name: d["name"].as_str().unwrap_or("iOS Simulator").into(),
            })
        })
        .collect()
}

/// Every connected Android device and booted Simulator. Missing tools are
/// not an error — a Linux machine simply has no Simulators.
pub fn list(runner: &dyn Runner) -> (Vec<DeviceInfo>, Vec<String>) {
    let mut devices = Vec::new();
    let mut notes = Vec::new();
    match runner.run("adb", &s(&["devices", "-l"])) {
        Ok(out) => devices.extend(parse_adb_devices(&out)),
        Err(e) => notes.push(e.to_string()),
    }
    if cfg!(target_os = "macos") {
        match runner.run("xcrun", &s(&["simctl", "list", "devices", "booted", "-j"])) {
            Ok(out) => devices.extend(parse_simctl_devices(&out)),
            Err(e) => notes.push(e.to_string()),
        }
    }
    (devices, notes)
}

/// Pick `wanted`, or the only device when there is exactly one.
pub fn choose(devices: &[DeviceInfo], wanted: Option<&str>) -> Result<DeviceInfo, DeviceError> {
    match wanted {
        Some(id) => {
            devices.iter().find(|d| d.id == id).cloned().ok_or_else(|| DeviceError::UnknownDevice(id.into()))
        }
        None => match devices {
            [] => Err(DeviceError::NoDevice),
            [one] => Ok(one.clone()),
            many => {
                Err(DeviceError::Ambiguous(many.iter().map(|d| d.id.as_str()).collect::<Vec<_>>().join(", ")))
            }
        },
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Records every command and replies from a script of canned outputs.
    #[derive(Default)]
    pub struct FakeRunner {
        pub calls: Mutex<Vec<String>>,
        pub replies: Mutex<Vec<(String, String)>>,
    }

    impl FakeRunner {
        pub fn reply(&self, prefix: &str, out: &str) {
            self.replies.lock().unwrap().push((prefix.into(), out.into()));
        }
    }

    impl Runner for FakeRunner {
        fn run(&self, program: &str, args: &[String]) -> Result<String, DeviceError> {
            let line = format!("{program} {}", args.join(" "));
            self.calls.lock().unwrap().push(line.clone());
            let replies = self.replies.lock().unwrap();
            Ok(replies
                .iter()
                .rev()
                .find(|(p, _)| line.starts_with(p.as_str()))
                .map(|(_, o)| o.clone())
                .unwrap_or_default())
        }
    }

    fn android() -> DeviceInfo {
        DeviceInfo { id: "emulator-5554".into(), platform: Platform::Android, name: "Pixel".into() }
    }

    #[test]
    fn android_command_lines() {
        let r = FakeRunner::default();
        let b = Backend { runner: &r, device: android() };
        b.tap(540, 1070).unwrap();
        b.type_text("hi there & co").unwrap();
        b.swipe((540, 1800), (540, 600), 300).unwrap();
        b.key("back").unwrap();
        b.launch("com.example").unwrap();
        let calls = r.calls.lock().unwrap();
        assert_eq!(calls[0], "adb -s emulator-5554 shell input tap 540 1070");
        assert_eq!(calls[1], "adb -s emulator-5554 shell input text hi%sthere%s\\&%sco");
        assert_eq!(calls[2], "adb -s emulator-5554 shell input swipe 540 1800 540 600 300");
        assert_eq!(calls[3], "adb -s emulator-5554 shell input keyevent KEYCODE_BACK");
        assert!(calls[4].contains("monkey -p com.example"));
    }

    #[test]
    fn ios_command_lines() {
        let r = FakeRunner::default();
        let b = Backend {
            runner: &r,
            device: DeviceInfo { id: "ABC".into(), platform: Platform::Ios, name: "iPhone".into() },
        };
        b.tap(195, 282).unwrap();
        b.swipe((0, 0), (0, 100), 500).unwrap();
        b.launch("com.apple.Preferences").unwrap();
        let calls = r.calls.lock().unwrap();
        assert_eq!(calls[0], "idb ui tap 195 282 --udid ABC");
        assert_eq!(calls[1], "idb ui swipe 0 0 0 100 --duration 0.50 --udid ABC");
        assert_eq!(calls[2], "xcrun simctl launch ABC com.apple.Preferences");
        assert!(b.key("back").is_err(), "iOS has no back key");
    }

    #[test]
    fn parses_device_lists() {
        let adb = "List of devices attached\nemulator-5554          device product:sdk model:Pixel_8 device:emu\nR58M       unauthorized usb:1-1\n\n";
        let d = parse_adb_devices(adb);
        assert_eq!(
            d,
            vec![DeviceInfo {
                id: "emulator-5554".into(),
                platform: Platform::Android,
                name: "Pixel 8".into()
            }]
        );
        let simctl = r#"{"devices":{"com.apple.CoreSimulator.SimRuntime.iOS-26-0":[{"udid":"ABC","name":"iPhone 17","state":"Booted"},{"udid":"DEF","name":"iPad","state":"Shutdown"}]}}"#;
        assert_eq!(parse_simctl_devices(simctl).len(), 1);
    }

    #[test]
    fn choosing_a_device() {
        let one = vec![android()];
        assert_eq!(choose(&one, None).unwrap().id, "emulator-5554");
        assert!(matches!(choose(&[], None), Err(DeviceError::NoDevice)));
        let two = vec![android(), DeviceInfo { id: "x".into(), ..android() }];
        assert!(matches!(choose(&two, None), Err(DeviceError::Ambiguous(_))));
        assert!(matches!(choose(&two, Some("nope")), Err(DeviceError::UnknownDevice(_))));
    }
}
