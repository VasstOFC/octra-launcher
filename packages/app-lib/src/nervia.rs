//! Octra App identity (workspace folder is still "Nervia App").
//!
//! Minecraft sign-in uses the Xbox SISU flow. Catalog still talks to Modrinth.
//! Octra accounts use the VPS at [`SKINS_URL`] (register/login + JWT skin uploads).
//!
//! # Changing the skin registry URL (HTTPS / domain cutover)
//!
//! 1. Prefer editing [`SKINS_URL`] below — single compile-time source of truth.
//! 2. Optional runtime override: set env `OCTRA_SKINS_URL` (no trailing slash), e.g.
//!    `https://<your-domain>`. Useful for local/dev builds without rebuilding.
//! 3. After changing the URL, also update:
//!    - `apps/app/capabilities/plugins.json` — add `https://<domain>/*` to http allowlist
//!    - `apps/app/tauri.conf.json` — CSP `connect-src` and `img-src`
//!    - Rebuild/redistribute the launcher so authlib-injector points at the new base URL
//! 4. Full VPS + Cloudflare checklist: `services/octra-api/deploy/README.md`
//!
//! Authlib-injector uses [`skins_url()`] as the shared remote Yggdrasil API root
//! (see `octra_skins::ygg_root` / `octra_skins::prepare_launch`).

use std::sync::OnceLock;

/// Azure public client used by Octra (device-code / public-client flows).
pub const AZURE_CLIENT_ID: &str = "bfe8ec3a-8e09-4be9-bbef-68f0fa0d1321";

/// Default skin registry base URL (production).
///
/// Keep the HTTP IP until a real domain + HTTPS is ready. Do not invent a placeholder
/// domain here — change this value (or set `OCTRA_SKINS_URL`) only when DNS works.
///
/// Prefer [`skins_url()`] at call sites so the optional env override is honored.
pub const SKINS_URL: &str = "http://92.5.186.6";

/// Env var that overrides [`SKINS_URL`] at process start (trimmed, no trailing `/`).
pub const SKINS_URL_ENV: &str = "OCTRA_SKINS_URL";

/// Resolved skin registry base URL (no trailing slash).
///
/// Order: non-empty `OCTRA_SKINS_URL` env → [`SKINS_URL`].
pub fn skins_url() -> &'static str {
	static RESOLVED: OnceLock<String> = OnceLock::new();
	RESOLVED
		.get_or_init(|| {
			std::env::var(SKINS_URL_ENV)
				.ok()
				.map(|s| s.trim().trim_end_matches('/').to_string())
				.filter(|s| !s.is_empty())
				.unwrap_or_else(|| SKINS_URL.trim_end_matches('/').to_string())
		})
		.as_str()
}

/// Host part of [`skins_url()`] (hostname or IP, no port/scheme).
pub fn skins_host() -> &'static str {
	static HOST: OnceLock<String> = OnceLock::new();
	HOST.get_or_init(|| {
		let base = skins_url();
		let without_scheme = base
			.strip_prefix("https://")
			.or_else(|| base.strip_prefix("http://"))
			.unwrap_or(base);
		without_scheme
			.split(['/', ':'])
			.next()
			.unwrap_or(without_scheme)
			.to_string()
	})
	.as_str()
}

/// Header `X-Octra-Key` for skin uploads. Must match `/etc/octra-skins.env`.
pub const SKINS_API_KEY: &str =
	"73184f02fd2715d7d07952222621461de55478fa3856747c";

/// Relative path used when looking for a local drop-in `.mrpack`.
pub const FEATURED_PACK: &str = "packs/Cobblemon vasst 1.0.0.mrpack";
pub const FEATURED_PACK_TITLE: &str = "Cobblemon Vasst";
pub const FEATURED_PACK_BLURB: &str =
	"Autorska paczka — parę kliknięć i możesz grać.";
/// Hosted next to the skin registry so the NSIS installer stays small.
/// Keep on the same host as [`SKINS_URL`] until the HTTPS cutover.
pub const FEATURED_PACK_URL: &str =
	"http://92.5.186.6/packs/Cobblemon-vasst.mrpack";
pub const FEATURED_PACK_VERSION: &str = "1.0.0";
pub const FEATURED_PACK_CACHE_NAME: &str = "cobblemon-vasst.mrpack";

/// Discord application ID. The "Playing …" label is the app name in the Discord
/// Developer Portal — create an application named **Octra App** and paste its ID here.
pub const DISCORD_APP_ID: &str = "1543732302264410152";
