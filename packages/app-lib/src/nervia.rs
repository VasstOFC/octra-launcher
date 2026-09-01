//! Octra App identity (workspace folder is still "Nervia App").
//!
//! Minecraft sign-in uses the Xbox SISU flow. Catalog still talks to Modrinth.
//! Octra accounts are not live yet — the UI shows them as coming soon.

/// Azure public client used by Octra (device-code / public-client flows).
pub const AZURE_CLIENT_ID: &str = "bfe8ec3a-8e09-4be9-bbef-68f0fa0d1321";

/// HTTP skin registry (Oracle VPS, Caddy :80 → localhost:8787).
pub const SKINS_URL: &str = "http://92.5.186.6";

/// Header `X-Octra-Key` for skin uploads. Must match `/etc/octra-skins.env`.
pub const SKINS_API_KEY: &str =
    "73184f02fd2715d7d07952222621461de55478fa3856747c";

/// Relative path used when looking for a local drop-in `.mrpack`.
pub const FEATURED_PACK: &str = "packs/Cobblemon vasst 1.0.0.mrpack";
pub const FEATURED_PACK_TITLE: &str = "Cobblemon Vasst";
pub const FEATURED_PACK_BLURB: &str =
    "Autorska paczka — parę kliknięć i możesz grać.";
/// Hosted next to the skin registry so the NSIS installer stays small.
pub const FEATURED_PACK_URL: &str =
    "http://92.5.186.6/packs/Cobblemon-vasst.mrpack";
pub const FEATURED_PACK_VERSION: &str = "1.0.0";
pub const FEATURED_PACK_CACHE_NAME: &str = "cobblemon-vasst.mrpack";

/// Discord application ID. The "Playing …" label is the app name in the Discord
/// Developer Portal — create an application named **Octra App** and paste its ID here.
pub const DISCORD_APP_ID: &str = "1543732302264410152";
