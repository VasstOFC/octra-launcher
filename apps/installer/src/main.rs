#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]
#![allow(unexpected_cfgs)]

mod install;

use tauri::Manager;

fn main() {
	tracing_subscriber::fmt()
		.with_env_filter("info")
		.with_target(false)
		.init();

	if install::try_uninstall_from_cli().unwrap_or(false) {
		return;
	}

	let mode = install::parse_update_request()
		.map(|install_dir| install::InstallerMode::Update { install_dir })
		.unwrap_or(install::InstallerMode::Fresh);

	tauri::Builder::default()
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_opener::init())
		.manage(mode)
		.invoke_handler(tauri::generate_handler![
			install::default_install_dir,
			install::run_install,
			install::launch_installed_app,
			install::installer_mode,
			install::run_update,
		])
		.setup(|app| {
			let window = app.get_webview_window("main").expect("main window");
			let _ = window.show();
			let _ = window.set_focus();
			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running Octra installer");
}
