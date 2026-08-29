use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::paths::{self, Dirs};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_mem_max")]
    pub memory_max_mb: u32,
    #[serde(default = "default_mem_min")]
    pub memory_min_mb: u32,
    #[serde(default)]
    pub java_path: Option<String>,
    #[serde(default = "default_java_mode")]
    pub java_mode: String,
    #[serde(default)]
    pub azure_client_id: String,
    #[serde(default)]
    pub show_snapshots: bool,
    #[serde(default)]
    pub close_on_launch: bool,
    #[serde(default)]
    pub data_dir: Option<String>,
    /// Zostaje w JSON dla starych ustawień; karta korzysta z `config::FEATURED_PACK`.
    #[serde(default)]
    pub featured_pack: String,
    #[serde(default)]
    pub featured_pack_title: String,
    #[serde(default)]
    pub featured_server_name: String,
    #[serde(default)]
    pub featured_server_address: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_accent")]
    pub accent_color: String,
    #[serde(default = "default_accent_preset")]
    pub accent_preset: String,
    #[serde(default = "default_true")]
    pub advanced_rendering: bool,
    #[serde(default)]
    pub system_window_frame: bool,
    #[serde(default)]
    pub compact_library: bool,
    #[serde(default = "default_true")]
    pub show_play_time: bool,
    #[serde(default = "default_true")]
    pub jump_into: bool,
    #[serde(default = "default_true")]
    pub warn_unknown_mrpack: bool,
    #[serde(default)]
    pub skip_non_essential_warnings: bool,
    #[serde(default)]
    pub default_fullscreen: bool,
    #[serde(default = "default_width")]
    pub default_window_width: u32,
    #[serde(default = "default_height")]
    pub default_window_height: u32,
    #[serde(default)]
    pub default_java_args: String,
    #[serde(default)]
    pub default_env_vars: String,
    #[serde(default)]
    pub java8_path: Option<String>,
    #[serde(default)]
    pub java17_path: Option<String>,
    #[serde(default)]
    pub java21_path: Option<String>,
    #[serde(default)]
    pub java25_path: Option<String>,
    #[serde(default = "default_max_downloads")]
    pub max_concurrent_downloads: u32,
    #[serde(default = "default_max_writes")]
    pub max_concurrent_writes: u32,
    /// Opcjonalny HTTP serwer skinów Lumen (`GET/PUT /skins/{uuid}`).
    #[serde(default)]
    pub skins_url: String,
    /// Zamknięcie okna chowa launcher do zasobnika zamiast kończyć proces.
    #[serde(default = "default_true")]
    pub hide_to_tray: bool,
    /// Discord Rich Presence (domyślnie włączone).
    #[serde(default = "default_true")]
    pub discord_rpc: bool,
    /// Sprawdzaj aktualizacje przy starcie (tylko kanał Stable).
    #[serde(default = "default_true")]
    pub auto_check_updates: bool,
}

fn default_mem_max() -> u32 {
    4096
}
fn default_mem_min() -> u32 {
    512
}
fn default_java_mode() -> String {
    "auto".into()
}
fn default_theme() -> String {
    "dark".into()
}
fn default_accent() -> String {
    "lilac".into()
}
fn default_accent_preset() -> String {
    "violet".into()
}
fn default_true() -> bool {
    true
}
fn default_width() -> u32 {
    854
}
fn default_height() -> u32 {
    480
}
fn default_max_downloads() -> u32 {
    10
}
fn default_max_writes() -> u32 {
    10
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            memory_max_mb: default_mem_max(),
            memory_min_mb: default_mem_min(),
            java_path: None,
            java_mode: default_java_mode(),
            azure_client_id: String::new(),
            show_snapshots: false,
            close_on_launch: false,
            data_dir: None,
            featured_pack: String::new(),
            featured_pack_title: String::new(),
            featured_server_name: String::new(),
            featured_server_address: String::new(),
            theme: default_theme(),
            accent_color: default_accent(),
            accent_preset: default_accent_preset(),
            advanced_rendering: true,
            system_window_frame: false,
            compact_library: false,
            show_play_time: true,
            jump_into: true,
            warn_unknown_mrpack: true,
            skip_non_essential_warnings: false,
            default_fullscreen: false,
            default_window_width: default_width(),
            default_window_height: default_height(),
            default_java_args: String::new(),
            default_env_vars: String::new(),
            java8_path: None,
            java17_path: None,
            java21_path: None,
            java25_path: None,
            max_concurrent_downloads: default_max_downloads(),
            max_concurrent_writes: default_max_writes(),
            skins_url: String::new(),
            hide_to_tray: true,
            discord_rpc: true,
            auto_check_updates: true,
        }
    }
}

