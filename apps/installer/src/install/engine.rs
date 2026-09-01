use std::fs::{self, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use std::process::Command;

use tauri::{AppHandle, Emitter, Manager};
use thiserror::Error;
use zip::ZipArchive;

use super::{APP_DISPLAY_NAME, APP_EXECUTABLE, INSTALLER_EXECUTABLE, InstallProgress};

const REGISTRY_KEY: &str = "OctraApp";

#[derive(Debug, Clone)]
pub struct InstallOptions {
	pub install_dir: PathBuf,
	pub desktop_shortcut: bool,
	pub launch_after: bool,
}

#[derive(Debug, Error)]
pub enum InstallError {
	#[error("payload not found: {0}")]
	PayloadMissing(String),
	#[error("{0}")]
	Io(#[from] std::io::Error),
	#[error("{0}")]
	Zip(#[from] zip::result::ZipError),
	#[error("{0}")]
	Other(String),
}

pub fn default_install_dir() -> PathBuf {
	directories::BaseDirs::new()
		.map(|dirs| {
			dirs.home_dir()
				.join("AppData/Local/Programs")
				.join(APP_DISPLAY_NAME)
		})
		.unwrap_or_else(|| PathBuf::from(r"C:\Octra App"))
}

pub async fn run_install(app: &AppHandle, options: InstallOptions) -> Result<(), InstallError> {
	let emit = |step: &str, progress: f32, message: &str| {
		let _ = app.emit(
			"install-progress",
			InstallProgress {
				step: step.to_owned(),
				progress,
				message: message.to_owned(),
			},
		);
	};

	emit("prepare", 0.05, "Przygotowywanie instalacji…");
	cleanup_legacy_installs()?;
	kill_running_apps()?;

	emit("extract", 0.2, "Kopiowanie plików…");
	let payload = resolve_payload(app)?;
	let install_dir = options.install_dir.clone();
	tokio::task::spawn_blocking(move || extract_payload(&payload, &install_dir))
		.await
		.map_err(|error| InstallError::Other(format!("install task failed: {error}")))??;

	emit("shortcuts", 0.75, "Tworzenie skrótów…");
	copy_uninstaller(app, &options.install_dir)?;
	create_shortcuts(&options)?;

	emit("registry", 0.9, "Rejestrowanie instalacji…");
	register_uninstall(&options.install_dir)?;

	emit("done", 1.0, "Instalacja zakończona");
	if options.launch_after {
		launch_app(&options.install_dir)?;
	}

	Ok(())
}

pub fn launch_app(install_dir: &Path) -> Result<(), InstallError> {
	let exe = install_dir.join(APP_EXECUTABLE);
	if !exe.is_file() {
		return Err(InstallError::Other(format!(
			"application binary not found at {}",
			exe.display()
		)));
	}

	Command::new(exe)
		.current_dir(install_dir)
		.spawn()
		.map_err(|error| InstallError::Other(format!("failed to launch app: {error}")))?;
	Ok(())
}

pub fn uninstall(install_dir: PathBuf) -> Result<(), InstallError> {
	remove_shortcuts()?;
	if install_dir.exists() {
		fs::remove_dir_all(&install_dir)?;
	}
	remove_uninstall_registry()?;
	Ok(())
}

fn resolve_payload(app: &AppHandle) -> Result<PathBuf, InstallError> {
	if let Ok(path) = std::env::var("OCTRA_INSTALLER_PAYLOAD") {
		let path = PathBuf::from(path);
		if path.is_file() {
			return Ok(path);
		}
	}

	let resource = app
		.path()
		.resource_dir()
		.map_err(|error| InstallError::Other(error.to_string()))?
		.join("payload/app.zip");
	if resource.is_file() {
		return Ok(resource);
	}

	let dev_candidates = [
		PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("../../target/release")
			.join(APP_EXECUTABLE),
		PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("../app/target/release")
			.join(APP_EXECUTABLE),
	];
	for dev_exe in dev_candidates {
		if dev_exe.is_file() {
			let temp_zip = std::env::temp_dir().join("octra-dev-payload.zip");
			create_zip_from_exe(&dev_exe, &temp_zip)?;
			return Ok(temp_zip);
		}
	}

	Err(InstallError::PayloadMissing(
		"brak paczki instalacyjnej (app.zip). Zbuduj najpierw Octra App.".into(),
	))
}

fn create_zip_from_exe(exe: &Path, zip_path: &Path) -> Result<(), InstallError> {
	use zip::write::SimpleFileOptions;
	use zip::ZipWriter;

	let file = File::create(zip_path)?;
	let mut zip = ZipWriter::new(file);
	let options =
		SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
	zip.start_file(APP_EXECUTABLE, options)?;
	let mut source = File::open(exe)?;
	copy(&mut source, &mut zip)?;
	zip.finish()?;
	Ok(())
}

fn extract_payload(payload: &Path, install_dir: &Path) -> Result<(), InstallError> {
	if install_dir.exists() {
		fs::remove_dir_all(install_dir)?;
	}
	fs::create_dir_all(install_dir)?;

	let file = File::open(payload)?;
	let mut archive = ZipArchive::new(file)?;
	for index in 0..archive.len() {
		let mut entry = archive.by_index(index)?;
		let outpath = match entry.enclosed_name() {
			Some(path) => install_dir.join(path),
			None => continue,
		};

		if entry.name().ends_with('/') {
			fs::create_dir_all(&outpath)?;
			continue;
		}

		if let Some(parent) = outpath.parent() {
			fs::create_dir_all(parent)?;
		}

		let mut outfile = File::create(&outpath)?;
		copy(&mut entry, &mut outfile)?;
	}
	Ok(())
}

fn copy_uninstaller(_app: &AppHandle, install_dir: &Path) -> Result<(), InstallError> {
	let current_exe = std::env::current_exe()?;
	let target = install_dir.join(INSTALLER_EXECUTABLE);
	fs::copy(current_exe, target)?;
	Ok(())
}

fn create_shortcuts(options: &InstallOptions) -> Result<(), InstallError> {
	#[cfg(windows)]
	{
		let exe = options.install_dir.join(APP_EXECUTABLE);
		create_windows_shortcut(
			&start_menu_programs_dir()?.join(format!("{APP_DISPLAY_NAME}.lnk")),
			&exe,
			&options.install_dir,
		)?;
		if options.desktop_shortcut {
			create_windows_shortcut(
				&desktop_dir()?.join(format!("{APP_DISPLAY_NAME}.lnk")),
				&exe,
				&options.install_dir,
			)?;
		}
	}
	#[cfg(not(windows))]
	{
		let _ = options;
	}
	Ok(())
}

fn remove_shortcuts() -> Result<(), InstallError> {
	#[cfg(windows)]
	{
		let _ = fs::remove_file(start_menu_programs_dir()?.join(format!("{APP_DISPLAY_NAME}.lnk")));
		let _ = fs::remove_file(desktop_dir()?.join(format!("{APP_DISPLAY_NAME}.lnk")));
		let _ = fs::remove_file(start_menu_programs_dir()?.join("Octra Launcher.lnk"));
		let _ = fs::remove_file(desktop_dir()?.join("Octra Launcher.lnk"));
	}
	Ok(())
}

fn register_uninstall(install_dir: &Path) -> Result<(), InstallError> {
	#[cfg(windows)]
	{
		use winreg::RegKey;
		use winreg::enums::*;

		let hkcu = RegKey::predef(HKEY_CURRENT_USER);
		let (key, _) = hkcu.create_subkey(format!(
			r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{REGISTRY_KEY}"
		))?;
		let uninstaller = install_dir.join(INSTALLER_EXECUTABLE);
		let app_exe = install_dir.join(APP_EXECUTABLE);
		key.set_value("DisplayName", &APP_DISPLAY_NAME)?;
		key.set_value(
			"UninstallString",
			&format!(
				"\"{}\" --uninstall \"{}\"",
				uninstaller.display(),
				install_dir.display()
			),
		)?;
		key.set_value("InstallLocation", &install_dir.to_string_lossy().to_string())?;
		key.set_value("DisplayIcon", &app_exe.to_string_lossy().to_string())?;
		key.set_value("Publisher", &"Octra")?;
	}
	#[cfg(not(windows))]
	{
		let _ = install_dir;
	}
	Ok(())
}

fn remove_uninstall_registry() -> Result<(), InstallError> {
	#[cfg(windows)]
	{
		use winreg::RegKey;
		use winreg::enums::*;

		let hkcu = RegKey::predef(HKEY_CURRENT_USER);
		let _ = hkcu.delete_subkey_all(format!(
			r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{REGISTRY_KEY}"
		));
	}
	Ok(())
}

fn cleanup_legacy_installs() -> Result<(), InstallError> {
	#[cfg(windows)]
	{
		let _ = fs::remove_file(desktop_dir()?.join("Octra Launcher.lnk"));
		let _ = fs::remove_file(start_menu_programs_dir()?.join("Octra Launcher.lnk"));
		let legacy_dir = directories::BaseDirs::new()
			.map(|dirs| dirs.home_dir().join("AppData/Local/Octra Launcher"))
			.unwrap_or_default();
		if legacy_dir.is_dir() {
			let _ = fs::remove_dir_all(legacy_dir);
		}
	}
	Ok(())
}

fn kill_running_apps() -> Result<(), InstallError> {
	#[cfg(windows)]
	{
		for process in ["Octra App.exe", "Octra Launcher.exe", "octra-launcher.exe"] {
			let _ = Command::new("taskkill").args(["/F", "/IM", process]).status();
		}
	}
	Ok(())
}

#[cfg(windows)]
fn desktop_dir() -> Result<PathBuf, InstallError> {
	directories::UserDirs::new()
		.and_then(|dirs| dirs.desktop_dir().map(|path| path.to_path_buf()))
		.ok_or_else(|| InstallError::Other("desktop folder not found".into()))
}

#[cfg(windows)]
fn start_menu_programs_dir() -> Result<PathBuf, InstallError> {
	directories::BaseDirs::new()
		.map(|dirs| {
			dirs.home_dir()
				.join("AppData/Roaming/Microsoft/Windows/Start Menu/Programs")
		})
		.ok_or_else(|| InstallError::Other("start menu folder not found".into()))
}

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
fn create_windows_shortcut(
	shortcut_path: &Path,
	target_path: &Path,
	working_dir: &Path,
) -> Result<(), InstallError> {
	use windows::Win32::System::Com::{
		CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
		CoInitializeEx, CoUninitialize, IPersistFile,
	};
	use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
	use windows::core::{Interface, PCWSTR};

	let shortcut_path = wide_path(shortcut_path);
	let target_path = wide_path(target_path);
	let working_dir = wide_path(working_dir);

	unsafe {
		let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
		let result = (|| {
			let shortcut: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
				.map_err(|error| InstallError::Other(error.to_string()))?;
			shortcut
				.SetPath(PCWSTR::from_raw(target_path.as_ptr()))
				.map_err(|error| InstallError::Other(error.to_string()))?;
			shortcut
				.SetWorkingDirectory(PCWSTR::from_raw(working_dir.as_ptr()))
				.map_err(|error| InstallError::Other(error.to_string()))?;
			shortcut
				.SetIconLocation(PCWSTR::from_raw(target_path.as_ptr()), 0)
				.map_err(|error| InstallError::Other(error.to_string()))?;
			let persist: IPersistFile = shortcut.cast().map_err(|error| InstallError::Other(error.to_string()))?;
			persist
				.Save(PCWSTR::from_raw(shortcut_path.as_ptr()), true)
				.map_err(|error| InstallError::Other(error.to_string()))?;
			Ok(())
		})();
		CoUninitialize();
		result
	}
}

#[cfg(windows)]
fn wide_path(path: &Path) -> Vec<u16> {
	path.as_os_str()
		.encode_wide()
		.chain(std::iter::once(0))
		.collect()
}
