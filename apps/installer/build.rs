use std::path::Path;

fn main() {
	let payload = Path::new("resources/payload/app.zip");
	if payload.is_file() {
		println!("cargo:rerun-if-changed=resources/payload/app.zip");
		println!("cargo:rustc-cfg=has_payload");
	}

	tauri_build::build()
}
