use std::io;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        // Only run for the installer, skip for the main executable if possible,
        // but since we only have one build.rs, it applies to all bins unless filtered.
        // We can just add the icon to all executables in the crate.
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/vice_icon.ico");
        res.compile()?;
    }
    Ok(())
}
