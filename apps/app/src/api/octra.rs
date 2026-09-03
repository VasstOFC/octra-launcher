use crate::api::Result;
use serde::{Deserialize, Serialize};
use theseus::octra_accounts::{
	self, OctraAccountSession, OctraChatChannel, OctraChatMessage, OctraCommunitySnapshot,
	OctraSharedServer,
};
use theseus::octra_sync::{self, OctraServerEntry};
use theseus::pack::featured;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
	tauri::plugin::Builder::new("octra")
		.invoke_handler(tauri::generate_handler![
			list_servers,
			octra_account_session,
			octra_account_register,
			octra_account_login,
			octra_account_logout,
			octra_community,
			octra_chat_channels,
			octra_chat_open_dm,
			octra_chat_create_group,
			octra_chat_add_members,
			octra_chat_list,
			octra_chat_post,
			octra_chat_mark_read,
			octra_chat_delete_message,
			octra_chat_pin_message,
			octra_chat_react_message,
			octra_share_join_address,
			octra_shared_servers_list,
			octra_shared_servers_add,
			octra_shared_servers_delete,
			octra_cache_mrpack_url,
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

#[tauri::command]
pub async fn octra_chat_channels() -> Result<Vec<OctraChatChannel>> {
	Ok(octra_accounts::chat_channels().await?)
}

#[tauri::command]
pub async fn octra_chat_open_dm(user_id: i64) -> Result<OctraChatChannel> {
	Ok(octra_accounts::chat_open_dm(user_id).await?)
}

#[tauri::command]
pub async fn octra_chat_create_group(
	name: &str,
	member_ids: Vec<i64>,
) -> Result<OctraChatChannel> {
	Ok(octra_accounts::chat_create_group(name, &member_ids).await?)
}

#[tauri::command]
pub async fn octra_chat_add_members(
	channel_id: i64,
	member_ids: Vec<i64>,
) -> Result<OctraChatChannel> {
	Ok(octra_accounts::chat_add_group_members(channel_id, &member_ids).await?)
}

#[tauri::command]
pub async fn octra_chat_list(channel_id: i64, after_id: i64) -> Result<Vec<OctraChatMessage>> {
	Ok(octra_accounts::chat_list(channel_id, after_id).await?)
}

#[tauri::command]
pub async fn octra_chat_post(channel_id: i64, text: &str) -> Result<OctraChatMessage> {
	Ok(octra_accounts::chat_post(channel_id, text).await?)
}

#[tauri::command]
pub async fn octra_chat_mark_read(channel_id: i64, last_read_id: i64) -> Result<()> {
	octra_accounts::chat_mark_read(channel_id, last_read_id).await?;
	Ok(())
}

#[tauri::command]
pub async fn octra_chat_delete_message(message_id: i64) -> Result<OctraChatMessage> {
	Ok(octra_accounts::chat_delete_message(message_id).await?)
}

#[tauri::command]
pub async fn octra_chat_pin_message(message_id: i64, pinned: bool) -> Result<OctraChatMessage> {
	Ok(octra_accounts::chat_pin_message(message_id, pinned).await?)
}

#[tauri::command]
pub async fn octra_chat_react_message(
	message_id: i64,
	emoji: &str,
) -> Result<OctraChatMessage> {
	Ok(octra_accounts::chat_react_message(message_id, emoji).await?)
}

#[tauri::command]
pub async fn octra_share_join_address(address: &str) -> Result<()> {
	octra_accounts::share_join_address(address).await?;
	Ok(())
}

#[tauri::command]
pub async fn octra_shared_servers_list() -> Result<Vec<OctraSharedServer>> {
	Ok(octra_accounts::shared_servers_list().await?)
}

#[tauri::command]
pub async fn octra_shared_servers_add(name: &str, address: &str) -> Result<OctraSharedServer> {
	Ok(octra_accounts::shared_servers_add(name, address).await?)
}

#[tauri::command]
pub async fn octra_shared_servers_delete(server_id: i64) -> Result<()> {
	octra_accounts::shared_servers_delete(server_id).await?;
	Ok(())
}

#[tauri::command]
pub async fn octra_cache_mrpack_url(url: &str) -> Result<String> {
	let path = featured::cache_mrpack_from_url(url).await?;
	Ok(path.to_string_lossy().into_owned())
}


