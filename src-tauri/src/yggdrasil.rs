//! Lokalny Yggdrasil (authlib-injector) + LAN gossip + opcjonalny rejestr HTTP.
//! Minecraft łączy się tylko z 127.0.0.1. LAN serwuje wyłącznie PNG skinów.

use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::Notify;

use crate::auth::{self, hyphenate_uuid, is_offline_uuid, offline_player_uuid, plain_uuid};
use crate::error::{Error, Result};
use crate::paths::Dirs;
use crate::settings::Settings;
use crate::skins::{self, SkinModel, StoredSkin};
use base64::Engine as _;

pub const LAN_UDP_PORT: u16 = 38741;

#[derive(Clone)]
pub struct SkinHub {
    inner: Arc<Inner>,
}

struct Inner {
    ygg_port: AtomicU16,
    lan_port: AtomicU16,
    textures: Mutex<HashMap<String, Vec<u8>>>,
    lan_peers: Mutex<HashMap<String, LanPeer>>,
    /// Offline-server UUID (v3) → player name, filled by hasJoined / bulk-byname.
    offline_names: Mutex<HashMap<String, String>>,
    announce: Notify,
    started: tokio::sync::Mutex<bool>,
}

#[derive(Clone)]
struct LanPeer {
    name: String,
    model: SkinModel,
    sha256: String,
    fetch_url: String,
}

#[derive(Serialize, Deserialize)]
struct Gossip {
    v: u8,
    app: String,
    uuid: String,
    name: String,
    model: String,
    sha256: String,
    port: u16,
}

impl SkinHub {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                ygg_port: AtomicU16::new(0),
                lan_port: AtomicU16::new(0),
                textures: Mutex::new(HashMap::new()),
                lan_peers: Mutex::new(HashMap::new()),
                offline_names: Mutex::new(HashMap::new()),
                announce: Notify::new(),
                started: tokio::sync::Mutex::new(false),
            }),
        }
    }

    pub fn ygg_root(&self) -> Option<String> {
        let p = self.inner.ygg_port.load(Ordering::Relaxed);
        if p == 0 {
            None
        } else {
            Some(format!("http://127.0.0.1:{p}"))
        }
    }

    pub fn lan_port(&self) -> u16 {
        self.inner.lan_port.load(Ordering::Relaxed)
    }

    pub fn lan_advertise_url(&self) -> Option<String> {
        let p = self.lan_port();
        if p == 0 {
            return None;
        }
        let ip = primary_lan_ipv4().unwrap_or_else(|| "127.0.0.1".into());
        Some(format!("http://{ip}:{p}/"))
    }

    pub fn put_texture(&self, sha: String, bytes: Vec<u8>) {
        self.inner.textures.lock().insert(sha, bytes);
    }

    pub fn notify_lan(&self) {
        self.inner.announce.notify_waiters();
    }

    /// Pobiera i cache'uje skiny wszystkich kont (offline + premium Mojang) pod offline UUID.
    pub async fn prefetch_account_skins(&self, http: &reqwest::Client, dirs: &Dirs) {
        let Ok(file) = auth::load_accounts(dirs) else {
            return;
        };
        for acc in file.accounts {
            if acc.is_offline() {
                if let Some(skin) = skins::load_local_skin(dirs, &acc.uuid) {
                    let offline = offline_player_uuid(&acc.name);
                    if plain_uuid(&acc.uuid) != plain_uuid(&offline) {
                        let mut alias = skin.clone();
                        alias.uuid = offline.clone();
                        let _ = skins::write_cached_skin(dirs, &alias);
                    }
                    self.put_texture(skin.sha256.clone(), skin.png.clone());
                }
                continue;
            }
            if let Some(mojang_uuid) = auth::resolve_mojang_uuid(http, &acc.name).await {
                if let Some(png) = fetch_mojang_skin_png(http, &mojang_uuid).await {
                    let offline = offline_player_uuid(&acc.name);
                    let model = SkinModel::Classic;
                    let stored = StoredSkin {
                        uuid: offline.clone(),
                        name: acc.name.clone(),
                        model: model.clone(),
                        sha256: skins::sha256_hex(&png),
                        png: png,
                    };
                    let _ = skins::write_cached_skin(dirs, &stored);
                    self.put_texture(stored.sha256.clone(), stored.png.clone());
                }
            }
        }
        self.notify_lan();
    }

    pub fn reindex(&self, dirs: &Dirs) {
        for (sha, bytes) in skins::index_texture_bytes(dirs) {
            self.put_texture(sha, bytes);
        }
    }

    pub async fn ensure_started(&self, http: reqwest::Client) -> Result<()> {
        let mut g = self.inner.started.lock().await;
        if *g && self.inner.ygg_port.load(Ordering::Relaxed) != 0 {
            return Ok(());
        }
        self.start_inner(http).await?;
        *g = true;
        Ok(())
    }

    async fn start_inner(&self, http: reqwest::Client) -> Result<()> {
        let (settings, dirs) = Settings::load()?;
        let _ = settings;
        dirs.ensure()?;
        self.reindex(&dirs);

        let ygg = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .await
            .map_err(|e| Error::msg(format!("Serwer skinów (localhost): {e}")))?;
        let ygg_port = ygg.local_addr()?.port();
        self.inner.ygg_port.store(ygg_port, Ordering::Relaxed);

        let lan = TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
            .await
            .map_err(|e| Error::msg(format!("Serwer skinów (LAN): {e}")))?;
        let lan_port = lan.local_addr()?.port();
        self.inner.lan_port.store(lan_port, Ordering::Relaxed);

        let ctx = ReqCtx {
            hub: self.clone(),
            http,
            ygg_port,
        };

        let ygg_ctx = ctx.clone();
        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = ygg.accept().await else {
                    continue;
                };
                let c = ygg_ctx.clone();
                tokio::spawn(async move {
                    let _ = handle_ygg(stream, c).await;
                });
            }
        });

        let lan_ctx = ctx.clone();
        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = lan.accept().await else {
                    continue;
                };
                let c = lan_ctx.clone();
                tokio::spawn(async move {
                    let _ = handle_lan(stream, c).await;
                });
            }
        });

        let gossip = self.clone();
        let gossip_http = ctx.http.clone();
        tokio::spawn(async move {
            lan_gossip_loop(gossip, gossip_http, lan_port).await;
        });

        eprintln!("Lumen skins: ygg http://127.0.0.1:{ygg_port}  lan :{lan_port}/skins/{{uuid}}");
        Ok(())
    }
}

