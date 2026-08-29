use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::error::{Error, Result};
use crate::paths::Dirs;

const MSA_DEVICE: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const MSA_TOKEN: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MC_LOGIN: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE: &str = "https://api.minecraftservices.com/minecraft/profile";
const SESSION_PROFILE: &str = "https://sessionserver.mojang.com/session/minecraft/profile/";
const MC_ENTITLEMENTS: &str = "https://api.minecraftservices.com/entitlements/mcstore";
const SCOPE: &str = "XboxLive.signin offline_access";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TokenStore {
    #[serde(default)]
    entries: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    #[default]
    Microsoft,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub uuid: String,
    pub name: String,
    #[serde(default)]
    pub xuid: String,
    #[serde(default)]
    pub kind: AccountKind,
}

impl Account {
    pub fn is_offline(&self) -> bool {
        self.kind == AccountKind::Offline
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountsFile {
    #[serde(default)]
    pub active: Option<String>,
    #[serde(default)]
    pub accounts: Vec<Account>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCode {
    #[serde(alias = "user_code")]
    pub user_code: String,
    #[serde(alias = "device_code")]
    pub device_code: String,
    #[serde(alias = "verification_uri")]
    pub verification_uri: String,
    #[serde(alias = "verification_uri_complete", default)]
    pub verification_uri_complete: Option<String>,
    #[serde(alias = "expires_in")]
    pub expires_in: u64,
    pub interval: u64,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
}

pub(crate) fn auth_log(dirs: &Dirs, line: &str) {
    let path = dirs.root.join("logs/auth.log");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| {
            use std::io::Write;
            writeln!(file, "[{ts}] {line}")
        });
}

fn load_token_store(dirs: &Dirs) -> Result<TokenStore> {
    let path = dirs.auth_tokens_file();
    if !path.exists() {
        return Ok(TokenStore::default());
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn save_token_store(dirs: &Dirs, store: &TokenStore) -> Result<()> {
    std::fs::write(
        dirs.auth_tokens_file(),
        serde_json::to_string_pretty(store)?,
    )?;
    Ok(())
}

fn store_refresh(dirs: &Dirs, uuid: &str, refresh: &str) -> Result<()> {
    let refresh = refresh.trim();
    if refresh.is_empty() {
        return Err(Error::msg("Microsoft zwróciło pusty refresh token."));
    }
    let mut store = load_token_store(dirs)?;
    let canonical = hyphenate_uuid(uuid);
    store.entries.insert(canonical.clone(), refresh.to_string());
    store
        .entries
        .insert(plain_uuid(&canonical), refresh.to_string());
    save_token_store(dirs, &store)?;
    if try_load_refresh(dirs, uuid).as_deref() != Some(refresh) {
        return Err(Error::msg(
            "Nie udało się zapisać tokenu logowania na dysku. Sprawdź uprawnienia folderu Octra.",
        ));
    }
    auth_log(
        dirs,
        &format!("token zapisany dla {canonical} ({} znaków)", refresh.len()),
    );
    Ok(())
}

fn refresh_key_candidates(uuid: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for candidate in [uuid, &hyphenate_uuid(uuid), &plain_uuid(uuid)] {
        let c = candidate.trim();
        if c.is_empty() {
            continue;
        }
        if !keys.iter().any(|k| k == c) {
            keys.push(c.to_string());
        }
    }
    keys
}

fn try_load_refresh(dirs: &Dirs, uuid: &str) -> Option<String> {
    let store = load_token_store(dirs).ok()?;
    for key in refresh_key_candidates(uuid) {
        if let Some(token) = store.entries.get(&key) {
            let token = token.trim();
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }
    None
}

fn delete_refresh(dirs: &Dirs, uuid: &str) {
    let Ok(mut store) = load_token_store(dirs) else {
        return;
    };
    let mut changed = false;
    for key in refresh_key_candidates(uuid) {
        if store.entries.remove(&key).is_some() {
            changed = true;
        }
    }
    if changed {
        let _ = save_token_store(dirs, &store);
    }
}

#[derive(Debug, Clone)]
pub struct McSession {
    pub uuid: String,
    pub name: String,
    pub access_token: String,
    pub xuid: String,
    pub user_type: String,
}

#[derive(Debug, Deserialize)]
struct XblResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: DisplayClaims,
}

#[derive(Debug, Deserialize)]
struct DisplayClaims {
    xui: Vec<Xui>,
}

#[derive(Debug, Deserialize)]
struct Xui {
    uhs: String,
    #[serde(default)]
    xid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct McLogin {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct McProfile {
    id: String,
    name: String,
}

pub fn load_accounts(dirs: &Dirs) -> Result<AccountsFile> {
    let path = dirs.accounts_file();
    if !path.exists() {
        return Ok(AccountsFile::default());
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

pub fn save_accounts(dirs: &Dirs, file: &AccountsFile) -> Result<()> {
    std::fs::write(dirs.accounts_file(), serde_json::to_string_pretty(file)?)?;
    Ok(())
}

pub fn has_refresh_token(dirs: &Dirs, account: &Account) -> bool {
    if account.is_offline() {
        return true;
    }
    resolve_refresh_token(dirs, account).is_ok()
}

/// Szuka refresh tokenu po UUID (różne formaty) i duplikatach Premium o tym samym nicku.
pub fn resolve_refresh_token(dirs: &Dirs, account: &Account) -> Result<String> {
    if account.is_offline() {
        return Err(Error::msg("Konto offline nie używa tokenu Microsoft."));
    }
    if let Some(token) = try_load_refresh(dirs, &account.uuid) {
        return Ok(token);
    }
    let file = load_accounts(dirs)?;
    for other in &file.accounts {
        if other.kind != AccountKind::Microsoft {
            continue;
        }
        if other.uuid == account.uuid {
            continue;
        }
        if !other.name.eq_ignore_ascii_case(&account.name) {
            continue;
        }
        if let Some(token) = try_load_refresh(dirs, &other.uuid) {
            return Ok(token);
        }
    }
    auth_log(
        dirs,
        &format!(
            "brak tokenu dla {} ({}) — klucze sprawdzone: {:?}",
            account.name,
            account.uuid,
            refresh_key_candidates(&account.uuid)
        ),
    );
    Err(Error::msg(
        "Brak zapisanego tokenu Microsoft. W menu konta wybierz „Dodaj konto Premium” i zaloguj się ponownie.",
    ))
}

fn reconcile_premium_account(dirs: &Dirs, account: &Account, session: &McSession) -> Result<()> {
    let mut file = load_accounts(dirs)?;
    let old_uuid = account.uuid.clone();
    let mut changed = false;

    file.accounts.retain(|a| {
        if a.kind == AccountKind::Offline && a.name.eq_ignore_ascii_case(&session.name) {
            changed = true;
            return false;
        }
        true
    });

    if let Some(entry) = file.accounts.iter_mut().find(|a| a.uuid == old_uuid) {
        if entry.uuid != session.uuid || entry.name != session.name {
            entry.uuid = session.uuid.clone();
            entry.name = session.name.clone();
            entry.xuid = session.xuid.clone();
            changed = true;
        }
    } else if !file.accounts.iter().any(|a| a.uuid == session.uuid) {
        file.accounts.push(Account {
            uuid: session.uuid.clone(),
            name: session.name.clone(),
            xuid: session.xuid.clone(),
            kind: AccountKind::Microsoft,
        });
        changed = true;
    }

    if file.active.as_deref() == Some(old_uuid.as_str()) {
        file.active = Some(session.uuid.clone());
        changed = true;
    }

    if changed {
        save_accounts(dirs, &file)?;
    }
    Ok(())
}

pub async fn request_device_code(client: &reqwest::Client, client_id: &str) -> Result<DeviceCode> {
    if client_id.trim().is_empty() {
        return Err(Error::msg(
            "Brak Azure Client ID w launcherze. Wklej go do src-tauri/src/config.rs i przebuduj aplikację.",
        ));
    }
    let resp = client
        .post(MSA_DEVICE)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!(
            "client_id={}&scope={}",
            urlencoding_lite(client_id),
            urlencoding_lite(SCOPE)
        ))
        .send()
        .await?;
    let status = resp.status();
    let value: serde_json::Value = resp.json().await?;
    if !status.is_success() {
        return Err(Error::msg(microsoft_oauth_error(&value, status.as_u16())));
    }
    Ok(serde_json::from_value(value)?)
}

pub async fn poll_device_code(
    client: &reqwest::Client,
    client_id: &str,
    device_code: &str,
    interval: u64,
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<TokenResponse> {
    let mut wait = interval.max(1);
    loop {
        if cancel.is_cancelled() {
            return Err(Error::msg("Logowanie anulowane."));
        }
        tokio::select! {
            _ = cancel.cancelled() => return Err(Error::msg("Logowanie anulowane.")),
            _ = tokio::time::sleep(Duration::from_secs(wait)) => {}
        }
        let resp = client
            .post(MSA_TOKEN)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "grant_type={}&client_id={}&device_code={}",
                urlencoding_lite("urn:ietf:params:oauth:grant-type:device_code"),
                urlencoding_lite(client_id),
                urlencoding_lite(device_code)
            ))
            .send()
            .await?;
        let status = resp.status();
        let value: serde_json::Value = resp.json().await?;
        if status.is_success() {
            let tokens: TokenResponse = serde_json::from_value(value)?;
            return Ok(tokens);
        }
        let err = value
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        match err {
            "authorization_pending" => continue,
            "slow_down" => wait += 5,
            "expired_token" => return Err(Error::msg("Kod wygasł. Spróbuj zalogować się jeszcze raz.")),
            "authorization_declined" => return Err(Error::msg("Logowanie odrzucone.")),
            other => {
                let desc = value
                    .get("error_description")
                    .and_then(|v| v.as_str())
                    .unwrap_or(other);
                return Err(Error::msg(format!("Microsoft: {desc}")));
            }
        }
    }
}

pub async fn refresh_msa(
    client: &reqwest::Client,
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenResponse> {
    let resp = client
        .post(MSA_TOKEN)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!(
            "grant_type=refresh_token&client_id={}&refresh_token={}&scope={}",
            urlencoding_lite(client_id),
            urlencoding_lite(refresh_token),
            urlencoding_lite(SCOPE)
        ))
        .send()
        .await?;
    let status = resp.status();
    let value: serde_json::Value = resp.json().await?;
    if !status.is_success() {
        return Err(Error::msg(microsoft_oauth_error(&value, status.as_u16())));
    }
    Ok(serde_json::from_value(value)?)
}

pub async fn xbox_minecraft_session(
    client: &reqwest::Client,
    msa_access: &str,
) -> Result<McSession> {
    let xbl = client
        .post(XBL)
        .json(&serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={msa_access}")
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        }))
        .send()
        .await?
        .error_for_status()?
        .json::<XblResponse>()
        .await
        .map_err(|_| Error::msg("Nie udało się zalogować do Xbox Live."))?;
    let uhs = xbl
        .display_claims
        .xui
        .first()
        .map(|x| x.uhs.clone())
        .ok_or_else(|| Error::msg("Brak user hash z Xbox Live."))?;
    let xuid = xbl
        .display_claims
        .xui
        .first()
        .and_then(|x| x.xid.clone())
        .unwrap_or_default();

    let xsts_resp = client
        .post(XSTS)
        .json(&serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbl.token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send()
        .await?;
    if !xsts_resp.status().is_success() {
        let v: serde_json::Value = xsts_resp.json().await.unwrap_or_default();
        let code = v.get("XErr").and_then(|x| x.as_u64()).unwrap_or(0);
        return Err(Error::msg(match code {
            2148916233 => "To konto Microsoft nie ma profilu Xbox. Utwórz go na xbox.com.",
            2148916238 => "Konto jest ograniczone wiekowo. Poproś opiekuna o zgodę.",
            _ => "Xbox odmówił tokenu XSTS.",
        }));
    }
    let xsts: XblResponse = xsts_resp.json().await?;

    let mc: McLogin = client
        .post(MC_LOGIN)
        .json(&serde_json::json!({
            "identityToken": format!("XBL3.0 x={uhs};{}", xsts.token)
        }))
        .send()
        .await
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::FORBIDDEN) {
                Error::msg(
                    "Minecraft Services odrzuciło aplikację (403). Zgłoś Azure Client ID do recenzji Mojang: aka.ms/mce-reviewappid",
                )
            } else if e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                Error::msg(
                    "Za dużo prób logowania do Minecraft (429). Odczekaj 1–2 minuty i spróbuj ponownie.",
                )
            } else {
                Error::from(e)
            }
        })?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::FORBIDDEN) {
                Error::msg(
                    "Minecraft Services odrzuciło aplikację (403). Zgłoś Azure Client ID do recenzji Mojang: aka.ms/mce-reviewappid",
                )
            } else if e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                Error::msg(
                    "Za dużo prób logowania do Minecraft (429). Odczekaj 1–2 minuty i spróbuj ponownie.",
                )
            } else {
                Error::msg(format!("Logowanie do Minecraft: {e}"))
            }
        })?
        .json()
        .await?;

    let entitlements = client
        .get(MC_ENTITLEMENTS)
        .bearer_auth(&mc.access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    let items = entitlements
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if items.is_empty() {
        return Err(Error::msg(
            "To konto nie ma Minecraft Java Edition. Zaloguj się kontem, które posiada grę.",
        ));
    }

    let profile_resp = client
        .get(MC_PROFILE)
        .bearer_auth(&mc.access_token)
        .send()
        .await?;
    if profile_resp.status().as_u16() == 404 {
        return Err(Error::msg(
            "Konto nie ma profilu Java. Uruchom oficjalny launcher raz, żeby go utworzyć.",
        ));
    }
    let profile: McProfile = profile_resp.error_for_status()?.json().await?;
    let uuid = hyphenate_uuid(&profile.id);
    Ok(McSession {
        uuid,
        name: profile.name,
        access_token: mc.access_token,
        xuid,
        user_type: "msa".into(),
    })
}

