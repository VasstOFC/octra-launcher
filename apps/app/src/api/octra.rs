use crate::api::Result;
use serde::{Deserialize, Serialize};
use theseus::octra_sync::{self, OctraServerEntry};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("octra")
        .invoke_handler(tauri::generate_handler![list_servers])
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
