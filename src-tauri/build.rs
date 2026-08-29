fn main() {
    println!("cargo:rerun-if-env-changed=LUMEN_CHANNEL");
    let ch = std::env::var("LUMEN_CHANNEL").unwrap_or_default();
    println!("cargo:rustc-env=LUMEN_COMPILED_CHANNEL={ch}");
    tauri_build::build();
}