pub async fn complete_login(
    client: &reqwest::Client,
    dirs: &Dirs,
    client_id: &str,
    tokens: TokenResponse,
) -> Result<Account> {
    let session = xbox_minecraft_session(client, &tokens.access_token).await?;
    let refresh = tokens
        .refresh_token
        .ok_or_else(|| Error::msg("Microsoft nie zwróciło refresh tokenu (sprawdź scope offline_access)."))?;
    store_refresh(dirs, &session.uuid, &refresh)?;
    let mut file = load_accounts(dirs)?;
    file.accounts.retain(|a| {
        !(a.kind == AccountKind::Offline && a.name.eq_ignore_ascii_case(&session.name))
    });
    file.accounts.retain(|a| a.uuid != session.uuid);
    let account = Account {
        uuid: session.uuid.clone(),
        name: session.name.clone(),
        xuid: session.xuid,
        kind: AccountKind::Microsoft,
    };
    file.accounts.push(account.clone());
    file.active = Some(session.uuid);
    let _ = client_id;
    save_accounts(dirs, &file)?;
    auth_log(
        dirs,
        &format!(
            "logowanie OK: {} ({}) xuid={}",
            account.name, account.uuid, account.xuid
        ),
    );
    Ok(account)
}

pub async fn session_for_account(
    client: &reqwest::Client,
    client_id: &str,
    dirs: &Dirs,
    account: &Account,
) -> Result<McSession> {
    if account.is_offline() {
        return Ok(offline_session(account));
    }
    let refresh = resolve_refresh_token(dirs, account)?;
    let tokens = refresh_msa(client, client_id, &refresh).await.map_err(|e| {
        auth_log(
            dirs,
            &format!("odświeżanie tokenu nieudane dla {}: {e}", account.name),
        );
        Error::msg(format!(
            "Sesja Microsoft wygasła. Zaloguj się ponownie przez menu konta. ({e})"
        ))
    })?;
    let mut session = xbox_minecraft_session(client, &tokens.access_token).await?;
    if session.xuid.is_empty() {
        session.xuid = account.xuid.clone();
    }
    reconcile_premium_account(dirs, account, &session)?;
    let refresh_to_store = tokens
        .refresh_token
        .as_deref()
        .unwrap_or(&refresh);
    let _ = store_refresh(dirs, &session.uuid, refresh_to_store);
    Ok(session)
}

