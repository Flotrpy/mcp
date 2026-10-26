# handset

Lets an agent drive Android phones and emulators (over `adb`) and the iOS Simulator (over `idb` and `simctl`) without drowning it in XML.

## Install

```sh
cargo install --git https://github.com/Flotrpy/mcp handset
```

```json
{ "mcpServers": { "handset": { "command": "handset-mcp" } } }
```

Android needs `adb` (Android platform-tools). The iOS Simulator needs Xcode and [idb](https://fbidb.io).

## Tools

| Tool | Purpose |
|---|---|
| `list_devices` | Connected devices and booted Simulators |
| `snapshot` | The screen as one compact line per element |
| `act` | `tap`, `type`, `scroll`, `key` or `launch`, then **only what changed** |
| `wait_for` | Poll until text, a label or a ref appears or disappears |

## What it looks like

```text
e1 text "Welcome back"
e2 input "Email" = "jo@example.com" (focused)
e3 input "Password" = "••••"
e4 checkbox "Remember me" [ ]
e5 button "Sign in" (disabled)
```

That is the whole login screen: **more than 10× smaller** than the raw UI Automator dump. Then:

```text
act {"action": "tap", "ref": "e5"}
→ Tapped e5 "Sign in".
  + e6 text "Inbox"
  - e5 button "Sign in"
```

## What makes it different
- **Ref-tagged compact trees.** Wrapper containers are collapsed into the text they hold, and decorative views are dropped. Passwords are never echoed.
- **Refs stay stable.** An element keeps its ref across snapshots while its role, label and id stay the same, so `e7` still means the same button after a scroll. Identical siblings stay distinct.
- **Change-only replies.** Every action answers with `+` added, `-` removed and `~` changed lines. When most of the screen changes (navigation), the new screen is included automatically.
- **`wait_for`, not sleep.** It polls for text or a ref to appear or disappear, with a timeout.
- **One API for both platforms.** The same four tools work on Android and the iOS Simulator.
- **Recoverable errors.** A stale ref says to take a snapshot, and ambiguous text lists the matching refs.
