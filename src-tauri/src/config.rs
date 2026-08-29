//! Stałe konfiguracyjne Octra. Client ID jest publiczny (OAuth public client)
//! i idzie razem z launcherem — znajomi nie muszą nic wpisywać w ustawieniach.

/// Application (client) ID z Azure Entra ID.
/// Public client, konta osobiste Microsoft, „Allow public client flows” = Yes.
/// Recenzja Mojang: https://aka.ms/mce-reviewappid
///
/// Wklej tutaj swój ID, potem commituj — każdy build będzie go miał w sobie.
pub const AZURE_CLIENT_ID: &str = "bfe8ec3a-8e09-4be9-bbef-68f0fa0d1321";

/// Autorska paczka (Cobblemon Vasst): względna ścieżka do `.mrpack`.
/// Launcher szuka pliku przy data dir, exe i zasobach Tauri.
pub const FEATURED_PACK: &str = "packs/Cobblemon vasst 1.0.0.mrpack";
pub const FEATURED_PACK_TITLE: &str = "Cobblemon Vasst";
pub const FEATURED_PACK_BLURB: &str =
    "Autorska paczka — parę kliknięć i możesz grać.";
pub const FEATURED_SERVER_NAME: &str = "Serwer";
/// Adres serwera (host albo host:port). Puste = bez auto-join.
pub const FEATURED_SERVER_ADDRESS: &str = "";

/// HTTP serwer skinów Octra (nie ely.by / LittleSkin).
/// Pusty = tylko własny skin + LAN. Wpisz tu albo w Ustawieniach, jeśli
/// znajomi poza siecią lokalną mają widzieć skiny offline.
pub const LUMEN_SKINS_URL: &str = "";

/// Discord Application ID (Rich Presence). Placeholder — zamień na własne.
pub const DISCORD_APP_ID: &str = "142857142857142857";