impl Settings {
    pub fn azure_client_id(&self) -> String {
        if let Ok(from_env) = std::env::var("LUMEN_AZURE_CLIENT_ID") {
            if !from_env.trim().is_empty() {
                return from_env;
            }
        }
        crate::config::AZURE_CLIENT_ID.trim().to_string()
    }

    fn first_nonempty(values: &[&str]) -> String {
        values
            .iter()
            .map(|s| s.trim())
            .find(|s| !s.is_empty())
            .unwrap_or("")
            .to_string()
    }

    pub fn featured_pack_query(&self) -> String {
        crate::config::FEATURED_PACK.trim().to_string()
    }

    pub fn featured_pack_title(&self) -> String {
        crate::config::FEATURED_PACK_TITLE.trim().to_string()
    }

    pub fn featured_pack_blurb(&self) -> String {
        crate::config::FEATURED_PACK_BLURB.trim().to_string()
    }

    pub fn featured_server_name(&self) -> String {
        crate::config::FEATURED_SERVER_NAME.trim().to_string()
    }

    pub fn featured_server_address(&self) -> String {
        crate::config::FEATURED_SERVER_ADDRESS.trim().to_string()
    }

    pub fn skins_url(&self) -> String {
        Self::first_nonempty(&[&self.skins_url, crate::config::LUMEN_SKINS_URL])
            .trim_end_matches('/')
            .to_string()
    }

    pub fn java_path_for_major(&self, major: u32) -> Option<&str> {
        match major {
            8 => self.java8_path.as_deref().filter(|s| !s.trim().is_empty()),
            17 => self.java17_path.as_deref().filter(|s| !s.trim().is_empty()),
            21 => self.java21_path.as_deref().filter(|s| !s.trim().is_empty()),
            25 => self.java25_path.as_deref().filter(|s| !s.trim().is_empty()),
            _ => None,
        }
    }

    pub fn apply_runtime_limits(&self) {
        crate::download::set_max_concurrent_downloads(self.max_concurrent_downloads);
    }

    pub fn load_global() -> Result<Self> {
        let root = paths::default_root();
        let path = root.join("settings.json");
        if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            let s: Settings = serde_json::from_str(&raw)?;
            s.apply_runtime_limits();
            return Ok(s);
        }
        Ok(Settings::default())
    }

    pub fn load() -> Result<(Self, Dirs)> {
        let mut settings = Self::load_global()?;
        if let Ok(env_dir) = std::env::var("OCTRA_DATA_DIR").or_else(|_| std::env::var("LUMEN_DATA_DIR")) {
            if !env_dir.is_empty() {
                settings.data_dir = Some(env_dir);
            }
        }
        let dirs = Dirs::resolve(&settings);
        dirs.ensure()?;
        // If the custom data dir has its own settings, prefer those for the rest
        // except data_dir itself.
        let local = dirs.settings_file();
        if local.exists() && settings.data_dir.is_some() {
            let raw = std::fs::read_to_string(&local)?;
            let mut nested: Settings = serde_json::from_str(&raw)?;
            nested.data_dir = settings.data_dir.clone();
            settings = nested;
        }
        settings.apply_runtime_limits();
        Ok((settings, dirs))
    }

    pub fn save(&self, dirs: &Dirs) -> Result<()> {
        dirs.ensure()?;
        self.apply_runtime_limits();
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(dirs.settings_file(), json)?;
        // Keep a copy in the channel default root so we remember a custom data_dir.
        // Nigdy nie zapisuj wskaźnika do folderu innej wersji (prod vs beta).
        if dirs.root != paths::default_root() {
            let default_root = paths::default_root();
            std::fs::create_dir_all(&default_root)?;
            let pointer = Settings {
                data_dir: Some(dirs.root.to_string_lossy().to_string()),
                ..self.clone()
            };
            std::fs::write(
                default_root.join("settings.json"),
                serde_json::to_string_pretty(&pointer)?,
            )?;
        }
        Ok(())
    }
}