fn primary_lan_ipv4() -> Option<String> {
    local_ipv4_hosts()
        .into_iter()
        .find(|h| h != "127.0.0.1" && h != "localhost")
}

fn local_ipv4_hosts() -> Vec<String> {
    let mut out = vec!["127.0.0.1".into(), "localhost".into()];
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                if let IpAddr::V4(v) = addr.ip() {
                    if !v.is_loopback() && !v.is_unspecified() && !v.is_link_local() {
                        let s = v.to_string();
                        if !out.contains(&s) {
                            out.push(s);
                        }
                    }
                }
            }
        }
    }
    if let Ok(host) = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")) {
        let host = host.trim().to_string();
        if !host.is_empty() && !out.iter().any(|h| h.eq_ignore_ascii_case(&host)) {
            if let Ok(addrs) = (host.as_str(), 0).to_socket_addrs() {
                for a in addrs {
                    if let IpAddr::V4(v) = a.ip() {
                        if !v.is_loopback() && !v.is_unspecified() && !v.is_link_local() {
                            let s = v.to_string();
                            if !out.contains(&s) {
                                out.push(s);
                            }
                        }
                    }
                }
            }
            out.push(host);
        }
    }
    out
}

fn skin_domains() -> Vec<String> {
    local_ipv4_hosts()
}

#[derive(Clone)]
struct ReqCtx {
    hub: SkinHub,
    http: reqwest::Client,
    ygg_port: u16,
}

struct HttpReq {
    method: String,
    path: String,
    query: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

struct HttpResp {
    status: u16,
    reason: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
}

impl HttpResp {
    fn json(status: u16, reason: &'static str, v: Value) -> Self {
        Self {
            status,
            reason,
            content_type: "application/json; charset=utf-8",
            body: v.to_string().into_bytes(),
        }
    }
    fn png(bytes: Vec<u8>) -> Self {
        Self {
            status: 200,
            reason: "OK",
            content_type: "image/png",
            body: bytes,
        }
    }
    fn empty(status: u16, reason: &'static str) -> Self {
        Self {
            status,
            reason,
            content_type: "text/plain",
            body: Vec::new(),
        }
    }
    fn text(status: u16, reason: &'static str, s: &str) -> Self {
        Self {
            status,
            reason,
            content_type: "text/plain; charset=utf-8",
            body: s.as_bytes().to_vec(),
        }
    }
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

async fn read_http(stream: &mut TcpStream) -> io::Result<HttpReq> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];
    loop {
        let n = tokio::time::timeout(Duration::from_secs(12), stream.read(&mut tmp))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "timeout"))??;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_header_end(&buf) {
            let header = std::str::from_utf8(&buf[..pos])
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .to_string();
            let mut lines = header.split("\r\n");
            let reqline = lines.next().unwrap_or("");
            let mut parts = reqline.split_whitespace();
            let method = parts.next().unwrap_or("GET").to_string();
            let raw_path = parts.next().unwrap_or("/").to_string();
            let (path_part, qs) = raw_path
                .split_once('?')
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .unwrap_or((raw_path, String::new()));
            let mut headers = HashMap::new();
            for line in lines {
                if let Some((k, v)) = line.split_once(':') {
                    headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
                }
            }
            let clen: usize = headers
                .get("content-length")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0)
                .min(2 * 1024 * 1024);
            let start = pos + 4;
            while buf.len() < start + clen {
                let n = stream.read(&mut tmp).await?;
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&tmp[..n]);
            }
            let body = buf.get(start..start + clen).unwrap_or(&[]).to_vec();
            let mut query = HashMap::new();
            for pair in qs.split('&') {
                if pair.is_empty() {
                    continue;
                }
                let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
                query.insert(url_decode(k), url_decode(v));
            }
            return Ok(HttpReq {
                method: method.to_ascii_uppercase(),
                path: path_part,
                query,
                headers,
                body,
            });
        }
        if buf.len() > 32 * 1024 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "headers too large"));
        }
    }
    Err(io::Error::new(io::ErrorKind::UnexpectedEof, "eof"))
}

fn url_decode(s: &str) -> String {
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < b.len() => {
                let hex = &s[i + 1..i + 3];
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    out.push(v);
                    i += 3;
                } else {
                    out.push(b[i]);
                    i += 1;
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

async fn write_http(stream: &mut TcpStream, resp: HttpResp) -> io::Result<()> {
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, PUT, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, X-Lumen-Model, X-Lumen-Name\r\n\r\n",
        resp.status,
        resp.reason,
        resp.content_type,
        resp.body.len()
    );
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(&resp.body).await?;
    stream.flush().await
}

fn segs(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty()).collect()
}

