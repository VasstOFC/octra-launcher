use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::paths::Dirs;

/// Shared launcher server list (`servers.json` in the data dir).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerList {
    #[serde(default)]
    pub servers: Vec<ServerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub address: String,
}

pub fn load(dirs: &Dirs) -> Result<ServerList> {
    dirs.ensure()?;
    let path = dirs.servers_file();
    if !path.exists() {
        let list = ServerList::default();
        save(dirs, &list)?;
        return Ok(list);
    }
    let raw = std::fs::read_to_string(&path)?;
    let mut list: ServerList = serde_json::from_str(&raw)?;
    list.servers = sanitize(list.servers);
    Ok(list)
}

pub fn save(dirs: &Dirs, list: &ServerList) -> Result<ServerList> {
    dirs.ensure()?;
    let cleaned = ServerList {
        servers: sanitize(list.servers.clone()),
    };
    let json = serde_json::to_string_pretty(&cleaned)?;
    std::fs::write(dirs.servers_file(), json)?;
    Ok(cleaned)
}

/// Insert or replace a play-list entry by address (used by local hosting).
pub fn upsert(dirs: &Dirs, name: &str, address: &str) -> Result<ServerList> {
    let mut list = load(dirs)?;
    let key = address.trim().to_lowercase();
    if key.is_empty() || name.trim().is_empty() {
        return save(dirs, &list);
    }
    if let Some(e) = list
        .servers
        .iter_mut()
        .find(|s| s.address.trim().to_lowercase() == key)
    {
        e.name = name.trim().to_string();
    } else {
        list.servers.insert(
            0,
            ServerEntry {
                name: name.trim().to_string(),
                address: address.trim().to_string(),
            },
        );
    }
    save(dirs, &list)
}

pub fn remove_address(dirs: &Dirs, address: &str) -> Result<ServerList> {
    let mut list = load(dirs)?;
    let key = address.trim().to_lowercase();
    list.servers
        .retain(|s| s.address.trim().to_lowercase() != key);
    save(dirs, &list)
}

/// Pull servers added in-game (or on other instances) into `servers.json`.
/// `servers.dat` entries are merged before JSON so in-game edits win over stale launcher data.
pub fn collect_all(dirs: &Dirs) -> Result<ServerList> {
    let mut merged = Vec::new();
    if dirs.instances.exists() {
        for entry in std::fs::read_dir(&dirs.instances)?.flatten() {
            let dat = entry.path().join("minecraft").join("servers.dat");
            match read_servers_dat_file(&dat) {
                Ok(extra) => merged.extend(extra),
                Err(e) if dat.exists() => {
                    eprintln!("Lumen servers: nie odczytano {}: {e}", dat.display());
                }
                Err(_) => {}
            }
        }
    }
    merged.extend(load(dirs)?.servers);
    save(dirs, &ServerList { servers: merged })
}

/// Merge every instance's Multiplayer list, then write it into this instance.
pub fn sync_instance(dirs: &Dirs, game_dir: &Path) -> Result<usize> {
    let mut merged = Vec::new();
    let dat_path = game_dir.join("servers.dat");
    match read_servers_dat_file(&dat_path) {
        Ok(local) => merged.extend(local),
        Err(e) if dat_path.exists() => {
            eprintln!("Lumen servers: nie odczytano {}: {e}", dat_path.display());
        }
        Err(_) => {}
    }
    merged.extend(collect_all(dirs)?.servers);
    let list = ServerList {
        servers: sanitize(merged),
    };
    save(dirs, &list)?;
    write_servers_dat(game_dir, &list.servers)?;
    Ok(list.servers.len())
}

fn sanitize(entries: Vec<ServerEntry>) -> Vec<ServerEntry> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for e in entries {
        let name = e.name.trim().to_string();
        let address = e.address.trim().to_string();
        if name.is_empty() || address.is_empty() {
            continue;
        }
        let key = address.to_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(ServerEntry { name, address });
    }
    out
}

