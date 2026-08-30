use std::env;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo = manifest.parent().unwrap();
    let pkg = repo.join("package.json");
    println!("cargo:rerun-if-changed={}", pkg.display());
    println!(
        "cargo:rerun-if-changed={}",
        repo.join("src-tauri/icons/icon.ico").display()
    );
    println!("cargo:rerun-if-changed={}", manifest.join("assets/fonts/figtree-400.ttf").display());
    println!("cargo:rerun-if-changed={}", manifest.join("assets/fonts/figtree-600.ttf").display());

    let version = read_version(&pkg).unwrap_or_else(|| env!("CARGO_PKG_VERSION").into());
    println!("cargo:rustc-env=LUMEN_VERSION={version}");

    let parts: Vec<&str> = version.split('.').collect();
    let major: u16 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u16 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch: u16 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    #[cfg(windows)]
    {
        let icon = repo.join("src-tauri/icons/icon.ico");
        let mut res = winres::WindowsResource::new();
        if icon.is_file() {
            res.set_icon(icon.to_str().unwrap());
        }
        res.set("ProductName", "Octra App");
        res.set("FileDescription", "Instalator Octra");
        res.set("CompanyName", "Octra App");
        res.set("LegalCopyright", "Octra App");
        res.set("OriginalFilename", "Octra-setup.exe");
        res.set("InternalName", "Octra App");
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, packed(major, minor, patch));
        res.set_version_info(winres::VersionInfo::FILEVERSION, packed(major, minor, patch));
        res.set("ProductVersion", &version);
        res.set("FileVersion", &version);
        if let Err(e) = res.compile() {
            println!("cargo:warning=winres: {e}");
        }
    }
}

fn packed(major: u16, minor: u16, patch: u16) -> u64 {
    ((major as u64) << 48) | ((minor as u64) << 32) | ((patch as u64) << 16)
}

fn read_version(pkg: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(pkg).ok()?;
    let v = serde_json::from_str::<serde_json::Value>(&text).ok()?;
    v.get("version")?.as_str().map(|s| s.to_string())
}