async fn handle_ygg(mut stream: TcpStream, ctx: ReqCtx) -> io::Result<()> {
    let req = read_http(&mut stream).await?;
    let resp = dispatch_ygg(&req, &ctx).await;
    write_http(&mut stream, resp).await
}

async fn handle_lan(mut stream: TcpStream, ctx: ReqCtx) -> io::Result<()> {
    let req = read_http(&mut stream).await?;
    let resp = dispatch_lan(&req, &ctx).await;
    write_http(&mut stream, resp).await
}

async fn dispatch_lan(req: &HttpReq, ctx: &ReqCtx) -> HttpResp {
    if req.method == "OPTIONS" {
        return HttpResp::empty(204, "No Content");
    }
    let s = segs(&req.path);
    if req.method == "GET" && s.len() == 2 && s[0] == "skins" {
        if let Ok((_, dirs)) = Settings::load() {
            if let Some(skin) = skins::load_local_skin(&dirs, s[1]) {
                return HttpResp::png(skin.png);
            }
            if let Some(skin) = skins::load_cached_skin(&dirs, s[1]) {
                return HttpResp::png(skin.png);
            }
        }
        return HttpResp::empty(404, "Not Found");
    }
    if (req.method == "PUT" || req.method == "POST") && s.len() == 2 && s[0] == "skins" {
        return store_lan_skin(req, ctx, s[1]).await;
    }
    HttpResp::empty(404, "Not Found")
}

async fn store_lan_skin(req: &HttpReq, ctx: &ReqCtx, uuid: &str) -> HttpResp {
    let model = req
        .headers
        .get("x-lumen-model")
        .map(|s| s.as_str())
        .unwrap_or("classic");
    let name = req
        .headers
        .get("x-lumen-name")
        .cloned()
        .unwrap_or_default();
    let png = if req.body.first().copied() == Some(b'{') {
        let info: RegistryJson = match serde_json::from_slice(&req.body) {
            Ok(v) => v,
            Err(_) => return HttpResp::text(400, "Bad Request", "niepoprawny JSON"),
        };
        let Some(url) = info.skin_url.filter(|u| !u.is_empty()) else {
            return HttpResp::text(400, "Bad Request", "brak url skina");
        };
        let Ok(resp) = ctx
            .http
            .get(&url)
            .timeout(Duration::from_secs(8))
            .send()
            .await
        else {
            return HttpResp::text(502, "Bad Gateway", "nie pobrano skina");
        };
        let Ok(bytes) = resp.bytes().await else {
            return HttpResp::text(502, "Bad Gateway", "nie pobrano skina");
        };
        bytes.to_vec()
    } else {
        req.body.clone()
    };
    let Ok((_, dirs)) = Settings::load() else {
        return HttpResp::text(500, "Internal Server Error", "brak katalogu danych");
    };
    match skins::store_registry_skin(&dirs, uuid, &png, model, &name) {
        Ok(skin) => {
            ctx.hub.put_texture(skin.sha256.clone(), skin.png.clone());
            ctx.hub.notify_lan();
            HttpResp::json(
                200,
                "OK",
                json!({ "uuid": skin.uuid, "sha256": skin.sha256 }),
            )
        }
        Err(e) => HttpResp::text(400, "Bad Request", &e.to_string()),
    }
}

