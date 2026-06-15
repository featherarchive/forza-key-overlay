# Windows Key Overlay Design

## Goal

Build a portable Windows overlay app that can be sent to a friend as a single executable. The app displays currently pressed driving keys in a visible overlay above all windows, primarily for Forza 6 but not tied to the game process.

## Target Platform

- Windows 10 and Windows 11.
- Portable `.exe`; no installer.
- Built from this Linux workspace using Rust cross-compilation when available.

## Overlay Behavior

- The overlay is always on top of other windows.
- The overlay is transparent outside the key visuals.
- The overlay is click-through so it does not steal mouse input from the game.
- The overlay is not Forza-specific and remains above any active app.
- The `U` key toggles overlay visibility globally.
- The app keeps running when hidden so pressing `U` again can show it.

Exclusive fullscreen games can block ordinary always-on-top overlays. The intended operating mode is borderless windowed or windowed fullscreen.

## Displayed Keys

The overlay shows:

- `W`
- `A`
- `S`
- `D`
- `SPACE`
- `SHIFT`
- `CTRL`

The layout is the selected Left Stack arrangement:

- WASD cluster at the top.
- Space below the WASD cluster.
- Shift and Ctrl below Space.

The selected visual style is Glass Edge:

- Semi-transparent key backgrounds.
- Bright, thin glass-like borders.
- Soft glow around pressed or active keys.
- Released keys remain visible but subdued.

## Input Tracking

The app uses Windows keyboard state APIs to read global key state without requiring game focus. The input loop polls the target keys at a steady interval and updates the overlay state when values change.

`U` acts as a toggle key, not a displayed key. Holding `U` should not rapidly flicker visibility; the app toggles once per key press.

## Architecture

The app is a Rust binary with two layers:

1. `overlay_state`: platform-independent key state and toggle logic, covered by Linux-runnable unit tests.
2. Windows UI layer: Win32 window creation, transparency, always-on-top behavior, click-through flags, key polling, and drawing.

The UI layer creates a borderless layered window and paints custom keycaps directly. The app does not depend on a browser runtime, Python, Electron, or an installer.

## Packaging

Primary output:

- `target/x86_64-pc-windows-gnu/release/forza-key-overlay.exe`

Supporting scripts:

- A build script to cross-compile the release executable from Linux.
- A README with run instructions and the borderless/windowed fullscreen note.

If the required Windows Rust target or linker is unavailable locally, the project will still include clear build instructions and a GitHub Actions workflow for producing the `.exe`.

## Testing

Linux-runnable tests cover:

- Pressed/released key state updates.
- `U` toggles visibility once per press.
- Holding `U` does not repeatedly toggle.
- Releasing and pressing `U` again toggles again.

Windows-specific overlay behavior cannot be fully validated from Linux. Verification from Linux is limited to compiling, unit tests, and static checks. Runtime validation should be done on a Windows machine.
