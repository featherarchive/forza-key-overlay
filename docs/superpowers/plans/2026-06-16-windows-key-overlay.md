# Windows Key Overlay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a portable Windows `.exe` that shows a transparent always-on-top keyboard overlay and toggles visibility with `U`.

**Architecture:** Create a small Rust binary with platform-independent overlay state in a library module and a Windows-only Win32 UI executable. Linux tests cover key state and toggle behavior; Windows builds compile the overlay window, drawing, global key polling, and packaging.

**Tech Stack:** Rust 2024 edition, `windows` crate for Win32 APIs, Cargo unit tests, `x86_64-pc-windows-gnu` target, GitHub Actions fallback build.

---

## File Structure

- `Cargo.toml`: Rust package metadata, binary/lib targets, `windows` dependency behind Windows target config.
- `src/lib.rs`: Exports platform-independent modules for tests.
- `src/overlay_state.rs`: Key enum, snapshot type, visibility toggle debounce logic.
- `src/main.rs`: Windows entry point and non-Windows stub message.
- `src/windows_app.rs`: Win32 overlay window, timer loop, keyboard polling, layered/click-through/topmost styles, GDI drawing.
- `README.md`: Build, run, and Windows behavior notes.
- `scripts/build-windows.sh`: Linux cross-compile helper.
- `.github/workflows/windows-release.yml`: CI workflow that builds the Windows executable.

## Task 1: Scaffold Rust Package

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`

- [ ] **Step 1: Create minimal package files**

```toml
# Cargo.toml
[package]
name = "forza-key-overlay"
version = "0.1.0"
edition = "2024"

[lib]
path = "src/lib.rs"