async fn dispatch_ygg(req: &HttpReq, ctx: &ReqCtx) -> HttpResp {
    if req.method == "OPTIONS" {
        return HttpResp::empty(204, "No Content");
    }
    let s = segs(&req.path);

    if req.method == "GET" && (s.is_empty() || s == ["index.json"]) {
        return HttpResp::json(
            200,
            "OK",
            json!({
                "meta": {
                    "serverName": "Lumen",
                    "implementationName": "lumen-yggdrasil",
                    "implementationVersion": env!("CARGO_PKG_VERSION"),
                    "feature.non_email_login": true
                },
                "skinDomains": skin_domains(),
            }),
        );
    }

    if req.method == "GET" && s == ["status"] {
        return HttpResp::json(200, "OK", json!({"Lumen": "OK"}));
    }

    if req.method == "GET" && s.len() >= 3 && s[0] == "skins" && s[1] == "MinecraftSkins" {
        let raw = url_decode(s[2].trim_end_matches(".png"));
        return legacy_skin_png(req, ctx, &raw).await;
    }

    if req.method == "GET" && s.len() == 2 && s[0] == "textures" {
        let hash = s[1].to_ascii_lowercase();
        if let Some(bytes) = ctx.hub.inner.textures.lock().get(&hash).cloned() {
            return HttpResp::png(bytes);
        }
        if let Ok((_, dirs)) = Settings::load() {
            ctx.hub.reindex(&dirs);
            if let Some(bytes) = ctx.hub.inner.textures.lock().get(&hash).cloned() {
                return HttpResp::png(bytes);
            }
        }
        return HttpResp::empty(404, "Not Found");
    }

    if req.method == "POST" && path_ends(&s, &["authserver", "authenticate"]) {
        return auth_dummy(req);
    }
    if req.method == "POST" && path_ends(&s, &["authserver", "refresh"]) {
        return auth_dummy(req);
    }
    if req.method == "POST"
        && (path_ends(&s, &["authserver", "validate"])
            || path_ends(&s, &["authserver", "invalidate"])
            || path_ends(&s, &["authserver", "signout"])
            || path_ends(&s, &["join"])
            || path_ends(&s, &["sessionserver", "session", "minecraft", "join"]))
    {
        return HttpResp::empty(204, "No Content");
    }

    if req.method == "GET"
        && (path_ends(&s, &["hasJoined"])
            || path_ends(&s, &["sessionserver", "session", "minecraft", "hasJoined"]))
    {
        let name = req.query.get("username").cloned().unwrap_or_default();
        if name.is_empty() {
            return HttpResp::empty(204, "No Content");
        }
        let uuid = resolve_name_to_uuid(&ctx.http, &name).await;
        return profile_response(ctx, &uuid, Some(&name)).await;
    }

    if req.method == "GET" && s.len() >= 2 && s[s.len() - 2] == "profile" {
        let uuid = s[s.len() - 1];
        let name_q = req
            .query
            .get("username")
            .or(req.query.get("name"))
            .map(|s| s.as_str());
        return profile_response(ctx, uuid, name_q).await;
    }
    if req.method == "GET" && s.len() >= 2 && s[s.len() - 2] == "lookup" && s.get(s.len().saturating_sub(3)) != Some(&"name")
    {
        let uuid = s[s.len() - 1];
        let p = lookup_id_name(ctx, uuid).await;
        return HttpResp::json(200, "OK", p);
    }
    if req.method == "GET" && s.len() >= 3 && s[s.len() - 3] == "lookup" && s[s.len() - 2] == "name"
    {
        let name = url_decode(s[s.len() - 1]);
        let uuid = resolve_name_to_uuid(&ctx.http, &name).await;
        return HttpResp::json(
            200,
            "OK",
            json!({ "id": plain_uuid(&uuid), "name": name }),
        );
    }
    if req.method == "GET" && s.len() >= 3 && s[s.len() - 3] == "users" && s[s.len() - 2] == "profiles"
    {
        let name = url_decode(s[s.len() - 1]);
        let uuid = resolve_name_to_uuid(&ctx.http, &name).await;
        return HttpResp::json(
            200,
            "OK",
            json!({ "id": plain_uuid(&uuid), "name": name }),
        );
    }
    if req.method == "POST" && path_ends(&s, &["profiles", "minecraft"]) {
        return profiles_bulk(req, ctx).await;
    }
    if req.method == "POST" && path_ends(&s, &["lookup", "bulk", "byname"]) {
        return lookup_bulk_byname(req, ctx).await;
    }
    if req.method == "GET" && path_ends(&s, &["minecraftservices", "minecraft", "profile"]) {
        if let Ok((_, dirs)) = Settings::load() {
            if let Ok(Some(acc)) = auth::active_account(&dirs) {
                return HttpResp::json(
                    200,
                    "OK",
                    json!({
                        "id": plain_uuid(&acc.uuid),
                        "name": acc.name,
                        "skins": [],
                        "capes": []
                    }),
                );
            }
        }
        return HttpResp::empty(404, "Not Found");
    }

    HttpResp::text(404, "Not Found", "not found")
}

fn path_ends(s: &[&str], suffix: &[&str]) -> bool {
    s.len() >= suffix.len() && &s[s.len() - suffix.len()..] == suffix
}

fn auth_dummy(req: &HttpReq) -> HttpResp {
    let v: Value = serde_json::from_slice(&req.body).unwrap_or(json!({}));
    let name = v
        .get("username")
        .and_then(|x| x.as_str())
        .unwrap_or("Player");
    let uuid = if let Ok((_, dirs)) = Settings::load() {
        auth::load_accounts(&dirs)
            .ok()
            .and_then(|f| {
                f.accounts
                    .into_iter()
                    .find(|a| a.name.eq_ignore_ascii_case(name))
            })
            .map(|a| a.uuid)
            .unwrap_or_else(|| offline_player_uuid(name))
    } else {
        offline_player_uuid(name)
    };
    HttpResp::json(
        200,
        "OK",
        json!({
            "accessToken": "0",
            "clientToken": "lumen",
            "selectedProfile": { "id": plain_uuid(&uuid), "name": name },
            "availableProfiles": [{ "id": plain_uuid(&uuid), "name": name }]
        }),
    )
}

async fn profiles_bulk(req: &HttpReq, ctx: &ReqCtx) -> HttpResp {
    let names: Vec<String> = serde_json::from_slice(&req.body).unwrap_or_default();
    let mut out = Vec::new();
    for n in names {
        remember_offline_name(&ctx.hub, &n);
        let uuid = resolve_name_to_uuid(&ctx.http, &n).await;
        out.push(json!({ "id": plain_uuid(&uuid), "name": n }));
    }
    HttpResp::json(200, "OK", Value::Array(out))
}

async fn lookup_bulk_byname(req: &HttpReq, ctx: &ReqCtx) -> HttpResp {
    profiles_bulk(req, ctx).await
}

async fn lookup_id_name(ctx: &ReqCtx, uuid: &str) -> Value {
    let id = hyphenate_uuid(uuid);
    let name = name_for_uuid(ctx, &id).await.unwrap_or_else(|| "Player".into());
    json!({ "id": plain_uuid(&id), "name": name })
}

