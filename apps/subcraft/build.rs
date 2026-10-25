//! Embeds the application icon and version info into the Windows executable.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=../../assets/branding/SkinStrike.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        let icon = if std::env::var_os("CARGO_FEATURE_SKINSTRIKE_CANDIDATE").is_some() {
            "../../assets/branding/SkinStrike.ico"
        } else {
            "assets/icon.ico"
        };
        res.set_icon(icon).set("ProductName", "SubCraft").set("FileDescription", "SubCraft texture painter");
        // The approved candidate must not silently ship without its Windows icon.
        // Feature-off donor builds retain their existing missing-SDK warning behavior.
        if let Err(e) = res.compile() {
            if std::env::var_os("CARGO_FEATURE_SKINSTRIKE_CANDIDATE").is_some() {
                eprintln!("embedding SkinStrike Windows icon failed: {e}");
                std::process::exit(1);
            }
            println!("cargo:warning=could not embed the Windows icon: {e}");
        }
    }
}