fn write_servers_dat(game_dir: &Path, servers: &[ServerEntry]) -> Result<()> {
    std::fs::create_dir_all(game_dir)?;
    let mut blob = nbt::Blob::new();
    let list: Vec<nbt::Value> = servers.iter().map(entry_to_nbt).collect();
    blob.insert("servers", nbt::Value::List(list))
        .map_err(|e| Error::msg(format!("Błąd NBT: {e}")))?;

    let path = game_dir.join("servers.dat");
    let mut f = std::fs::File::create(&path)?;
    // Minecraft zapisuje servers.dat jako nieskompresowany NBT (nie gzip).
    blob.to_writer(&mut f)
        .map_err(|e| Error::msg(format!("Błąd NBT: {e}")))?;
    f.flush()?;
    Ok(())
}

fn read_servers_dat_file(path: &Path) -> Result<Vec<ServerEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = std::fs::read(path)?;
    let blob = nbt::Blob::from_gzip_reader(&mut data.as_slice())
        .or_else(|_| nbt::Blob::from_reader(&mut data.as_slice()))
        .map_err(|e| Error::msg(format!("Błąd NBT: {e}")))?;
    Ok(entries_from_blob(&blob))
}

fn entries_from_blob(blob: &nbt::Blob) -> Vec<ServerEntry> {
    match blob.get("servers") {
        Some(nbt::Value::List(items)) => items
            .iter()
            .filter_map(|item| {
                let nbt::Value::Compound(c) = item else {
                    return None;
                };
                let name = string_tag(c.get("name"))?;
                let address = string_tag(c.get("ip"))?;
                Some(ServerEntry { name, address })
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn string_tag(v: Option<&nbt::Value>) -> Option<String> {
    match v {
        Some(nbt::Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

fn entry_to_nbt(s: &ServerEntry) -> nbt::Value {
    let mut map = HashMap::new();
    map.insert("name".into(), nbt::Value::String(s.name.clone()));
    map.insert("ip".into(), nbt::Value::String(s.address.clone()));
    map.insert("acceptTextures".into(), nbt::Value::Byte(1));
    map.insert("hidden".into(), nbt::Value::Byte(0));
    nbt::Value::Compound(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn nbt_roundtrip_compound_list() {
        let entries = vec![ServerEntry {
            name: "Moja sieć".into(),
            address: "192.168.0.10:25565".into(),
        }];
        let mut blob = nbt::Blob::new();
        let list: Vec<nbt::Value> = entries.iter().map(entry_to_nbt).collect();
        blob.insert("servers", nbt::Value::List(list)).unwrap();
        let mut buf = Vec::new();
        blob.to_writer(&mut buf).unwrap();
        assert_ne!(buf[0], 0x1f, "servers.dat must not be gzip");
        let parsed = nbt::Blob::from_reader(&mut buf.as_slice()).unwrap();
        match parsed.get("servers") {
            Some(nbt::Value::List(items)) => {
                assert_eq!(items.len(), 1);
                let nbt::Value::Compound(c) = &items[0] else {
                    panic!("expected compound");
                };
                assert_eq!(c.get("name"), Some(&nbt::Value::String("Moja sieć".into())));
                assert_eq!(
                    c.get("ip"),
                    Some(&nbt::Value::String("192.168.0.10:25565".into()))
                );
            }
            other => panic!("unexpected servers tag: {other:?}"),
        }
    }

    #[test]
    fn write_servers_dat_matches_minecraft_reader() {
        let dir = std::env::temp_dir().join(format!("octra-servers-{}", std::process::id()));
        let game_dir = dir.join("minecraft");
        fs::create_dir_all(&game_dir).unwrap();
        let entries = vec![
            ServerEntry {
                name: "Test".into(),
                address: "127.0.0.1:25565".into(),
            },
            ServerEntry {
                name: "Sky".into(),
                address: "skyup.pl".into(),
            },
        ];
        write_servers_dat(&game_dir, &entries).unwrap();
        let data = fs::read(game_dir.join("servers.dat")).unwrap();
        assert_ne!(data[0], 0x1f, "servers.dat must not be gzip");
        let parsed = read_servers_dat_file(&game_dir.join("servers.dat")).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "Test");
        assert_eq!(parsed[1].address, "skyup.pl");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sanitize_prefers_first_entry_per_address() {
        let merged = sanitize(vec![
            ServerEntry {
                name: "In-game".into(),
                address: "play.example.com".into(),
            },
            ServerEntry {
                name: "Stale JSON".into(),
                address: "play.example.com".into(),
            },
            ServerEntry {
                name: "Other".into(),
                address: "other.example.com".into(),
            },
        ]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].name, "In-game");
    }
}