async fn name_for_uuid(ctx: &ReqCtx, uuid: &str) -> Option<String> {
    if let Ok((_, dirs)) = Settings::load() {
        if let Ok(file) = auth::load_accounts(&dirs) {
            if let Some(a) = file
                .accounts
                .iter()
                .find(|a| plain_uuid(&a.uuid) == plain_uuid(uuid))
            {
                return Some(a.name.clone());
            }
        }
        if let Some(s) = skins::load_local_skin(&dirs, uuid) {
            if !s.name.is_empty() {
                return Some(s.name);
            }
        }
        if let Some(s) = skins::load_cached_skin(&dirs, uuid) {
            if !s.name.is_empty() {
                return Some(s.name);
            }
        }
    }
    if let Some(p) = ctx
        .hub
        .inner
        .lan_peers
        .lock()
        .get(&plain_uuid(uuid))
        .cloned()
    {
        if !p.name.is_empty() {
            return Some(p.name);
        }
    }
    None
}

fn remember_offline_name(hub: &SkinHub, name: &str) {
    let key = plain_uuid(&offline_player_uuid(name));
    hub.inner
        .offline_names
        .lock()
        .insert(key, name.to_string());
}

async fn resolve_name_to_uuid(http: &reqwest::Client, name: &str) -> String {
    let offline = offline_player_uuid(name);
    if let Ok((_, dirs)) = Settings::load() {
        if let Ok(file) = auth::load_accounts(&dirs) {
            if let Some(a) = file
                .accounts
                .into_iter()
                .find(|a| a.name.eq_ignore_ascii_case(name))
            {
                if a.is_offline() {
                    return a.uuid;
                }
                // Premium MS account on offline-mode servers still uses OfflinePlayer UUID.
                return offline;
            }
        }
        if skins::load_local_skin(&dirs, &offline).is_some()
            || skins::load_cached_skin(&dirs, &offline).is_some()
        {
            return offline;
        }
    }
    // Known Mojang name — entity UUID on offline servers is still OfflinePlayer-derived.
    if auth::resolve_mojang_uuid(http, name).await.is_some() {
        return offline;
    }
    offline
}

fn cached_offline_name(hub: &SkinHub, id: &str) -> Option<String> {
    hub.inner
        .offline_names
        .lock()
        .get(&plain_uuid(id))
        .cloned()
}

fn offline_uuid_to_name(hub: &SkinHub, id: &str) -> Option<String> {
    let target = plain_uuid(id);
    if let Ok((_, dirs)) = Settings::load() {
        if let Ok(file) = auth::load_accounts(&dirs) {
            for a in file.accounts {
                if plain_uuid(&a.uuid) == target {
                    return Some(a.name);
                }
                if plain_uuid(&offline_player_uuid(&a.name)) == target {
                    return Some(a.name);
                }
            }
        }
        for skin in skins::list_local_custom_skins(&dirs) {
            if plain_uuid(&skin.uuid) == target {
                return Some(skin.name);
            }
            if !skin.name.is_empty() && plain_uuid(&offline_player_uuid(&skin.name)) == target {
                return Some(skin.name);
            }
        }
    }
    cached_offline_name(hub, id)
}

fn rewrite_profile_id(profile: &mut Value, uuid: &str, name: &str) {
    if let Some(obj) = profile.as_object_mut() {
        obj.insert("id".into(), json!(plain_uuid(uuid)));
        obj.insert("name".into(), json!(name));
    }
    if let Some(props) = profile.get_mut("properties").and_then(|p| p.as_array_mut()) {
        for prop in props {
            if prop.get("name").and_then(|n| n.as_str()) != Some("textures") {
                continue;
            }
            let Some(value_b64) = prop.get("value").and_then(|v| v.as_str()) else {
                continue;
            };
            let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(value_b64) else {
                continue;
            };
            let Ok(mut tex) = serde_json::from_slice::<Value>(&bytes) else {
                continue;
            };
            if let Some(obj) = tex.as_object_mut() {
                obj.insert("profileId".into(), json!(plain_uuid(uuid)));
                obj.insert("profileName".into(), json!(name));
            }
            let encoded = base64::engine::general_purpose::STANDARD.encode(tex.to_string().as_bytes());
            if let Some(obj) = prop.as_object_mut() {
                obj.insert("value".into(), json!(encoded));
            }
        }
    }
}

fn ygg_log(msg: &str) {
    eprintln!("Lumen ygg: {msg}");
    if let Ok((_, dirs)) = Settings::load() {
        let path = dirs.root.join("logs").join("yggdrasil.log");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(
                f,
                "{} {msg}",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
            );
        }
    }
}

fn profile_skin_model(profile: &Value) -> SkinModel {
    let props = profile.get("properties").and_then(|p| p.as_array());
    let Some(props) = props else {
        return SkinModel::Classic;
    };
    let Some(tex_b64) = props
        .iter()
        .find(|p| p.get("name").and_then(|n| n.as_str()) == Some("textures"))
        .and_then(|p| p.get("value").and_then(|v| v.as_str()))
    else {
        return SkinModel::Classic;
    };
    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(tex_b64) else {
        return SkinModel::Classic;
    };
    let Ok(tex_json) = serde_json::from_slice::<Value>(&bytes) else {
        return SkinModel::Classic;
    };
    let model = tex_json
        .get("textures")
        .and_then(|t| t.get("SKIN"))
        .and_then(|s| s.get("metadata"))
        .and_then(|m| m.get("model"))
        .and_then(|m| m.as_str())
        .unwrap_or("default");
    SkinModel::parse(model).unwrap_or(SkinModel::Classic)
}

async fn cache_png_skin(
    ctx: &ReqCtx,
    uuid: &str,
    name: &str,
    png: &[u8],
    model: SkinModel,
) {
    let Ok((_, dirs)) = Settings::load() else {
        return;
    };
    let stored = StoredSkin {
        uuid: hyphenate_uuid(uuid),
        name: name.to_string(),
        model,
        sha256: skins::sha256_hex(png),
        png: png.to_vec(),
    };
    let _ = skins::write_cached_skin(&dirs, &stored);
    ctx.hub.put_texture(stored.sha256.clone(), stored.png.clone());
}