[[bin]]
name = "forza-key-overlay"
path = "src/main.rs"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_Graphics_Gdi",
  "Win32_System_LibraryLoader",
  "Win32_UI_Input_KeyboardAndMouse",
  "Win32_UI_WindowsAndMessaging"
] }
```

```rust
// src/lib.rs
pub mod overlay_state;
```

```rust
// src/main.rs
#[cfg(windows)]
mod windows_app;

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    windows_app::run()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("forza-key-overlay is a Windows overlay. Build for x86_64-pc-windows-gnu to create the .exe.");
}
```

- [ ] **Step 2: Run format/check**

Run: `cargo fmt --check && cargo check`

Expected: `cargo check` fails because `src/overlay_state.rs` is not created yet. This verifies the scaffold references the intended module.

- [ ] **Step 3: Commit**

Run: `git add Cargo.toml src/lib.rs src/main.rs && git commit -m "chore: scaffold Rust overlay package"`

## Task 2: Add Overlay State with TDD

**Files:**
- Create: `src/overlay_state.rs`

- [ ] **Step 1: Write failing tests**

```rust
// src/overlay_state.rs
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DisplayKey {
    W,
    A,
    S,
    D,
    Space,
    Shift,
    Ctrl,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_pressed_keys_from_snapshot() {
        let mut overlay = OverlayState::new();

        overlay.update_keys(KeySnapshot::from_pressed([DisplayKey::W, DisplayKey::Space]));

        assert!(overlay.is_pressed(DisplayKey::W));
        assert!(overlay.is_pressed(DisplayKey::Space));
        assert!(!overlay.is_pressed(DisplayKey::A));
    }

    #[test]
    fn toggles_visibility_once_per_u_press() {
        let mut overlay = OverlayState::new();

        overlay.update_toggle_key(true);
        overlay.update_toggle_key(true);
        assert!(!overlay.visible());

        overlay.update_toggle_key(false);
        overlay.update_toggle_key(true);
        assert!(overlay.visible());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test overlay_state`

Expected: FAIL with missing `OverlayState` and `KeySnapshot` types.

- [ ] **Step 3: Implement minimal state logic**

```rust
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DisplayKey {
    W,
    A,
    S,
    D,
    Space,
    Shift,
    Ctrl,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KeySnapshot {
    pressed: HashSet<DisplayKey>,
}

impl KeySnapshot {
    pub fn from_pressed<const N: usize>(keys: [DisplayKey; N]) -> Self {
        Self {
            pressed: keys.into_iter().collect(),
        }
    }

    pub fn is_pressed(&self, key: DisplayKey) -> bool {
        self.pressed.contains(&key)
    }
}

#[derive(Clone, Debug)]
pub struct OverlayState {
    visible: bool,
    toggle_was_down: bool,
    keys: KeySnapshot,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayState {
    pub fn new() -> Self {
        Self {
            visible: true,
            toggle_was_down: false,
            keys: KeySnapshot::default(),
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn update_toggle_key(&mut self, is_down: bool) {
        if is_down && !self.toggle_was_down {
            self.visible = !self.visible;
        }
        self.toggle_was_down = is_down;
    }

    pub fn update_keys(&mut self, snapshot: KeySnapshot) {
        self.keys = snapshot;
    }

    pub fn is_pressed(&self, key: DisplayKey) -> bool {
        self.keys.is_pressed(key)
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test`

Expected: PASS.

- [ ] **Step 5: Commit**

Run: `git add src/overlay_state.rs && git commit -m "feat: add overlay key state"`

## Task 3: Add Windows Overlay App

**Files:**
- Create: `src/windows_app.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add Windows app implementation**

Implement `windows_app::run()` using:

- `CreateWindowExW` with `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW`.
- `SetLayeredWindowAttributes` with a black color key for transparent background.
- `SetWindowPos(HWND_TOPMOST, ...)` to keep the overlay above normal windows.
- `SetTimer` at 16 ms or 33 ms for key polling and repaint.
- `GetAsyncKeyState` for `W`, `A`, `S`, `D`, `VK_SPACE`, `VK_SHIFT`, `VK_CONTROL`, and `U`.
- `ShowWindow(hwnd, SW_SHOW/SW_HIDE)` for visibility toggling.
- `WM_PAINT` GDI drawing for the Left Stack Glass Edge keys.

The key geometry is:

```rust
const KEY: i32 = 54;
const GAP: i32 = 8;
const SMALL_W: i32 = 84;
const SPACE_W: i32 = KEY * 3 + GAP * 2;
const LEFT: i32 = 24;
const TOP: i32 = 220;
```

- [ ] **Step 2: Cross-check build on Linux**

Run: `cargo check`

Expected: PASS on Linux because `windows_app` is compiled only on Windows.

- [ ] **Step 3: Commit**

Run: `git add src/main.rs src/windows_app.rs && git commit -m "feat: add Windows overlay window"`

## Task 4: Add Build Scripts and CI

**Files:**
- Create: `scripts/build-windows.sh`
- Create: `.github/workflows/windows-release.yml`
- Modify: `.gitignore`

- [ ] **Step 1: Add local build helper**

```bash
#!/usr/bin/env bash
set -euo pipefail

rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
echo "Built target/x86_64-pc-windows-gnu/release/forza-key-overlay.exe"
```

- [ ] **Step 2: Add GitHub Actions workflow**

```yaml
name: Build Windows EXE

on:
  workflow_dispatch:
  push:
    branches: [ master, main ]

jobs:
  windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo build --release
      - uses: actions/upload-artifact@v4
        with:
          name: forza-key-overlay-windows
          path: target/release/forza-key-overlay.exe
```

- [ ] **Step 3: Update `.gitignore`**

Ensure it contains:

```gitignore
.superpowers/
target/
```

- [ ] **Step 4: Run verification**

Run: `chmod +x scripts/build-windows.sh && cargo test`

Expected: PASS.

- [ ] **Step 5: Commit**

Run: `git add .github/workflows/windows-release.yml scripts/build-windows.sh .gitignore && git commit -m "chore: add Windows build automation"`

## Task 5: Add README and Final Verification

**Files:**
- Create: `README.md`

- [ ] **Step 1: Add README**

Document:

- The app is Windows-only at runtime.
- Run `forza-key-overlay.exe`.
- Press `U` to hide/show.
- Use Forza in borderless/windowed fullscreen for best overlay compatibility.
- Build locally with `scripts/build-windows.sh`.
- If local cross-compile fails, use the GitHub Actions artifact.

- [ ] **Step 2: Run final verification**

Run:

```bash
cargo fmt --check
cargo test
cargo check
```

Expected: all PASS.

Then try:

```bash
./scripts/build-windows.sh
```

Expected if MinGW linker is installed: creates `target/x86_64-pc-windows-gnu/release/forza-key-overlay.exe`.

Expected if MinGW linker is missing: fail with a linker/toolchain error; README and GitHub Actions remain the supported fallback for producing the `.exe`.

- [ ] **Step 3: Commit**

Run: `git add README.md && git commit -m "docs: add overlay usage instructions"`

## Self-Review

- Spec coverage: The plan covers portable `.exe`, Rust/Win32, always-on-top, transparent click-through overlay, Left Stack, Glass Edge, displayed keys, `U` toggle debounce, Linux tests, build script, README, and CI fallback.
- Placeholder scan: No unfinished placeholder markers are used.
- Type consistency: `DisplayKey`, `KeySnapshot`, and `OverlayState` names are introduced in Task 2 and used consistently by later tasks.
