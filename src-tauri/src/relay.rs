//! LAN P2P — discovery i czat między klientami Octra w sieci lokalnej.

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::error::{Error, Result};

const DISCOVERY_PORT: u16 = 47_894;
const CHAT_PORT: u16 = 47_895;
const ANNOUNCE_INTERVAL: Duration = Duration::from_secs(4);
const PEER_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayPeerInfo {
    pub id: String,
    pub name: String,
    pub addr: String,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireAnnounce {
    #[serde(rename = "type")]
    kind: String,
    id: String,
    name: String,
    port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireChat {
    #[serde(rename = "type")]
    kind: String,
    id: String,
    name: String,
    text: String,
    at: i64,
}

#[derive(Clone)]
struct Peer {
    name: String,
    addr: SocketAddr,
    last_seen: Instant,
}

pub struct RelayHub {
    running: Arc<AtomicBool>,
    local_id: String,
    local_name: Arc<Mutex<String>>,
    peers: Arc<Mutex<HashMap<String, Peer>>>,
}

impl RelayHub {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            local_id: Uuid::new_v4().to_string(),
            local_name: Arc::new(Mutex::new(String::new())),
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn list_peers(&self) -> Vec<RelayPeerInfo> {
        let now = Instant::now();
        self.peers
            .lock()
            .iter()
            .filter(|(_, p)| now.duration_since(p.last_seen) < PEER_TIMEOUT)
            .map(|(id, p)| RelayPeerInfo {
                id: id.clone(),
                name: p.name.clone(),
                addr: p.addr.to_string(),
                online: true,
            })
            .collect()
    }

    pub fn start(&self, app: AppHandle, name: String) -> Result<()> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(Error::msg("Podaj nick do Relay LAN."));
        }
        if self.running.swap(true, Ordering::SeqCst) {
            *self.local_name.lock() = name;
            return Ok(());
        }
        *self.local_name.lock() = name;

        let running = self.running.clone();
        let local_id = self.local_id.clone();
        let local_name = self.local_name.clone();
        let peers = self.peers.clone();

        let running2 = running.clone();
        let local_id2 = local_id.clone();
        let local_name2 = local_name.clone();
        let app2 = app.clone();
        std::thread::spawn(move || {
            if let Err(e) = run_discovery(running, local_id, local_name, peers, &app) {
                eprintln!("Octra Relay discovery: {e}");
            }
        });
        std::thread::spawn(move || {
            if let Err(e) = run_chat_server(running2, local_id2, local_name2, &app2) {
                eprintln!("Octra Relay chat: {e}");
            }
        });
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.peers.lock().clear();
    }

    pub fn send(&self, peer_id: &str, text: &str) -> Result<()> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(());
        }
        let peer = self
            .peers
            .lock()
            .get(peer_id)
            .cloned()
            .ok_or_else(|| Error::msg("Peer nie jest dostępny w LAN."))?;
        let chat_addr = SocketAddr::new(peer.addr.ip(), CHAT_PORT);
        let msg = WireChat {
            kind: "chat".into(),
            id: self.local_id.clone(),
            name: self.local_name.lock().clone(),
            text: text.to_string(),
            at: chrono::Utc::now().timestamp_millis(),
        };
        let payload = serde_json::to_vec(&msg)?;
        let sock = UdpSocket::bind("0.0.0.0:0")?;
        sock.send_to(&payload, chat_addr)?;
        Ok(())
    }
}

fn run_discovery(
    running: Arc<AtomicBool>,
    local_id: String,
    local_name: Arc<Mutex<String>>,
    peers: Arc<Mutex<HashMap<String, Peer>>>,
    app: &AppHandle,
) -> Result<()> {
    let sock = UdpSocket::bind(format!("0.0.0.0:{DISCOVERY_PORT}"))?;
    sock.set_broadcast(true)?;
    sock.set_read_timeout(Some(Duration::from_secs(1)))?;

    let broadcast = SocketAddrV4::new(Ipv4Addr::BROADCAST, DISCOVERY_PORT);

    while running.load(Ordering::SeqCst) {
        let announce = WireAnnounce {
            kind: "announce".into(),
            id: local_id.clone(),
            name: local_name.lock().clone(),
            port: CHAT_PORT,
        };
        if let Ok(bytes) = serde_json::to_vec(&announce) {
            let _ = sock.send_to(&bytes, broadcast);
        }

        let mut buf = [0u8; 2048];
        match sock.recv_from(&mut buf) {
            Ok((n, from)) => {
                if let Ok(msg) = serde_json::from_slice::<WireAnnounce>(&buf[..n]) {
                    if msg.kind == "announce" && msg.id != local_id {
                        let was_new = !peers.lock().contains_key(&msg.id);
                        peers.lock().insert(
                            msg.id.clone(),
                            Peer {
                                name: msg.name.clone(),
                                addr: from,
                                last_seen: Instant::now(),
                            },
                        );
                        if was_new {
                            let _ = app.emit(
                                "relay-peer-online",
                                RelayPeerInfo {
                                    id: msg.id,
                                    name: msg.name,
                                    addr: from.to_string(),
                                    online: true,
                                },
                            );
                        }
                    }
                }
            }
            Err(_) => {}
        }

        let now = Instant::now();
        let mut gone = Vec::new();
        peers.lock().retain(|id, p| {
            if now.duration_since(p.last_seen) >= PEER_TIMEOUT {
                gone.push((id.clone(), p.name.clone()));
                false
            } else {
                true
            }
        });
        for (id, name) in gone {
            let _ = app.emit(
                "relay-peer-offline",
                RelayPeerInfo {
                    id,
                    name,
                    addr: String::new(),
                    online: false,
                },
            );
        }

        std::thread::sleep(ANNOUNCE_INTERVAL);
    }
    Ok(())
}

fn run_chat_server(
    running: Arc<AtomicBool>,
    local_id: String,
    local_name: Arc<Mutex<String>>,
    app: &AppHandle,
) -> Result<()> {
    let _ = local_name;
    let sock = UdpSocket::bind(format!("0.0.0.0:{CHAT_PORT}"))?;
    sock.set_read_timeout(Some(Duration::from_secs(1)))?;
    let mut buf = [0u8; 4096];

    while running.load(Ordering::SeqCst) {
        match sock.recv_from(&mut buf) {
            Ok((n, _from)) => {
                if let Ok(msg) = serde_json::from_slice::<WireChat>(&buf[..n]) {
                    if msg.kind == "chat" && msg.id != local_id {
                        let _ = app.emit(
                            "relay-message",
                            serde_json::json!({
                                "peerId": msg.id,
                                "peerName": msg.name,
                                "text": msg.text,
                                "at": msg.at,
                            }),
                        );
                    }
                }
            }
            Err(_) => {}
        }
    }
    Ok(())
}

impl Default for RelayHub {
    fn default() -> Self {
        Self::new()
    }
}