async fn cache_profile_skin(ctx: &ReqCtx, uuid: &str, name: &str, profile: &Value) {
    let props = profile.get("properties").and_then(|p| p.as_array());
    let Some(tex_b64) = props.and_then(|props| {
        props
            .iter()
            .find(|p| p.get("name").and_then(|n| n.as_str()) == Some("textures"))
            .and_then(|p| p.get("value").and_then(|v| v.as_str()))
    }) else {
        return;
    };
    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(tex_b64) else {
        return;
    };
    let Ok(tex_json) = serde_json::from_slice::<Value>(&bytes) else {
        return;
    };
    let Some(url) = tex_json
        .get("textures")
        .and_then(|t| t.get("SKIN"))
        .and_then(|s| s.get("url"))
        .and_then(|u| u.as_str())
    else {
        return;
    };
    let Ok(resp) = ctx
        .http
        .get(url)
        .timeout(Duration::from_secs(8))
        .send()
        .await
    else {
        return;
    };
    if !resp.status().is_success() {
        return;
    }
    let Ok(png) = resp.bytes().await else {
        return;
    };
    let model = profile_skin_model(profile);
    cache_png_skin(ctx, uuid, name, &png, model).await;
}

async fn profile_response(ctx: &ReqCtx, uuid: &str, name_hint: Option<&str>) -> HttpResp {
    let id = hyphenate_uuid(uuid);
    if id.is_empty() {
        return HttpResp::empty(204, "No Content");
    }

    if let Some(name) = name_hint.filter(|s| !s.is_empty()) {
        remember_offline_name(&ctx.hub, name);
    }

    let name = name_hint
        .map(|s| s.to_string())
        .or_else(|| offline_uuid_to_name(&ctx.hub, &id))
        .or(name_for_uuid(ctx, &id).await);

    if let Some(ref player_name) = name {
        if let Some(mojang_uuid) = auth::resolve_mojang_uuid(&ctx.http, player_name).await {
            if let Some(mut profile) = fetch_mojang_profile(&ctx.http, &mojang_uuid).await {
                if plain_uuid(&id) != plain_uuid(&mojang_uuid) {
                    rewrite_profile_id(&mut profile, &id, player_name);
                }
                cache_profile_skin(ctx, &id, player_name, &profile).await;
                ygg_log(&format!(
                    "profil Mojang name={player_name} uuid={}",
                    plain_uuid(&id)
                ));
                return HttpResp::json(200, "OK", profile);
            }
        }
    }

    if let Some(skin) = resolve_lumen_skin_for_player(ctx, &id, name.as_deref()).await {
        let name = if !skin.name.is_empty() {
            skin.name.clone()
        } else {
            name.unwrap_or_else(|| "Player".into())
        };
        ctx.hub.put_texture(skin.sha256.clone(), skin.png.clone());
        ygg_log(&format!(
            "profil Lumen name={name} uuid={}",
            plain_uuid(&id)
        ));
        return HttpResp::json(200, "OK", lumen_profile_json(ctx, &id, &name, &skin));
    }

    if !is_offline_uuid(&id) {
        if let Some(resp) = fetch_mojang_profile(&ctx.http, &id).await {
            return HttpResp::json(200, "OK", resp);
        }
    }

    let name = name.unwrap_or_else(|| "Player".into());
    ygg_log(&format!(
        "pusty profil uuid={} name={name}",
        plain_uuid(&id)
    ));
    HttpResp::json(
        200,
        "OK",
        json!({
            "id": plain_uuid(&id),
            "name": name,
            "properties": []
        }),
    )
}

fn lumen_profile_json(ctx: &ReqCtx, uuid: &str, name: &str, skin: &StoredSkin) -> Value {
    let mut skin_obj = json!({
        "url": format!("http://127.0.0.1:{}/textures/{}", ctx.ygg_port, skin.sha256),
    });
    if let Some(m) = skin.model.ygg_metadata() {
        skin_obj["metadata"] = json!({ "model": m });
    }
    let textures = json!({
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "profileId": plain_uuid(uuid),
        "profileName": name,
        "textures": {
            "SKIN": skin_obj
        }
    });
    let value = base64::engine::general_purpose::STANDARD.encode(textures.to_string().as_bytes());
    json!({
        "id": plain_uuid(uuid),
        "name": name,
        "properties": [{ "name": "textures", "value": value }]
    })
}

async fn fetch_mojang_profile(http: &reqwest::Client, uuid: &str) -> Option<Value> {
    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{}?unsigned=true",
        plain_uuid(uuid)
    );
    let resp = http
        .get(url)
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let mut v: Value = resp.json().await.ok()?;
    if let Some(props) = v.get_mut("properties").and_then(|p| p.as_array_mut()) {
        for p in props {
            if let Some(obj) = p.as_object_mut() {
                obj.remove("signature");
            }
        }
    }
    Some(v)
}

