use crate::api::Result;
use serde::{Deserialize, Serialize};
use theseus::octra_accounts::{self, OctraAccountSession, OctraCommunitySnapshot};
use theseus::octra_sync::{self, OctraServerEntry};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
	tauri::plugin::Builder::new("octra")
		.invoke_handler(tauri::generate_handler![
			list_servers,
			octra_account_session,
			octra_account_register,
			octra_account_login,
			octra_account_logout,
			octra_community,
		])
		.build()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListedServer {
	pub name: String,
	pub address: String,
}

#[tauri::command]
pub fn list_servers() -> Result<Vec<ListedServer>> {
	Ok(octra_sync::list_octra_servers()
		.into_iter()
		.map(|OctraServerEntry { name, address }| ListedServer {
			name,
			address,
		})
		.collect())
}

#[tauri::command]
pub async fn octra_account_session() -> Result<Option<OctraAccountSession>> {
	Ok(octra_accounts::session().await?)
}

#[tauri::command]
pub async fn octra_account_register(password: &str) -> Result<OctraAccountSession> {
	Ok(octra_accounts::register(password).await?)
}

#[tauri::command]
pub async fn octra_account_login(
	username: &str,
	password: &str,
) -> Result<OctraAccountSession> {
	Ok(octra_accounts::login(username, password).await?)
}

#[tauri::command]
pub async fn octra_account_logout() -> Result<()> {
	octra_accounts::logout().await?;
	Ok(())
}

#[tauri::command]
pub async fn octra_community() -> Result<OctraCommunitySnapshot> {
	Ok(octra_accounts::community().await?)
}
