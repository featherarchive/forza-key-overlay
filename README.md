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

The overlay uses a left-side stacked layout with a transparent Glass Edge key style. Pressed keys fill in and glow red.

## How To Run

On Windows, run:

```powershell
forza-key-overlay.exe
```

Press `U` to hide or show the overlay. The app keeps running while hidden, so pressing `U` again shows it.

Hold and drag the small dot left of `W` to reposition the overlay.

The overlay is always-on-top and normally click-through, so it should sit above normal windows without stealing mouse input outside the drag dot.

While running, the app appears as `Forza Key Overlay` in the Windows notification area / hidden icons tray. Left-click that icon to open the `Quit` option.

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