async fn resolve_lumen_skin_for_player(
    ctx: &ReqCtx,
    uuid: &str,
    name: Option<&str>,
) -> Option<StoredSkin> {
    let (_, dirs) = Settings::load().ok()?;
    if let Some(s) = skins::load_local_skin(&dirs, uuid) {
        return Some(s);
    }
    if let Some(s) = skins::load_cached_skin(&dirs, uuid) {
        return Some(s);
    }
    if let Some(name) = name.filter(|n| !n.is_empty()) {
        let offline = offline_player_uuid(name);
        if plain_uuid(uuid) != plain_uuid(&offline) {
            if let Some(s) = skins::load_local_skin(&dirs, &offline) {
                return Some(s);
            }
            if let Some(s) = skins::load_cached_skin(&dirs, &offline) {
                return Some(s);
            }
        }
        for s in skins::list_local_custom_skins(&dirs) {
            if s.name.eq_ignore_ascii_case(name) {
                return Some(s);
            }
        }
    }
    let peer = ctx
        .hub
        .inner
        .lan_peers
        .lock()
        .get(&plain_uuid(uuid))
        .cloned();
    if let Some(peer) = peer {
        if let Some(s) = fetch_png_as_skin(&ctx.http, &peer.fetch_url, uuid, &peer.name, &peer.model).await
        {
            let _ = skins::write_cached_skin(&dirs, &s);
            ctx.hub.put_texture(s.sha256.clone(), s.png.clone());
            return Some(s);
        }
    }
    fetch_registry_skin(&ctx.http, uuid).await
}

async fn legacy_skin_png(_req: &HttpReq, ctx: &ReqCtx, name: &str) -> HttpResp {
    if name.is_empty() {
        return HttpResp::empty(404, "Not Found");
    }
    remember_offline_name(&ctx.hub, name);
    ygg_log(&format!("legacy skin PNG name={name}"));
    if let Ok((_, dirs)) = Settings::load() {
        if let Some(skin) = skins::local_skin_by_username(&dirs, name) {
            ygg_log(&format!("legacy skin PNG local Octra name={name}"));
            ctx.hub.put_texture(skin.sha256.clone(), skin.png.clone());
            return HttpResp::png(skin.png);
        }
    }
    if let Some(mojang_uuid) = auth::resolve_mojang_uuid(&ctx.http, name).await {
        if let Some(png) = fetch_mojang_skin_png(&ctx.http, &mojang_uuid).await {
            let uuid = resolve_name_to_uuid(&ctx.http, name).await;
            cache_png_skin(ctx, &uuid, name, &png, SkinModel::Classic).await;
            return HttpResp::png(png);
        }
    }
    let uuid = resolve_name_to_uuid(&ctx.http, name).await;
    if let Some(skin) = resolve_lumen_skin_for_player(ctx, &uuid, Some(name)).await {
        ctx.hub.put_texture(skin.sha256.clone(), skin.png.clone());
        return HttpResp::png(skin.png);
    }
    HttpResp::empty(404, "Not Found")
}

async fn fetch_mojang_skin_png(http: &reqwest::Client, uuid: &str) -> Option<Vec<u8>> {
    let profile = fetch_mojang_profile(http, uuid).await?;
    let props = profile.get("properties")?.as_array()?;
    let tex_b64 = props
        .iter()
        .find(|p| p.get("name").and_then(|n| n.as_str()) == Some("textures"))?
        .get("value")?
        .as_str()?;
    let tex_json: Value = serde_json::from_slice(
        &base64::engine::general_purpose::STANDARD
            .decode(tex_b64)
            .ok()?,
    )
    .ok()?;
    let url = tex_json
        .get("textures")?
        .get("SKIN")?
        .get("url")?
        .as_str()?;
    let resp = http
        .get(url)
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.bytes().await.ok().map(|b| b.to_vec())
}

async fn fetch_png_as_skin(
    http: &reqwest::Client,
    url: &str,
    uuid: &str,
    name: &str,
    model: &SkinModel,
) -> Option<StoredSkin> {
    let resp = http
        .get(url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let bytes = resp.bytes().await.ok()?.to_vec();
    if skins::validate_skin_png(&bytes).is_err() {
        return None;
    }
    Some(StoredSkin {
        uuid: hyphenate_uuid(uuid),
        name: name.to_string(),
        model: model.clone(),
        sha256: skins::sha256_hex(&bytes),
        png: bytes,
    })
}

#[derive(Deserialize)]
struct RegistryJson {
    #[serde(alias = "url")]
    skin_url: Option<String>,
    model: Option<String>,
    name: Option<String>,
}

async fn fetch_registry_skin(http: &reqwest::Client, uuid: &str) -> Option<StoredSkin> {
    let (settings, dirs) = Settings::load().ok()?;
    let base = settings.skins_url();
    if base.is_empty() {
        return None;
    }
    let url = format!("{}/skins/{}", base, hyphenate_uuid(uuid));
    let resp = http
        .get(&url)
        .timeout(Duration::from_secs(6))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let ctype = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let model_hdr = resp
        .headers()
        .get("x-lumen-model")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("classic")
        .to_string();
    let name_hdr = resp
        .headers()
        .get("x-lumen-name")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = resp.bytes().await.ok()?.to_vec();
    let (png, model, name) = if ctype.contains("json")
        || bytes.first().copied() == Some(b'{')
    {
        let info: RegistryJson = serde_json::from_slice(&bytes).ok()?;
        let skin_url = info.skin_url?;
        let png = http
            .get(&skin_url)
            .timeout(Duration::from_secs(6))
            .send()
            .await
            .ok()?
            .bytes()
            .await
            .ok()?
            .to_vec();
        (
            png,
            SkinModel::parse(info.model.as_deref().unwrap_or("classic")).ok()?,
            info.name.unwrap_or_default(),
        )
    } else {
        (
            bytes,
            SkinModel::parse(&model_hdr).unwrap_or(SkinModel::Classic),
            name_hdr,
        )
    };
    if skins::validate_skin_png(&png).is_err() {
        return None;
    }
    let skin = StoredSkin {
        uuid: hyphenate_uuid(uuid),
        name,
        model,
        sha256: skins::sha256_hex(&png),
        png,
    };
    let _ = skins::write_cached_skin(&dirs, &skin);
    Some(skin)
}

pub async fn push_registry(
    http: &reqwest::Client,
    uuid: &str,
    png: &[u8],
    model: &str,
    name: &str,
) {
    let Ok((settings, _)) = Settings::load() else {
        return;
    };
    let base = settings.skins_url();
    if base.is_empty() {
        return;
    }
    let url = format!("{}/skins/{}", base, hyphenate_uuid(uuid));
    let send = |method: reqwest::Method| {
        http.request(method, &url)
            .header(reqwest::header::CONTENT_TYPE, "image/png")
            .header("X-Lumen-Model", model)
            .header("X-Lumen-Name", name)
            .timeout(Duration::from_secs(12))
            .body(png.to_vec())
            .send()
    };
    if send(reqwest::Method::PUT).await.is_err() {
        let _ = send(reqwest::Method::POST).await;
    }
}

async fn lan_gossip_loop(hub: SkinHub, http: reqwest::Client, lan_http_port: u16) {
    let sock = match UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, LAN_UDP_PORT))).await {
        Ok(s) => s,
        Err(_) => match UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Lumen skins LAN: {e}");
                return;
            }
        },
    };
    let _ = sock.set_broadcast(true);
    let mut buf = [0u8; 2048];
    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                announce(&hub, lan_http_port, &sock).await;
            }
            _ = hub.inner.announce.notified() => {
                announce(&hub, lan_http_port, &sock).await;
            }
            rec = sock.recv_from(&mut buf) => {
                if let Ok((n, from)) = rec {
                    handle_gossip(&hub, &http, &buf[..n], from).await;
                }
            }
        }
    }
}

