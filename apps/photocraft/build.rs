//! Windows only: embed the app icon and version info (VERSIONINFO) into `photocraft.exe`.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `PHOTOCRAFT_REQUIRE_WINRES=1` (set by the
//! release workflow) turns it into an error.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/photocraft.ico");
    println!("cargo:rerun-if-env-changed=PHOTOCRAFT_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    // Lumen 0.0.3 (MAJOR << 48 | MINOR << 32 | PATCH << 16): Lumen's own version, not the workspace's.
    let lumen_version: u64 = 3 << 16;
    res.set_version_info(winresource::VersionInfo::FILEVERSION, lumen_version)
        .set_version_info(winresource::VersionInfo::PRODUCTVERSION, lumen_version)
        .set("FileVersion", "0.0.3")
        .set("ProductVersion", "0.0.3")
        .set("Comments", "https://sifg.net");
    res.set_icon("../../assets/app-icon/photocraft.ico")
        // Lumen for PC is built on PhotoCraft (MIT OR Apache-2.0); its notice stays here.
        .set("ProductName", "Lumen Photo Editor")
        .set("FileDescription", "Lumen Photo Editor")
        .set("CompanyName", "SifG22")
        .set("LegalCopyright", "Copyright (c) SifG22. Based on PhotoCraft, copyright (c) the PhotoCraft authors, MIT OR Apache-2.0.")
        .set("OriginalFilename", "Lumen.exe")
        .set("InternalName", "Lumen");
    if let Err(e) = res.compile() {
        if std::env::var_os("PHOTOCRAFT_REQUIRE_WINRES").is_some() {
            panic!("embedding Windows resources failed: {e}");
        }
        println!("cargo:warning=photocraft.exe built without icon/version resources: {e}");
    }
}
