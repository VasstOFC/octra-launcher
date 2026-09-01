pub mod engine;

use std::path::PathBuf;

use engine::InstallOptions;
use serde::{Deserialize, Serialize};

pub const APP_DISPLAY_NAME: &str = "Octra App";
pub const APP_EXECUTABLE: &str = "Octra App.exe";
pub const INSTALLER_EXECUTABLE: &str = "Octra Setup.exe";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
	pub step: String,
	pub progress: f32,
	pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallRequest {
	pub install_dir: PathBuf,
	pub desktop_shortcut: bool,
	pub launch_after: bool,
}

#[tauri::command]
pub fn default_install_dir() -> PathBuf {
	engine::default_install_dir()
}

#[tauri::command]
pub async fn run_install(
	app: tauri::AppHandle,
	request: InstallRequest,
) -> Result<(), String> {
	let options = InstallOptions {
		install_dir: request.install_dir,
		desktop_shortcut: request.desktop_shortcut,
		launch_after: request.launch_after,
	};

	engine::run_install(&app, options).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub fn launch_installed_app(install_dir: PathBuf) -> Result<(), String> {
	engine::launch_app(&install_dir).map_err(|error| error.to_string())
}

pub fn try_uninstall_from_cli() -> Result<bool, String> {
	let mut args = std::env::args().skip(1);
	let Some(flag) = args.next() else {
		return Ok(false);
	};
	if flag != "--uninstall" {
		return Ok(false);
	}

	#[cfg(not(windows))]
	{
		return Err("uninstall is only supported on Windows".to_owned());
	}

	#[cfg(windows)]
	{
		let install_dir = args.next().ok_or_else(|| {
			"missing install directory argument for --uninstall".to_owned()
		})?;
		engine::uninstall(PathBuf::from(install_dir))
			.map_err(|error| error.to_string())?;
		Ok(true)
	}
}
