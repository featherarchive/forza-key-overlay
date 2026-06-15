# Forza Key Overlay

Portable Windows overlay that shows keyboard input for driving games.

## What It Shows

- `W`
- `A`
- `S`
- `D`
- `SPACE`
- `SHIFT`
- `CTRL`

The overlay uses a left-side stacked layout with a transparent Glass Edge key style. Pressed keys glow brighter.

## How To Run

On Windows, run:

```powershell
forza-key-overlay.exe
```

Press `U` to hide or show the overlay. The app keeps running while hidden, so pressing `U` again shows it.

Press `M` to enter or exit move mode. While move mode is on, drag the overlay to reposition it. Press `M` again before driving so the overlay goes back to click-through mode.

The overlay is always-on-top and normally click-through, so it should sit above normal windows without stealing mouse input outside move mode.

## Game Display Mode

Use Forza in borderless windowed or windowed fullscreen mode for best results.

Exclusive fullscreen games can block ordinary always-on-top overlays, so if the overlay does not appear above the game, switch the game display mode first.

## Build From Linux

Run:

```bash
./scripts/build-windows.sh
```

Expected output:

```text
dist/forza-key-overlay.exe
```

The Linux build uses `cargo-xwin` and the `x86_64-pc-windows-msvc` target so it does not require a system MinGW linker.

If local cross-compilation fails, use the GitHub Actions workflow. It builds on `windows-latest` and uploads `forza-key-overlay.exe` as an artifact named `forza-key-overlay-windows`.

## Development Checks

Run:

```bash
cargo fmt --check
cargo test
cargo check
cargo check --target x86_64-pc-windows-gnu
cargo xwin build --release --target x86_64-pc-windows-msvc
```
