use std::io;
#[cfg(windows)]
use winres::WindowsResource;

fn main() -> io::Result<()> { #[cfg(windows)]
    {
        WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            // .set_icon("src/assets/logo.ico")
            .set_manifest(
                r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="asInvoker" uiAccess="false" />
            </requestedPrivileges>
        </security>
    </trustInfo>
</assembly>
            "#,
            )
            .set(
                "FileDescription",
                "Simple Rust Application to Compare and Get Differentioal from 2 File Excel or Csv",
            )
            .set_language(winapi::um::winnt::MAKELANGID(
                winapi::um::winnt::LANG_INDONESIAN,
                winapi::um::winnt::SUBLANG_INDONESIAN_INDONESIA,
            ))
            .set(
                "LegalCopyright",
                "Copyright 2022 RizalAchp, All rights reserved",
            )
            .set(
                "Comments",
                "Designed and Optimized for Excel or Csv text processing ",
            )
            .compile()?;
    }
    Ok(())
}