pub fn add_offline_account(dirs: &Dirs, name: &str) -> Result<Account> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::msg("Podaj nick."));
    }
    if name.len() > 16 || name.contains(char::is_whitespace) {
        return Err(Error::msg(
            "Nick offline: 1–16 znaków, bez spacji (jak w Minecraft).",
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(Error::msg(
            "Nick może zawierać tylko litery, cyfry i podkreślenie.",
        ));
    }
    let uuid = offline_player_uuid(name);
    let mut file = load_accounts(dirs)?;
    file.accounts.retain(|a| a.uuid != uuid);
    let account = Account {
        uuid: uuid.clone(),
        name: name.to_string(),
        xuid: String::new(),
        kind: AccountKind::Offline,
    };
    file.accounts.push(account.clone());
    file.active = Some(uuid);
    save_accounts(dirs, &file)?;
    Ok(account)
}

fn offline_session(account: &Account) -> McSession {
    McSession {
        uuid: account.uuid.clone(),
        name: account.name.clone(),
        access_token: "0".into(),
        xuid: "0".into(),
        // Nowsze MC oczekuje msa; serwery offline-mode i tak biorą nick + UUID OfflinePlayer.
        user_type: "msa".into(),
    }
}

/// UUID v3 jak `UUID.nameUUIDFromBytes("OfflinePlayer:" + name)` w Javie.
pub fn offline_player_uuid(name: &str) -> String {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{name}").as_bytes());
    let mut bytes = hasher.finalize();
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex = hex::encode(bytes);
    hyphenate_uuid(&hex)
}

pub fn plain_uuid(id: &str) -> String {
    id.replace('-', "").to_lowercase()
}

/// UUID wersja (nibble w bajcie 6). OfflinePlayer z Javy to v3.
pub fn uuid_version(id: &str) -> u8 {
    let hex = plain_uuid(id);
    if hex.len() != 32 {
        return 0;
    }
    u8::from_str_radix(&hex[12..13], 16).unwrap_or(0)
}

pub fn is_offline_uuid(id: &str) -> bool {
    uuid_version(id) == 3
}

pub fn logout(dirs: &Dirs, uuid: &str) -> Result<AccountsFile> {
    delete_refresh(dirs, uuid);
    let mut file = load_accounts(dirs)?;
    file.accounts.retain(|a| a.uuid != uuid);
    if file.active.as_deref() == Some(uuid) || file.accounts.is_empty() {
        file.active = file.accounts.first().map(|a| a.uuid.clone());
    }
    save_accounts(dirs, &file)?;
    Ok(file)
}

pub fn set_active(dirs: &Dirs, uuid: &str) -> Result<AccountsFile> {
    let mut file = load_accounts(dirs)?;
    if !file.accounts.iter().any(|a| a.uuid == uuid) {
        return Err(Error::msg("Nie znaleziono konta."));
    }
    file.active = Some(uuid.to_string());
    save_accounts(dirs, &file)?;
    Ok(file)
}

pub fn active_account(dirs: &Dirs) -> Result<Option<Account>> {
    let file = load_accounts(dirs)?;
    Ok(file
        .active
        .as_ref()
        .and_then(|id| file.accounts.iter().find(|a| &a.uuid == id).cloned()))
}

pub fn hyphenate_uuid(id: &str) -> String {
    let id = id.replace('-', "");
    if id.len() != 32 {
        return id;
    }
    format!(
        "{}-{}-{}-{}-{}",
        &id[0..8],
        &id[8..12],
        &id[12..16],
        &id[16..20],
        &id[20..32]
    )
}

fn microsoft_oauth_error(value: &serde_json::Value, http_status: u16) -> String {
    let code = value
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let desc = value
        .get("error_description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let codes = value
        .get("error_codes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_u64())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if codes.contains(&700016) || desc.contains("AADSTS700016") {
        return [
            "Microsoft nie widzi tej aplikacji przy kontach osobistych (AADSTS700016).",
            "W Azure Entra ID otwórz aplikację Octra → Rejestracja aplikacji →",
            "„Kto może korzystać z tej aplikacji” musi być:",
            "konta osobiste Microsoft albo „dowolny katalog + konta osobiste”.",
            "Sama zgoda public client i zgodne Client ID nie wystarczą, jeśli aplikacja jest tylko służbowa.",
            "Tego typu często nie da się zmienić — wtedy utwórz nową rejestrację z kontami osobistymi,",
            "wklej nowy Client ID do src-tauri/src/config.rs i przebuduj launcher.",
            "Dodaj też platformę „Aplikacje mobilne i klasyczne” (nativeclient / localhost).",
        ]
        .join(" ");
    }

    if desc.is_empty() {
        format!("Microsoft OAuth HTTP {http_status} ({code}).")
    } else {
        let short = desc.split(" Trace ID:").next().unwrap_or(desc).trim();
        format!("Microsoft: {short}")
    }
}

fn urlencoding_lite(s: &str) -> String {
    let mut out = String::new();
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSkin {
    pub uuid: String,
    pub model: String,
    pub texture_url: Option<String>,
    pub cape_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub png_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cape_png_base64: Option<String>,
    pub is_premium: bool,
}

#[derive(Debug, Deserialize)]
struct SessionProfileProps {
    properties: Vec<SessionProfileProp>,
}

#[derive(Debug, Deserialize)]
struct SessionProfileProp {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct SessionTextures {
    #[serde(default)]
    textures: SessionTextureMap,
}

#[derive(Debug, Deserialize, Default)]
struct SessionTextureMap {
    #[serde(default)]
    skin: Option<SessionSkinTex>,
    #[serde(default)]
    cape: Option<SessionSkinTex>,
}

#[derive(Debug, Deserialize)]
struct SessionSkinTex {
    url: String,
    #[serde(default)]
    metadata: Option<SessionSkinMeta>,
}

#[derive(Debug, Deserialize)]
struct SessionSkinMeta {
    #[serde(default)]
    model: String,
}

fn premium_skin_default(uuid: &str) -> AccountSkin {
    AccountSkin {
        uuid: uuid.to_string(),
        model: "classic".into(),
        texture_url: None,
        cape_url: None,
        png_base64: None,
        cape_png_base64: None,
        is_premium: true,
    }
}

async fn texture_png_base64(client: &reqwest::Client, url: &str) -> Option<String> {
    use base64::Engine as _;
    let bytes = client
        .get(url)
        .timeout(Duration::from_secs(12))
        .send()
        .await
        .ok()?
        .bytes()
        .await
        .ok()?;
    Some(base64::engine::general_purpose::STANDARD.encode(bytes))
}

async fn enrich_skin_textures(client: &reqwest::Client, skin: &mut AccountSkin) {
    if let Some(url) = skin.texture_url.as_deref() {
        skin.png_base64 = texture_png_base64(client, url).await;
    }
    if let Some(url) = skin.cape_url.as_deref() {
        skin.cape_png_base64 = texture_png_base64(client, url).await;
    }
}

#[derive(Debug, Deserialize)]
struct MojangNameLookup {
    id: String,
}

pub(crate) async fn resolve_mojang_uuid(client: &reqwest::Client, name: &str) -> Option<String> {
    let encoded = urlencoding_lite(name);
    let urls = [
        format!("https://api.minecraftservices.com/minecraft/profile/lookup/name/{encoded}"),
        format!("https://api.mojang.com/users/profiles/minecraft/{encoded}"),
        format!("https://api.mojang.com/minecraft/profile/lookup/name/{encoded}"),
    ];
    for url in urls {
        let resp = match client.get(&url).timeout(Duration::from_secs(6)).send().await {
            Ok(r) => r,
            Err(_) => continue,
        };
        if !resp.status().is_success() {
            continue;
        }
        let body: MojangNameLookup = match resp.json().await {
            Ok(b) => b,
            Err(_) => continue,
        };
        return Some(hyphenate_uuid(&body.id));
    }
    None
}

async fn fetch_skin_from_session_server(
    client: &reqwest::Client,
    uuid: &str,
    no_cache: bool,
) -> Result<AccountSkin> {
    use base64::Engine as _;

    let url = format!(
        "{}{}?unsigned=true",
        SESSION_PROFILE,
        plain_uuid(uuid)
    );
    let mut req = client.get(&url).timeout(Duration::from_secs(8));
    if no_cache {
        req = req.header("Cache-Control", "no-cache");
    }
    let resp = req.send().await?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(Error::msg("Nie znaleziono profilu Mojang dla tego UUID."));
    }
    if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(Error::msg(
            "Mojang ograniczyło liczbę zapytań (429). Poczekaj chwilę i spróbuj ponownie.",
        ));
    }
    if !resp.status().is_success() {
        return Err(Error::msg(format!(
            "Nie udało się pobrać skina z Mojang (HTTP {}).",
            resp.status().as_u16()
        )));
    }
    let body: SessionProfileProps = resp.json().await?;
    let Some(textures_b64) = body
        .properties
        .iter()
        .find(|p| p.name == "textures")
        .map(|p| p.value.as_str())
    else {
        return Ok(premium_skin_default(uuid));
    };
    let json = base64::engine::general_purpose::STANDARD
        .decode(textures_b64)
        .map_err(|_| Error::msg("Nie udało się odczytać tekstur skina."))?;
    let tex: SessionTextures = serde_json::from_slice(&json)?;
    let Some(skin) = tex.textures.skin else {
        return Ok(premium_skin_default(uuid));
    };
    if skin.url.is_empty() {
        return Ok(premium_skin_default(uuid));
    };
    let model = skin
        .metadata
        .as_ref()
        .map(|m| {
            if m.model.eq_ignore_ascii_case("slim") {
                "slim"
            } else {
                "classic"
            }
        })
        .unwrap_or("classic")
        .to_string();
    let cape_url = tex
        .textures
        .cape
        .filter(|c| !c.url.is_empty())
        .map(|c| c.url.clone());
    Ok(AccountSkin {
        uuid: uuid.to_string(),
        model,
        texture_url: Some(skin.url),
        cape_url,
        png_base64: None,
        cape_png_base64: None,
        is_premium: true,
    })
}

pub async fn fetch_account_skin(
    client: &reqwest::Client,
    _client_id: &str,
    dirs: &Dirs,
    account: &Account,
    force_refresh: bool,
) -> Result<AccountSkin> {
    if account.is_offline() {
        return Ok(AccountSkin {
            uuid: account.uuid.clone(),
            model: "classic".into(),
            texture_url: None,
            cape_url: None,
            png_base64: None,
            cape_png_base64: None,
            is_premium: false,
        });
    }

    let resolved_uuid = resolve_mojang_uuid(client, &account.name)
        .await
        .unwrap_or_else(|| account.uuid.clone());

    let mut skin =
        fetch_skin_from_session_server(client, &resolved_uuid, force_refresh).await?;
    skin.uuid = resolved_uuid;
    enrich_skin_textures(client, &mut skin).await;

    if let Some(png_b64) = skin.png_base64.as_deref() {
        use base64::Engine as _;
        use sha2::{Digest, Sha256};
        if let Ok(png) = base64::engine::general_purpose::STANDARD.decode(png_b64) {
            let model = crate::skins::SkinModel::parse(&skin.model)
                .unwrap_or(crate::skins::SkinModel::Classic);
            let sha256 = format!("{:x}", Sha256::digest(&png));
            let _ = crate::skins::write_cached_skin(
                dirs,
                &crate::skins::StoredSkin {
                    uuid: skin.uuid.clone(),
                    png,
                    model,
                    sha256,
                    name: account.name.clone(),
                },
            );
        }
    }

    Ok(skin)
}
