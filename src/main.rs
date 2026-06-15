#[cfg(windows)]
mod windows_app;

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    windows_app::run()
}

#[cfg(not(windows))]
fn main() {
    eprintln!(
        "forza-key-overlay is a Windows overlay. Build for x86_64-pc-windows-gnu to create the .exe."
    );
}
