use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const RELEASES_LATEST: &str =
    "https://api.github.com/repos/VasstOFC/octra-launcher/releases/latest";

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GithubReleaseCheck {
    NotFound,
    #[serde(rename_all = "camelCase")]
    Current {
        version: String,
        tag_name: String,
        html_url: String,
    },
    #[serde(rename_all = "camelCase")]
    Newer {
        version: String,
        tag_name: String,
        html_url: String,
        installer_url: Option<String>,
        installer_name: Option<String>,
        notes: String,
        has_latest_json: bool,
    },
    #[serde(rename_all = "camelCase")]
    Unversioned {
        tag_name: String,
        name: String,
        html_url: String,
        installer_url: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    name: Option<String>,
    html_url: String,
    body: Option<String>,
    assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
}

pub async fn check_latest(client: &reqwest::Client) -> Result<GithubReleaseCheck> {
    let ua = format!(
        "Octra/{} (+https://github.com/VasstOFC/octra-launcher)",
        env!("CARGO_PKG_VERSION")
    );
    let resp = client
        .get(RELEASES_LATEST)
        .header(USER_AGENT, ua)
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Ok(GithubReleaseCheck::NotFound);
    }
    if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::UNAUTHORIZED {
        let body = resp.text().await.unwrap_or_default();
        let lower = body.to_ascii_lowercase();
        if lower.contains("rate limit") {
            return Err(Error::msg("Zbyt wiele prób. Spróbuj za chwilę."));
        }
        return Err(Error::msg(
            "Nie udało się sprawdzić aktualizacji. Spróbuj później.",
        ));
    }
    if !status.is_success() {
        return Err(Error::msg(
            "Nie udało się sprawdzić aktualizacji. Spróbuj później.",
        ));
    }

    let release: GhRelease = resp.json().await?;
    Ok(classify_release(&release, env!("CARGO_PKG_VERSION")))
}

fn classify_release(release: &GhRelease, current: &str) -> GithubReleaseCheck {
    let name = release.name.clone().unwrap_or_default();
    let installer = pick_installer(&release.assets);
    let installer_url = installer.map(|a| a.browser_download_url.clone());
    let installer_name = installer.map(|a| a.name.clone());
    let has_latest_json = release.assets.iter().any(|a| a.name == "latest.json");

    let Some(version) = version_from_release(&release.tag_name, &name) else {
        return GithubReleaseCheck::Unversioned {
            tag_name: release.tag_name.clone(),
            name,
            html_url: release.html_url.clone(),
            installer_url,
        };
    };

    let remote = parse_semver(&version);
    let local = parse_semver(current);
    let newer = match (remote, local) {
        (Some(r), Some(l)) => r > l,
        _ => false,
    };

    if newer {
        GithubReleaseCheck::Newer {
            version,
            tag_name: release.tag_name.clone(),
            html_url: release.html_url.clone(),
            installer_url,
            installer_name,
            notes: release.body.clone().unwrap_or_default(),
            has_latest_json,
        }
    } else {
        GithubReleaseCheck::Current {
            version,
            tag_name: release.tag_name.clone(),
            html_url: release.html_url.clone(),
        }
    }
}

fn pick_installer(assets: &[GhAsset]) -> Option<&GhAsset> {
    let skip = |n: &str| n == "latest.json" || n.ends_with(".json") || n.ends_with(".sig");
    let lower = |a: &GhAsset| a.name.to_ascii_lowercase();
    assets
        .iter()
        .find(|a| lower(a) == "octra-setup.exe")
        .or_else(|| {
            assets
                .iter()
                .find(|a| !skip(&lower(a)) && lower(a).ends_with("-setup.exe"))
        })
        .or_else(|| {
            assets.iter().find(|a| {
                let n = lower(a);
                !skip(&n) && n.contains("nsis") && n.ends_with(".exe")
            })
        })
        .or_else(|| {
            assets
                .iter()
                .find(|a| !skip(&lower(a)) && (lower(a).ends_with(".exe") || lower(a).ends_with(".msi")))
        })
}

fn version_from_release(tag: &str, name: &str) -> Option<String> {
    parse_semver(tag)
        .or_else(|| parse_semver(name))
        .map(|v| format!("{}.{}.{}", v.major, v.minor, v.patch))
}

fn parse_semver(raw: &str) -> Option<SemVer> {
    let s = raw.trim().trim_start_matches(['v', 'V']);
    let core = s.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(SemVer {
        major,
        minor,
        patch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str) -> GhAsset {
        GhAsset {
            name: name.into(),
            browser_download_url: format!("https://example.test/{name}"),
        }
    }

    #[test]
    fn parses_semver_from_tag_or_name() {
        assert_eq!(parse_semver("v0.1.0").unwrap().minor, 1);
        assert_eq!(parse_semver("0.1.0").unwrap().patch, 0);
        assert!(parse_semver("release").is_none());
        assert_eq!(
            version_from_release("release", "v0.1.0").as_deref(),
            Some("0.1.0")
        );
        assert_eq!(
            version_from_release("v0.2.0", "nightly").as_deref(),
            Some("0.2.0")
        );
    }

    #[test]
    fn current_release_named_v0_1_0() {
        let release = GhRelease {
            tag_name: "release".into(),
            name: Some("v0.1.0".into()),
            html_url: "https://github.com/VasstOFC/octra-launcher/releases/tag/release".into(),
            body: Some(String::new()),
            assets: vec![],
        };
        match classify_release(&release, "0.1.0") {
            GithubReleaseCheck::Current { version, tag_name, .. } => {
                assert_eq!(version, "0.1.0");
                assert_eq!(tag_name, "release");
            }
            other => panic!("expected current, got {other:?}"),
        }
    }

    #[test]
    fn newer_release_without_installer() {
        let release = GhRelease {
            tag_name: "v0.1.1".into(),
            name: Some("Octra 0.1.1".into()),
            html_url: "https://github.com/VasstOFC/octra-launcher/releases/tag/v0.1.1".into(),
            body: Some("fix".into()),
            assets: vec![asset("source.zip")],
        };
        match classify_release(&release, "0.1.0") {
            GithubReleaseCheck::Newer {
                version,
                installer_url,
                has_latest_json,
                ..
            } => {
                assert_eq!(version, "0.1.1");
                assert!(installer_url.is_none());
                assert!(!has_latest_json);
            }
            other => panic!("expected newer, got {other:?}"),
        }
    }

    #[test]
    fn prefers_setup_exe() {
        let assets = vec![
            asset("latest.json"),
            asset("Octra_0.1.1_x64_en-US.msi"),
            asset("Octra_0.1.1_x64-setup.exe"),
        ];
        assert_eq!(
            pick_installer(&assets).unwrap().name,
            "Octra_0.1.1_x64-setup.exe"
        );
    }

    #[test]
    fn prefers_octra_setup_over_nsis() {
        let assets = vec![
            asset("Octra_0.1.0_x64-setup-nsis.exe"),
            asset("Octra-setup.exe"),
            asset("latest.json"),
        ];
        assert_eq!(pick_installer(&assets).unwrap().name, "Octra-setup.exe");
    }

    #[test]
    fn nsis_only_as_fallback() {
        let assets = vec![
            asset("latest.json"),
            asset("Octra_0.1.0_x64-setup-nsis.exe"),
        ];
        assert_eq!(
            pick_installer(&assets).unwrap().name,
            "Octra_0.1.0_x64-setup-nsis.exe"
        );
    }
}
