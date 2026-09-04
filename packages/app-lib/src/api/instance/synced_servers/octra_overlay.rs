use super::SERVERS_FILE;
use super::codec::{read_servers, server_data, write_servers};
use super::operations::{compose_instance, effective};
use super::super::synced_options::{
    instance_dir, instance_is_running, sync_files_are_protected,
};
use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::{InstanceMetadata, State};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use image::imageops::FilterType;
use quartz_nbt::NbtCompound;
use std::sync::OnceLock;

const NAME_PREFIX: &str = "[Octra] ";
const ICON_SOURCE: &[u8] =
    include_bytes!("../../../../assets/octra-server-icon.png");

fn octra_icon_base64() -> &'static str {
    static ICON: OnceLock<String> = OnceLock::new();
    ICON.get_or_init(|| {
        let image = image::load_from_memory(ICON_SOURCE)
            .unwrap_or_else(|error| {
                panic!("failed to decode Octra server icon: {error}")
            })
            .resize_exact(64, 64, FilterType::Lanczos3);
        let mut png = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut png),
                image::ImageFormat::Png,
            )
            .unwrap_or_else(|error| {
                panic!("failed to encode Octra server icon: {error}")
            });
        BASE64.encode(png)
    })
}

pub(super) fn is_octra_overlay_entry(server: &NbtCompound) -> bool {
    server
        .get::<_, &str>("name")
        .ok()
        .is_some_and(|name| name.starts_with(NAME_PREFIX))
}

pub(super) fn strip_octra_overlay(
    servers: Vec<NbtCompound>,
) -> Vec<NbtCompound> {
    servers
        .into_iter()
        .filter(|server| !is_octra_overlay_entry(server))
        .collect()
}

pub(super) async fn overlay_entries() -> Vec<NbtCompound> {
    let Ok(shared) = crate::octra_accounts::shared_servers_list().await else {
        return Vec::new();
    };
    let icon = octra_icon_base64();
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for server in shared {
        let address = server.address.trim().to_string();
        if address.is_empty() {
            continue;
        }
        let identity = address.to_ascii_lowercase();
        if !seen.insert(identity) {
            continue;
        }
        let display_name = {
            let name = server.name.trim();
            if name.is_empty() {
                address.clone()
            } else {
                name.to_string()
            }
        };
        let mut entry =
            server_data(format!("{NAME_PREFIX}{display_name}"), address, None);
        entry.insert("icon", icon.to_string());
        entries.push(entry);
    }
    entries.sort_by(|left, right| {
        overlay_sort_key(left).cmp(&overlay_sort_key(right))
    });
    entries
}

fn overlay_sort_key(server: &NbtCompound) -> String {
    server
        .get::<_, &str>("name")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

pub(super) async fn prepend_overlay(
    servers: Vec<NbtCompound>,
) -> Vec<NbtCompound> {
    let overlay = overlay_entries().await;
    if overlay.is_empty() {
        return strip_octra_overlay(servers);
    }
    let mut combined = overlay;
    combined.extend(strip_octra_overlay(servers));
    combined
}

/// Refresh the Octra shared-server overlay in every installed instance's
/// `servers.dat` (skipped while that instance is running).
pub async fn refresh_octra_shared_servers_overlay() -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    for metadata in crate::state::list_instances(&state.pool).await? {
        if let Err(error) =
            refresh_instance_overlay_locked(&metadata, &state).await
        {
            tracing::warn!(
                "Failed to refresh Octra shared servers for {}: {error}",
                metadata.instance.id
            );
        }
    }
    Ok(())
}

pub async fn refresh_instance_octra_shared_servers(
    instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let Some(metadata) =
        crate::state::get_instance(instance_id, &state.pool).await?
    else {
        return Ok(());
    };
    refresh_instance_overlay_locked(&metadata, &state).await
}

pub(super) async fn refresh_instance_overlay_locked(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    if instance_is_running(metadata, state).await?
        || sync_files_are_protected(metadata)
    {
        return Ok(());
    }
    if effective(metadata, state).await? {
        compose_instance(metadata, state).await?;
    } else {
        let path = instance_dir(metadata, state).join(SERVERS_FILE);
        let current = if path.exists() {
            read_servers(&path).await?
        } else {
            Vec::new()
        };
        let next = prepend_overlay(current).await;
        write_servers(&path, &next).await?;
    }
    let _ = emit_instance(
        &metadata.instance.id,
        InstancePayloadType::ServersUpdated,
    )
    .await;
    Ok(())
}
