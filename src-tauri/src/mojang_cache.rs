//! Cache sesji Minecraft i profilu — ogranicza pełny łańcuch Microsoft → Xbox → MC przy każdym żądaniu.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tokio::sync::Mutex as AsyncMutex;

use crate::auth::{self, Account, McSession};
use crate::error::{Error, Result};
use crate::mojang_skins::{self, McPlayerProfile};
use crate::paths::Dirs;

/// Sesja Minecraft (XBL + MC login) — kosztowna w limity Mojang.
const SESSION_TTL: Duration = Duration::from_secs(25 * 60);
/// Profil (skin + peleryny) — krótszy TTL, bo zmienia się częściej.
const PROFILE_TTL: Duration = Duration::from_secs(120);

struct Cached<T> {
    value: T,
    expires_at: Instant,
}

impl<T: Clone> Cached<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    fn get(&self) -> Option<T> {
        if Instant::now() < self.expires_at {
            Some(self.value.clone())
        } else {
            None
        }
    }
}

struct MojangCacheInner {
    sessions: HashMap<String, Cached<McSession>>,
    profiles: HashMap<String, Cached<McPlayerProfile>>,
}

pub struct MojangCache {
    inner: Mutex<MojangCacheInner>,
    session_locks: Mutex<HashMap<String, Arc<AsyncMutex<()>>>>,
}

impl Default for MojangCache {
    fn default() -> Self {
        Self {
            inner: Mutex::new(MojangCacheInner {
                sessions: HashMap::new(),
                profiles: HashMap::new(),
            }),
            session_locks: Mutex::new(HashMap::new()),
        }
    }
}

static CACHE: OnceLock<MojangCache> = OnceLock::new();

pub fn global() -> &'static MojangCache {
    CACHE.get_or_init(MojangCache::default)
}

impl MojangCache {
    pub fn invalidate_account(&self, uuid: &str) {
        let key = auth::hyphenate_uuid(uuid);
        let mut inner = self.inner.lock();
        inner.sessions.remove(&key);
        inner.profiles.remove(&key);
    }

    pub fn set_profile(&self, uuid: &str, profile: McPlayerProfile) {
        let key = auth::hyphenate_uuid(uuid);
        self.inner
            .lock()
            .profiles
            .insert(key, Cached::new(profile, PROFILE_TTL));
    }

    fn get_session(&self, uuid: &str) -> Option<McSession> {
        let key = auth::hyphenate_uuid(uuid);
        self.inner.lock().sessions.get(&key).and_then(|c| c.get())
    }

    fn put_session(&self, uuid: &str, session: McSession) {
        let key = auth::hyphenate_uuid(uuid);
        self.inner
            .lock()
            .sessions
            .insert(key, Cached::new(session, SESSION_TTL));
    }

    fn get_profile(&self, uuid: &str) -> Option<McPlayerProfile> {
        let key = auth::hyphenate_uuid(uuid);
        self.inner.lock().profiles.get(&key).and_then(|c| c.get())
    }

    fn get_profile_stale(&self, uuid: &str) -> Option<McPlayerProfile> {
        let key = auth::hyphenate_uuid(uuid);
        self.inner
            .lock()
            .profiles
            .get(&key)
            .map(|c| c.value.clone())
    }

    fn session_lock(&self, uuid: &str) -> Arc<AsyncMutex<()>> {
        let key = auth::hyphenate_uuid(uuid);
        let mut locks = self.session_locks.lock();
        locks
            .entry(key)
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    }

    pub async fn session_for_account(
        &self,
        client: &reqwest::Client,
        client_id: &str,
        dirs: &Dirs,
        account: &Account,
    ) -> Result<McSession> {
        if account.is_offline() {
            return auth::fetch_session_uncached(client, client_id, dirs, account).await;
        }

        let uuid = auth::hyphenate_uuid(&account.uuid);
        if let Some(session) = self.get_session(&uuid) {
            return Ok(session);
        }

        let lock = self.session_lock(&uuid);
        let _guard = lock.lock().await;
        if let Some(session) = self.get_session(&uuid) {
            return Ok(session);
        }

        let session =
            auth::fetch_session_uncached(client, client_id, dirs, account).await?;
        self.put_session(&uuid, session.clone());
        Ok(session)
    }

    pub async fn profile_for_account(
        &self,
        client: &reqwest::Client,
        client_id: &str,
        dirs: &Dirs,
        account_uuid: &str,
        refresh: bool,
    ) -> Result<McPlayerProfile> {
        let uuid = auth::hyphenate_uuid(account_uuid);
        if !refresh {
            if let Some(profile) = self.get_profile(&uuid) {
                return Ok(profile);
            }
        }

        let account = load_premium_account(dirs, account_uuid)?;
        let session = self
            .session_for_account(client, client_id, dirs, &account)
            .await?;

        match mojang_skins::fetch_profile(client, &session.access_token).await {
            Ok(profile) => {
                auth::auth_log(
                    dirs,
                    &format!(
                        "profil Mojang {}: {} skinów, {} peleryn{}",
                        profile.name,
                        profile.skins.len(),
                        profile.capes.len(),
                        if refresh { " (odświeżony)" } else { "" },
                    ),
                );
                self.set_profile(&uuid, profile.clone());
                Ok(profile)
            }
            Err(e) if profile_auth_error(&e) => {
                auth::auth_log(
                    dirs,
                    &format!("profil Mojang wygasł dla {uuid}, ponawiam sesję…"),
                );
                self.invalidate_account(&uuid);
                let session = self
                    .session_for_account(client, client_id, dirs, &account)
                    .await?;
                let profile = mojang_skins::fetch_profile(client, &session.access_token).await?;
                self.set_profile(&uuid, profile.clone());
                Ok(profile)
            }
            Err(e) if profile_rate_limit_error(&e) => {
                if let Some(stale) = self.get_profile_stale(&uuid) {
                    auth::auth_log(
                        dirs,
                        &format!("profil Mojang {uuid}: HTTP 429, używam cache"),
                    );
                    return Ok(stale);
                }
                Err(e)
            }
            Err(e) => Err(e),
        }
    }
}

fn load_premium_account(dirs: &Dirs, account_uuid: &str) -> Result<Account> {
    let file = auth::load_accounts(dirs)?;
    let wanted = auth::hyphenate_uuid(account_uuid);
    let account = file
        .accounts
        .iter()
        .find(|a| auth::hyphenate_uuid(&a.uuid) == wanted)
        .ok_or_else(|| Error::msg("Nie znaleziono konta."))?
        .clone();
    if account.is_offline() {
        return Err(Error::msg("Profil Mojang wymaga konta Premium."));
    }
    Ok(account)
}

fn profile_rate_limit_error(err: &Error) -> bool {
    err.to_string().contains("(HTTP 429)")
}

fn profile_auth_error(err: &Error) -> bool {
    let msg = err.to_string();
    msg.contains("(HTTP 401)")
        || msg.contains("(HTTP 403)")
        || (msg.contains("401") && msg.to_ascii_lowercase().contains("profil"))
}