async fn announce(hub: &SkinHub, lan_http_port: u16, sock: &UdpSocket) {
    let Ok((_, dirs)) = Settings::load() else {
        return;
    };
    let skins = skins::list_gossip_skins(&dirs);
    for s in skins {
        let pkt = Gossip {
            v: 1,
            app: "lumen".into(),
            uuid: s.uuid,
            name: s.name,
            model: s.model.as_str().into(),
            sha256: s.sha256,
            port: lan_http_port,
        };
        if let Ok(bytes) = serde_json::to_vec(&pkt) {
            let _ = sock
                .send_to(&bytes, SocketAddr::from((Ipv4Addr::BROADCAST, LAN_UDP_PORT)))
                .await;
        }
    }
    let _ = hub;
}

async fn handle_gossip(hub: &SkinHub, http: &reqwest::Client, data: &[u8], from: SocketAddr) {
    let Ok(g) = serde_json::from_slice::<Gossip>(data) else {
        return;
    };
    if g.app != "lumen" || g.v != 1 || g.sha256.is_empty() {
        return;
    }
    let uuid = hyphenate_uuid(&g.uuid);
    let key = plain_uuid(&uuid);
    if let Ok((_, dirs)) = Settings::load() {
        if skins::load_local_skin(&dirs, &uuid).is_some() {
            return;
        }
    }
    let ip = match from.ip() {
        IpAddr::V4(v) => v,
        _ => return,
    };
    let fetch_url = format!("http://{}:{}/skins/{}", ip, g.port, uuid);
    let model = SkinModel::parse(&g.model).unwrap_or(SkinModel::Classic);
    {
        let mut peers = hub.inner.lan_peers.lock();
        if let Some(old) = peers.get(&key) {
            if old.sha256 == g.sha256 {
                return;
            }
        }
        peers.insert(
            key.clone(),
            LanPeer {
                name: g.name.clone(),
                model: model.clone(),
                sha256: g.sha256.clone(),
                fetch_url: fetch_url.clone(),
            },
        );
    }
    if let Some(skin) = fetch_png_as_skin(http, &fetch_url, &uuid, &g.name, &model).await {
        if let Ok((_, dirs)) = Settings::load() {
            let _ = skins::write_cached_skin(&dirs, &skin);
        }
        hub.put_texture(skin.sha256, skin.png);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vasst_offline_uuid_matches_java() {
        assert_eq!(
            plain_uuid(&offline_player_uuid("Vasst")),
            "1327ee532c783fdcbdb29b956278f064"
        );
    }

    #[test]
    fn offline_uuid_to_name_from_account_name() {
        let hub = SkinHub::new();
        let offline = offline_player_uuid("Vasst");
        remember_offline_name(&hub, "Vasst");
        assert_eq!(
            offline_uuid_to_name(&hub, &offline).as_deref(),
            Some("Vasst")
        );
    }

    #[test]
    fn rewrite_profile_id_updates_textures_payload() {
        let mut profile = json!({
            "id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "name": "Old",
            "properties": [{
                "name": "textures",
                "value": base64::engine::general_purpose::STANDARD.encode(
                    json!({
                        "profileId": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "profileName": "Old",
                        "textures": { "SKIN": { "url": "http://example/skin.png" } }
                    })
                    .to_string()
                    .as_bytes()
                )
            }]
        });
        let offline = offline_player_uuid("Vasst");
        rewrite_profile_id(&mut profile, &offline, "Vasst");
        assert_eq!(
            profile.get("id").and_then(|v| v.as_str()),
            Some(plain_uuid(&offline).as_str())
        );
        assert_eq!(profile.get("name").and_then(|v| v.as_str()), Some("Vasst"));
    }
}
