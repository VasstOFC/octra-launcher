use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VersionMeta {
    pub id: String,
    #[serde(default)]
    pub main_class: Option<String>,
    #[serde(default)]
    pub inherits_from: Option<String>,
    #[serde(default)]
    pub minecraft_arguments: Option<String>,
    #[serde(default)]
    pub arguments: Option<Arguments>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub downloads: Option<Downloads>,
    #[serde(default)]
    pub asset_index: Option<AssetIndexRef>,
    #[serde(default)]
    pub assets: Option<String>,
    #[serde(default)]
    pub java_version: Option<JavaVersionSpec>,
    #[serde(rename = "type", default)]
    pub version_type: Option<String>,
    #[serde(default)]
    pub logging: Option<Logging>,
    #[serde(default)]
    pub compliance_level: Option<i32>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub release_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
    pub jvm: Vec<Argument>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
    Plain(String),
    Ruled {
        #[serde(default)]
        rules: Vec<Rule>,
        value: StringOrVec,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringOrVec {
    One(String),
    Many(Vec<String>),
}

impl StringOrVec {
    pub fn as_vec(&self) -> Vec<String> {
        match self {
            Self::One(s) => vec![s.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Rule {
    pub action: String,
    #[serde(default)]
    pub os: Option<OsRule>,
    #[serde(default)]
    pub features: Option<FeatureRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FeatureRule {
    #[serde(default)]
    pub is_demo_user: Option<bool>,
    #[serde(default)]
    pub has_custom_resolution: Option<bool>,
    #[serde(default)]
    pub has_quick_plays_support: Option<bool>,
    #[serde(default)]
    pub is_quick_play_singleplayer: Option<bool>,
    #[serde(default)]
    pub is_quick_play_multiplayer: Option<bool>,
    #[serde(default)]
    pub is_quick_play_realms: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub natives: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub extract: Option<Extract>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Extract {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<std::collections::HashMap<String, Artifact>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Artifact {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Downloads {
    #[serde(default)]
    pub client: Option<Artifact>,
    #[serde(default)]
    pub client_mappings: Option<Artifact>,
    #[serde(default)]
    pub server: Option<Artifact>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndexRef {
    pub id: String,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub total_size: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersionSpec {
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub major_version: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Logging {
    #[serde(default)]
    pub client: Option<LoggingClient>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LoggingClient {
    #[serde(default)]
    pub argument: Option<String>,
    #[serde(default)]
    pub file: Option<ArtifactWithId>,
    #[serde(rename = "type", default)]
    pub log_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ArtifactWithId {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<ManifestVersion>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestVersion {
    pub id: String,
    #[serde(alias = "type")]
    pub version_type: String,
    pub url: String,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub release_time: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AssetIndex {
    #[serde(default)]
    pub objects: std::collections::HashMap<String, AssetObject>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AssetObject {
    pub hash: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Features {
    pub has_custom_resolution: bool,
    pub has_quick_plays_support: bool,
    pub is_quick_play_multiplayer: bool,
}

pub fn os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

pub fn os_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "unknown"
    }
}

pub fn rules_allow(rules: &[Rule], features: &Features) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut allow = false;
    for rule in rules {
        if os_matches(rule.os.as_ref()) && features_match(rule.features.as_ref(), features) {
            allow = rule.action == "allow";
        }
    }
    allow
}

fn os_matches(os: Option<&OsRule>) -> bool {
    let Some(os) = os else {
        return true;
    };
    if let Some(name) = &os.name {
        if name != os_name() {
            return false;
        }
    }
    if let Some(arch) = &os.arch {
        let ours = os_arch();
        if arch != ours && !(arch == "x64" && ours == "x86_64") {
            return false;
        }
    }
    true
}

fn features_match(rule: Option<&FeatureRule>, features: &Features) -> bool {
    let Some(rule) = rule else {
        return true;
    };
    if let Some(v) = rule.has_custom_resolution {
        if v != features.has_custom_resolution {
            return false;
        }
    }
    if let Some(v) = rule.has_quick_plays_support {
        if v != features.has_quick_plays_support {
            return false;
        }
    }
    if let Some(v) = rule.is_quick_play_multiplayer {
        if v != features.is_quick_play_multiplayer {
            return false;
        }
    }
    if rule.is_demo_user == Some(true)
        || rule.is_quick_play_singleplayer == Some(true)
        || rule.is_quick_play_realms == Some(true)
    {
        return false;
    }
    true
}

pub fn flatten_args(args: &[Argument], features: &Features) -> Vec<String> {
    let mut out = Vec::new();
    for arg in args {
        match arg {
            Argument::Plain(s) => out.push(s.clone()),
            Argument::Ruled { rules, value } => {
                if rules_allow(rules, features) {
                    out.extend(value.as_vec());
                }
            }
        }
    }
    out
}

pub fn maven_path(name: &str) -> String {
    let (name, ext) = name
        .split_once('@')
        .map(|(n, e)| (n, e))
        .unwrap_or((name, "jar"));
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return format!("{name}.{ext}");
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let filename = if parts.len() >= 4 {
        format!("{artifact}-{version}-{}.{ext}", parts[3])
    } else {
        format!("{artifact}-{version}.{ext}")
    };
    format!("{group}/{artifact}/{version}/{filename}")
}

pub fn merge_versions(base: VersionMeta, overlay: VersionMeta) -> VersionMeta {
    let mut merged = base;
    merged.id = overlay.id;
    if overlay.main_class.is_some() {
        merged.main_class = overlay.main_class;
    }
    merged.inherits_from = overlay.inherits_from;
    if overlay.minecraft_arguments.is_some() {
        merged.minecraft_arguments = overlay.minecraft_arguments;
    }
    match (merged.arguments.take(), overlay.arguments) {
        (Some(mut a), Some(b)) => {
            a.jvm.extend(b.jvm);
            a.game.extend(b.game);
            merged.arguments = Some(a);
        }
        (None, Some(b)) => merged.arguments = Some(b),
        (Some(a), None) => merged.arguments = Some(a),
        (None, None) => {}
    }
    merged.libraries.extend(overlay.libraries);
    if overlay.downloads.is_some() {
        merged.downloads = overlay.downloads;
    }
    if overlay.asset_index.is_some() {
        merged.asset_index = overlay.asset_index;
    }
    if overlay.assets.is_some() {
        merged.assets = overlay.assets;
    }
    if overlay.java_version.is_some() {
        merged.java_version = overlay.java_version;
    }
    if overlay.logging.is_some() {
        merged.logging = overlay.logging;
    }
    if overlay.version_type.is_some() {
        merged.version_type = overlay.version_type;
    }
    merged
}

pub fn native_classifier(lib: &Library) -> Option<String> {
    let natives = lib.natives.as_ref()?;
    let mut key = os_name().to_string();
    if let Some(v) = natives.get(&key) {
        return Some(v.replace("${arch}", native_arch_token()));
    }
    if os_name() == "windows" {
        key = format!("windows-{}", os_arch());
        if let Some(v) = natives.get("windows") {
            return Some(v.replace("${arch}", native_arch_token()));
        }
        let _ = key;
    }
    None
}

fn native_arch_token() -> &'static str {
    if cfg!(target_arch = "x86") {
        "32"
    } else {
        "64"
    }
}

pub fn is_native_library(lib: &Library) -> bool {
    lib.natives.is_some()
        || lib.name.contains(":natives-")
        || lib.name.contains("natives-windows")
        || lib.name.contains("natives-osx")
        || lib.name.contains("natives-linux")
}

pub fn required_java(meta: &VersionMeta) -> u32 {
    meta.java_version
        .as_ref()
        .and_then(|j| j.major_version)
        .unwrap_or_else(|| required_java_for_id(&meta.id))
}

/// Required JVM major when Mojang JSON is missing `javaVersion`.
///
/// - classic `1.16` and older → 8
/// - `1.17`–`1.20.4` → 17
/// - `1.20.5`+ (including `1.21`–`1.25`) → 21
/// - year-based drops `26.x` and `26w` snapshots → 25
pub fn required_java_for_id(id: &str) -> u32 {
    let id = id.trim();
    if id.is_empty() {
        return 8;
    }

    if let Some(yy) = snapshot_week_year(id) {
        return match yy {
            0..=20 => 8,
            21..=23 => 17,
            24..=25 => 21,
            _ => 25,
        };
    }

    let core = id.split(['-', '+', ' ']).next().unwrap_or(id);
    let mut parts = core.split('.');
    let first = parts.next().unwrap_or("");
    let Ok(first_n) = first.parse::<u32>() else {
        return 8;
    };

    // Year-based drops: 26.1, 26.2 (not 1.x).
    if first_n >= 25 && first_n < 100 {
        return if first_n >= 26 { 25 } else { 21 };
    }

    if first_n == 1 {
        let minor = parts.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let patch = parts
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        if minor >= 26 {
            return 25;
        }
        if minor > 20 || (minor == 20 && patch >= 5) {
            return 21;
        }
        if minor >= 17 {
            return 17;
        }
    }

    8
}

fn snapshot_week_year(id: &str) -> Option<u32> {
    let bytes = id.as_bytes();
    if bytes.len() < 4 || bytes[2] != b'w' {
        return None;
    }
    std::str::from_utf8(&bytes[..2]).ok()?.parse().ok()
}

#[cfg(test)]
mod required_java_tests {
    use super::required_java_for_id;

    #[test]
    fn maps_classic_and_year_drops() {
        assert_eq!(required_java_for_id("1.16.5"), 8);
        assert_eq!(required_java_for_id("1.17.1"), 17);
        assert_eq!(required_java_for_id("1.20.4"), 17);
        assert_eq!(required_java_for_id("1.20.5"), 21);
        assert_eq!(required_java_for_id("1.21.4"), 21);
        assert_eq!(required_java_for_id("1.22"), 21);
        assert_eq!(required_java_for_id("26.1"), 25);
        assert_eq!(required_java_for_id("26.2"), 25);
        assert_eq!(required_java_for_id("26.2-pre1"), 25);
        assert_eq!(required_java_for_id("24w14a"), 21);
        assert_eq!(required_java_for_id("25w41a"), 21);
        assert_eq!(required_java_for_id("26w06a"), 25);
    }
}
