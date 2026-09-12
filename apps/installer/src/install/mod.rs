pub mod engine;

use std::path::PathBuf;

use engine::InstallOptions;
use serde::{Deserialize, Serialize};

pub const APP_DISPLAY_NAME: &str = "Lumen App";
pub const APP_EXECUTABLE: &str = "Lumen App.exe";
pub const INSTALLER_EXECUTABLE: &str = "Lumen Setup.exe";

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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "mode")]
pub enum InstallerMode {
	Fresh,
	Update { install_dir: PathBuf },
}

pub fn parse_update_request() -> Option<PathBuf> {
	let mut args = std::env::args().skip(1);
	while let Some(flag) = args.next() {
		if flag == "--update" {
			return args.next().map(PathBuf::from);
		}
	}
	None
}

#[tauri::command]
pub fn installer_mode(state: tauri::State<InstallerMode>) -> InstallerMode {
	state.inner().clone()
}

#[tauri::command]
pub async fn run_update(app: tauri::AppHandle, install_dir: PathBuf) -> Result<(), String> {
	// Bezpiecznik: aktualizujemy tylko istniejącą instalację.
	// Świeżą instalację obsługuje run_install (tam katalog docelowy może nie istnieć).
	if !install_dir.join(APP_EXECUTABLE).is_file() {
		return Err(format!(
			"nie znaleziono instalacji Lumen App w {} — aktualizacja przerwana",
			install_dir.display()
		));
	}
	let options = InstallOptions {
		install_dir,
		desktop_shortcut: false,
		launch_after: true,
	};
	engine::run_install(&app, options).await.map_err(|error| error.to_string())
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
