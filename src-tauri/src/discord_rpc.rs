//! Discord Rich Presence — menu / pobieranie / gra.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::activity::{Activity, Assets, Timestamps};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use parking_lot::Mutex;

use crate::config;

pub struct DiscordRpc {
    client: Mutex<Option<DiscordIpcClient>>,
    enabled: AtomicBool,
}

impl DiscordRpc {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            enabled: AtomicBool::new(true),
        }
    }

    pub fn set_enabled(&self, on: bool) {
        self.enabled.store(on, Ordering::SeqCst);
        if !on {
            self.clear();
        } else {
            self.set_idle();
        }
    }

    fn ensure_client(&self) -> bool {
        if !self.enabled.load(Ordering::SeqCst) {
            return false;
        }
        let mut guard = self.client.lock();
        if guard.is_some() {
            return true;
        }
        let mut client = match DiscordIpcClient::new(config::DISCORD_APP_ID) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Discord RPC: {e}");
                return false;
            }
        };
        if client.connect().is_err() {
            return false;
        }
        *guard = Some(client);
        true
    }

    fn set_activity(&self, activity: Activity) {
        if !self.ensure_client() {
            return;
        }
        let mut guard = self.client.lock();
        if let Some(client) = guard.as_mut() {
            let _ = client.set_activity(activity);
        }
    }

    pub fn clear(&self) {
        let mut guard = self.client.lock();
        if let Some(client) = guard.as_mut() {
            let _ = client.close();
        }
        *guard = None;
    }

    pub fn set_idle(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.set_activity(
            Activity::new()
                .state("W menu")
                .details("Octra Launcher")
                .assets(
                    Assets::new()
                        .large_image("octra")
                        .large_text("Octra Launcher"),
                )
                .timestamps(Timestamps::new().start(now)),
        );
    }

    pub fn set_installing(&self, message: &str) {
        self.set_activity(
            Activity::new()
                .state("Pobieranie…")
                .details(message)
                .assets(
                    Assets::new()
                        .large_image("octra")
                        .large_text("Octra Launcher"),
                ),
        );
    }

    pub fn set_playing(&self, profile: &str) {
        self.set_activity(
            Activity::new()
                .state("Gra")
                .details(profile)
                .assets(
                    Assets::new()
                        .large_image("octra")
                        .large_text("Octra Launcher"),
                ),
        );
    }
}

impl Default for DiscordRpc {
    fn default() -> Self {
        Self::new()
    }
}
