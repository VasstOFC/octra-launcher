use std::path::PathBuf;

use crate::settings::Settings;

#[derive(Debug, Clone)]
pub struct Dirs {
    pub root: PathBuf,
    pub instances: PathBuf,
    pub versions: PathBuf,
    pub libraries: PathBuf,
    pub assets: PathBuf,
    pub runtime: PathBuf,
    pub cache: PathBuf,
}

impl Dirs {
    pub fn resolve(settings: &Settings) -> Self {
        let root = settings
            .data_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(default_root);
        Self::from_root(root)
    }

    pub fn from_root(root: PathBuf) -> Self {
        Self {
            instances: root.join("instances"),
            versions: root.join("versions"),
            libraries: root.join("libraries"),
            assets: root.join("assets"),
            runtime: root.join("runtime"),
            cache: root.join("cache"),
            root,
        }
    }

    pub fn ensure(&self) -> std::io::Result<()> {
        for p in [
            &self.root,
            &self.instances,
            &self.versions,
            &self.libraries,
            &self.assets,
            &self.assets.join("objects"),
            &self.assets.join("indexes"),
            &self.runtime,
            &self.cache,
            &self.cache.join("thumbs"),
            &self.root.join("skins"),
            &self.root.join("skins").join("cache"),
            &self.root.join("meta"),
        ] {
            std::fs::create_dir_all(p)?;
        }
        std::fs::create_dir_all(self.servers_root())?;
        Ok(())
    }

    pub fn servers_root(&self) -> PathBuf {
        self.root.join("servers")
    }

    pub fn local_server_dir(&self, id: &str) -> PathBuf {
        self.servers_root().join(id)
    }

    pub fn version_dir(&self, id: &str) -> PathBuf {
        self.versions.join(id)
    }

    pub fn version_json(&self, id: &str) -> PathBuf {
        self.version_dir(id).join(format!("{id}.json"))
    }

    pub fn version_jar(&self, id: &str) -> PathBuf {
        self.version_dir(id).join(format!("{id}.jar"))
    }

    pub fn instance_dir(&self, id: &str) -> PathBuf {
        self.instances.join(id)
    }

    pub fn game_dir(&self, id: &str) -> PathBuf {
        self.instance_dir(id).join("minecraft")
    }

    pub fn natives_dir(&self, id: &str) -> PathBuf {
        self.instance_dir(id).join("natives")
    }

    pub fn instance_logs(&self, id: &str) -> PathBuf {
        self.instance_dir(id).join("logs")
    }

    pub fn settings_file(&self) -> PathBuf {
        self.root.join("settings.json")
    }

    pub fn accounts_file(&self) -> PathBuf {
        self.root.join("accounts.json")
    }

    pub fn auth_tokens_file(&self) -> PathBuf {
        self.root.join("auth-tokens.json")
    }

    pub fn servers_file(&self) -> PathBuf {
        self.root.join("servers.json")
    }

    pub fn library_file(&self, relative: &str) -> PathBuf {
        self.libraries.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR))
    }

    pub fn skins_dir(&self) -> PathBuf {
        self.root.join("skins")
    }

    pub fn skins_cache_dir(&self) -> PathBuf {
        self.skins_dir().join("cache")
    }

    pub fn meta_dir(&self) -> PathBuf {
        self.root.join("meta")
    }
}

pub fn default_root() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let channel = crate::channel::current();
    let name = channel.data_dir_name();
    let root = crate::migrate::ensure_data_dir(&base, name).unwrap_or_else(|e| {
        eprintln!("Octra migracja danych: {e}");
        base.join(name)
    });
    if channel.is_stable() {
        let legacy = base.join("Octra");
        if !root.exists() && legacy.exists() {
            let _ = std::fs::rename(&legacy, &root);
        }
    }
    root
}
