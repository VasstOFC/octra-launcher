use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerPingResult {
    pub online: bool,
    pub latency_ms: Option<u32>,
    pub players: Option<u32>,
    pub max_players: Option<u32>,
    pub version: Option<String>,
    pub motd: Option<String>,
}

pub async fn ping_server(address: &str) -> Result<ServerPingResult> {
    let (host, port) = parse_address(address)?;
    let addr = format!("{host}:{port}");
    let started = Instant::now();
    let mut stream = match timeout(Duration::from_secs(4), TcpStream::connect(&addr)).await {
        Ok(Ok(s)) => s,
        _ => {
            return Ok(ServerPingResult {
                online: false,
                latency_ms: None,
                players: None,
                max_players: None,
                version: None,
                motd: None,
            });
        }
    };
    let handshake = build_handshake(&host, port);
    if stream.write_all(&handshake).await.is_err() {
        return Ok(offline());
    }
    if stream.write_all(&build_status_request()).await.is_err() {
        return Ok(offline());
    }
    let mut len_buf = [0u8; 1];
    if stream.read_exact(&mut len_buf).await.is_err() {
        return Ok(offline());
    }
    let packet_len = read_varint_prefix(&mut stream, len_buf[0]).await?;
    if packet_len <= 0 || packet_len > 65536 {
        return Ok(offline());
    }
    let mut packet = vec![0u8; packet_len as usize];
    if stream.read_exact(&mut packet).await.is_err() {
        return Ok(offline());
    }
    let latency_ms = started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32;
    let json = extract_status_json(&packet).unwrap_or_default();
    if json.is_empty() {
        return Ok(ServerPingResult {
            online: true,
            latency_ms: Some(latency_ms),
            players: None,
            max_players: None,
            version: None,
            motd: None,
        });
    }
    let parsed: serde_json::Value = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(_) => {
            return Ok(ServerPingResult {
                online: true,
                latency_ms: Some(latency_ms),
                players: None,
                max_players: None,
                version: None,
                motd: Some(json),
            });
        }
    };
    let version = parsed
        .get("version")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let players = parsed
        .get("players")
        .and_then(|p| p.get("online"))
        .and_then(|v| v.as_u64())
        .map(|n| n.min(u64::from(u32::MAX)) as u32);
    let max_players = parsed
        .get("players")
        .and_then(|p| p.get("max"))
        .and_then(|v| v.as_u64())
        .map(|n| n.min(u64::from(u32::MAX)) as u32);
    let motd = extract_motd(parsed.get("description"));
    Ok(ServerPingResult {
        online: true,
        latency_ms: Some(latency_ms),
        players,
        max_players,
        version,
        motd,
    })
}

fn offline() -> ServerPingResult {
    ServerPingResult {
        online: false,
        latency_ms: None,
        players: None,
        max_players: None,
        version: None,
        motd: None,
    }
}

fn parse_address(addr: &str) -> Result<(String, u16)> {
    let addr = addr.trim();
    if addr.is_empty() {
        return Err(Error::msg("Pusty adres serwera."));
    }
    if addr.starts_with('[') {
        if let Some(end) = addr.find(']') {
            let host = addr[1..end].to_string();
            let port = if addr.len() > end + 1 && addr.as_bytes().get(end + 1) == Some(&b':') {
                addr[end + 2..].parse().unwrap_or(25565)
            } else {
                25565
            };
            return Ok((host, port));
        }
    }
    if let Some((host, port_str)) = addr.rsplit_once(':') {
        if let Ok(port) = port_str.parse::<u16>() {
            if !host.is_empty() && !host.contains(':') {
                return Ok((host.to_string(), port));
            }
        }
    }
    Ok((addr.to_string(), 25565))
}

fn write_varint(buf: &mut Vec<u8>, mut value: i32) {
    loop {
        let mut temp = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            temp |= 0x80;
        }
        buf.push(temp);
        if value == 0 {
            break;
        }
    }
}

fn write_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.extend_from_slice(bytes);
}

fn build_handshake(host: &str, port: u16) -> Vec<u8> {
    let mut payload = Vec::new();
    write_varint(&mut payload, 0);
    write_varint(&mut payload, 47);
    write_string(&mut payload, host);
    payload.extend_from_slice(&port.to_be_bytes());
    write_varint(&mut payload, 1);
    let mut packet = Vec::new();
    write_varint(&mut packet, payload.len() as i32);
    packet.extend(payload);
    packet
}

fn build_status_request() -> Vec<u8> {
    let mut payload = Vec::new();
    write_varint(&mut payload, 0);
    let mut packet = Vec::new();
    write_varint(&mut packet, payload.len() as i32);
    packet.extend(payload);
    packet
}

async fn read_varint_prefix(stream: &mut TcpStream, first: u8) -> Result<i32> {
    let mut num_read = 0;
    let mut value = 0i32;
    let mut part = first;
    loop {
        value |= ((part & 0x7F) as i32) << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            return Err(Error::msg("Niepoprawna długość pakietu."));
        }
        if part & 0x80 == 0 {
            break;
        }
        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).await?;
        part = buf[0];
    }
    Ok(value)
}

fn read_varint(data: &mut &[u8]) -> Option<i32> {
    let mut num_read = 0;
    let mut value = 0i32;
    loop {
        let part = *data.first()?;
        *data = &data[1..];
        value |= ((part & 0x7F) as i32) << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            return None;
        }
        if part & 0x80 == 0 {
            break;
        }
    }
    Some(value)
}

fn extract_status_json(packet: &[u8]) -> Option<String> {
    let mut data = packet;
    let id = read_varint(&mut data)?;
    if id != 0 {
        return None;
    }
    let len = read_varint(&mut data)? as usize;
    if data.len() < len {
        return None;
    }
    String::from_utf8(data[..len].to_vec()).ok()
}

fn extract_motd(value: Option<&serde_json::Value>) -> Option<String> {
    let v = value?;
    if let Some(s) = v.as_str() {
        return Some(s.to_string());
    }
    if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
        let extra = v
            .get("extra")
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
                    .collect::<String>()
            })
            .unwrap_or_default();
        return Some(format!("{text}{extra}"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_host_port() {
        assert_eq!(
            parse_address("play.example.com:25565").unwrap(),
            ("play.example.com".into(), 25565)
        );
        assert_eq!(
            parse_address("skyup.pl").unwrap(),
            ("skyup.pl".into(), 25565)
        );
        assert_eq!(
            parse_address("[::1]:25565").unwrap(),
            ("::1".into(), 25565)
        );
    }
}
