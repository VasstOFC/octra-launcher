import { invoke } from "@tauri-apps/api/core";
import type {
  Account,
  AccountsFile,
  CreateInstance,
  DeviceCode,
  Instance,
  JavaRuntime,
  JavaStatus,
  Loader,
  LoaderVersions,
  ManifestVersion,
  ModrinthSearchResult,
  ModrinthProjectDetail,
  ModrinthPackVersionHit,
  PackUpdateInfo,
  ServerPingResult,
  ModrinthContentVersions,
  InstallContentResult,
  FeaturedPackInfo,
  ServerEntry,
  Settings,
  ContentFile,
  ContentKind,
  ContentUpdate,
  WorldEntry,
  CrashReport,
  CurseForgeInstanceHit,
  CreateLocalServer,
  LocalServerInfo,
  UpdateLocalServer,
  OfflineSkin,
  SkinLibraryEntry,
  SavePremiumLibraryReq,
  LocalSoftware,
  GithubReleaseCheck,
  AppInfo,
  ScreenshotEntry,
  GlobalScreenshotEntry,
  AccountSkin,
  CatalogGroup,
  McPlayerProfile,
  MojangNewsItem,
  RelayPeerInfo,
  LauncherInstanceHit,
} from "../types";

function unwrap<T>(p: Promise<T>): Promise<T> {
  return p.catch((e: unknown) => {
    throw new Error(typeof e === "string" ? e : String(e));
  });
}

export const api = {
  getSettings: () => unwrap(invoke<Settings>("get_settings")),
  saveSettings: (settings: Settings) =>
    unwrap(invoke<Settings>("save_settings", { settings })),
  getDataDir: () => unwrap(invoke<string>("get_data_dir")),
  getAppInfo: () => unwrap(invoke<AppInfo>("get_app_info")),
  scanJava: () => unwrap(invoke<JavaStatus>("scan_java")),
  downloadJava: (major: number) =>
    unwrap(invoke<JavaRuntime>("download_java", { major })),
  requiredJavaForVersion: (id: string) =>
    unwrap(invoke<number>("required_java_for_version", { id })),
  listInstances: () => unwrap(invoke<Instance[]>("list_instances")),
  getInstance: (id: string) => unwrap(invoke<Instance>("get_instance", { id })),
  createInstance: (req: CreateInstance) =>
    unwrap(invoke<Instance>("create_instance", { req })),
  updateInstance: (inst: Instance) =>
    unwrap(invoke<Instance>("update_instance", { inst })),
  deleteInstance: (id: string) => unwrap(invoke("delete_instance", { id })),
  unlinkInstancePack: (id: string) =>
    unwrap(invoke<Instance>("unlink_instance_pack", { id })),
  resyncInstancePack: (id: string, path?: string | null) =>
    unwrap(
      invoke<Instance>("resync_instance_pack", {
        id,
        path: path?.trim() ? path : null,
      }),
    ),
  setInstanceIconGlyph: (id: string, color: string, symbol: string) =>
    unwrap(
      invoke<Instance>("set_instance_icon_glyph", { id, color, symbol }),
    ),
  setInstanceIconBytes: (id: string, bytes: number[]) =>
    unwrap(invoke<Instance>("set_instance_icon_bytes", { id, bytes })),
  pickInstanceIconFile: (id: string) =>
    unwrap(invoke<Instance | null>("pick_instance_icon_file", { id })),
  pickProfileWallpaper: (id: string) =>
    unwrap(invoke<Instance | null>("pick_profile_wallpaper", { id })),
  setProfileWallpaper: (id: string, path: string) =>
    unwrap(invoke<Instance>("set_profile_wallpaper", { id, path })),
  setProfileWallpaperBytes: (id: string, bytes: number[]) =>
    unwrap(invoke<Instance>("set_profile_wallpaper_bytes", { id, bytes })),
  clearProfileWallpaper: (id: string) =>
    unwrap(invoke<Instance>("clear_profile_wallpaper", { id })),
  readInstanceWallpaper: (id: string) =>
    unwrap(invoke<string | null>("read_instance_wallpaper", { id })),
  listInstanceContent: (id: string) =>
    unwrap(invoke<ContentFile[]>("list_instance_content", { id })),
  toggleInstanceContent: (id: string, kind: ContentKind, name: string) =>
    unwrap(
      invoke<ContentFile[]>("toggle_instance_content", { id, kind, name }),
    ),
  deleteInstanceContent: (id: string, kind: ContentKind, name: string) =>
    unwrap(
      invoke<ContentFile[]>("delete_instance_content", { id, kind, name }),
    ),
  importLocalContent: (id: string, kind: ContentKind, path: string) =>
    unwrap(
      invoke<ContentFile[]>("import_local_content", { id, kind, path }),
    ),
  fetchMinecraftVersions: (includeAll = false) =>
    unwrap(
      invoke<ManifestVersion[]>("fetch_minecraft_versions", {
        includeAll,
      }),
    ),
  fetchLoaderVersions: (loader: Loader, gameVersion: string) =>
    unwrap(
      invoke<LoaderVersions>("fetch_loader_versions", { loader, gameVersion }),
    ),
  getAccounts: () => unwrap(invoke<AccountsFile>("get_accounts")),
  accountHasToken: (uuid: string) =>
    unwrap(invoke<boolean>("account_has_token", { uuid })),
  fetchMojangNews: () => unwrap(invoke<MojangNewsItem[]>("fetch_mojang_news")),
  setActiveAccount: (uuid: string) =>
    unwrap(invoke<AccountsFile>("set_active_account", { uuid })),
  logoutAccount: (uuid: string) =>
    unwrap(invoke<AccountsFile>("logout_account", { uuid })),
  addOfflineAccount: (name: string) =>
    unwrap(invoke<Account>("add_offline_account", { name })),
  getOfflineSkin: (uuid: string) =>
    unwrap(invoke<OfflineSkin>("get_offline_skin", { uuid })),
  saveOfflineSkin: (uuid: string, png: number[], model: string) =>
    unwrap(invoke<OfflineSkin>("save_offline_skin", { uuid, png, model })),
  setOfflineSkinModel: (uuid: string, model: string) =>
    unwrap(invoke<OfflineSkin>("set_offline_skin_model", { uuid, model })),
  resetOfflineSkin: (uuid: string) =>
    unwrap(invoke("reset_offline_skin", { uuid })),
  listOfflineSkinLibrary: (uuid: string) =>
    unwrap(invoke<SkinLibraryEntry[]>("list_offline_skin_library", { uuid })),
  addOfflineSkinLibraryEntry: (
    uuid: string,
    png: number[],
    model: string,
    name: string,
  ) =>
    unwrap(
      invoke<SkinLibraryEntry>("add_offline_skin_library_entry", {
        uuid,
        png,
        model,
        name,
      }),
    ),
  equipOfflineSkinLibraryEntry: (uuid: string, entryId: string) =>
    unwrap(
      invoke<OfflineSkin>("equip_offline_skin_library_entry", {
        uuid,
        entryId,
      }),
    ),
  deleteOfflineSkinLibraryEntry: (uuid: string, entryId: string) =>
    unwrap(
      invoke("delete_offline_skin_library_entry", { uuid, entryId }),
    ),
  setOfflineSkinLibraryModel: (
    uuid: string,
    entryId: string,
    model: string,
  ) =>
    unwrap(
      invoke<SkinLibraryEntry>("set_offline_skin_library_model", {
        uuid,
        entryId,
        model,
      }),
    ),
  listPremiumSkinLibrary: (uuid: string) =>
    unwrap(invoke<SkinLibraryEntry[]>("list_premium_skin_library", { uuid })),
  savePremiumSkinLibraryEntry: (uuid: string, req: SavePremiumLibraryReq) =>
    unwrap(
      invoke<SkinLibraryEntry>("save_premium_skin_library_entry", { uuid, req }),
    ),
  deletePremiumSkinLibraryEntry: (uuid: string, entryId: string) =>
    unwrap(
      invoke("delete_premium_skin_library_entry", { uuid, entryId }),
    ),
  syncPremiumSkinLibraryActive: (
    uuid: string,
    textureKey: string | null,
    variant: string,
    png?: number[] | null,
  ) =>
    unwrap(
      invoke<SkinLibraryEntry[]>("sync_premium_skin_library_active", {
        uuid,
        textureKey,
        variant,
        png: png ?? null,
      }),
    ),
  setPremiumSkinLibraryActive: (uuid: string, entryId: string) =>
    unwrap(
      invoke<SkinLibraryEntry[]>("set_premium_skin_library_active", {
        uuid,
        entryId,
      }),
    ),
  findPremiumSkinLibraryDuplicate: (
    uuid: string,
    textureKey: string | null,
    variant: string,
    capeId: string | null,
    png?: number[] | null,
  ) =>
    unwrap(
      invoke<SkinLibraryEntry | null>("find_premium_skin_library_duplicate", {
        uuid,
        textureKey,
        variant,
        capeId,
        png: png ?? null,
      }),
    ),
  startLogin: () => unwrap(invoke<DeviceCode>("start_login")),
  cancelLogin: () => unwrap(invoke("cancel_login")),
  installInstance: (id: string) =>
    unwrap(invoke<Instance>("install_instance", { id })),
  cancelInstall: () => unwrap(invoke("cancel_install")),
  launchInstance: (id: string) =>
    unwrap(invoke<number>("launch_instance", { id })),
  stopInstance: (id: string, accountUuid?: string) =>
    unwrap(invoke("stop_instance", { id, accountUuid: accountUuid ?? null })),
  readInstanceLog: (id: string) =>
    unwrap(invoke<string>("read_instance_log", { id })),
  instanceGameDir: (id: string) =>
    unwrap(invoke<string>("instance_game_dir", { id })),
  openInstanceFolder: (id: string) =>
    unwrap(invoke("open_instance_folder", { id })),
  openDataDir: () => unwrap(invoke("open_data_dir")),
  listServers: () => unwrap(invoke<ServerEntry[]>("list_servers")),
  saveServers: (servers: ServerEntry[]) =>
    unwrap(invoke<ServerEntry[]>("save_servers", { servers })),
  syncServersToInstance: (id: string) =>
    unwrap(invoke<number>("sync_servers_to_instance", { id })),
  pingServer: (address: string) =>
    unwrap(invoke<ServerPingResult>("ping_server", { address })),
  pickMrpackFile: (kind?: "zip" | "mrpack" | null) =>
    unwrap(invoke<string | null>("pick_mrpack_file", { kind: kind ?? null })),
  importMrpack: (path: string) =>
    unwrap(invoke<Instance>("import_mrpack", { path })),
  importModrinthPack: (query: string, iconUrl?: string | null) =>
    unwrap(
      invoke<Instance>("import_modrinth_pack", {
        query,
        iconUrl: iconUrl ?? null,
      }),
    ),
  readInstanceIcon: (id: string) =>
    unwrap(invoke<string | null>("read_instance_icon", { id })),
  searchModrinthPacks: (opts?: {
    query?: string;
    offset?: number;
    limit?: number;
    sort?: string;
  }) => unwrap(invoke<ModrinthSearchResult>("search_modrinth_packs", opts ?? {})),
  getModrinthProject: (slug: string) =>
    unwrap(invoke<ModrinthProjectDetail>("get_modrinth_project", { slug })),
  getModrinthPackVersions: (slug: string) =>
    unwrap(invoke<ModrinthPackVersionHit[]>("get_modrinth_pack_versions", { slug })),
  searchModrinthContent: (opts: {
    query?: string;
    offset?: number;
    limit?: number;
    sort?: string;
    projectType: string;
    gameVersion?: string;
    loader?: string;
  }) => unwrap(invoke<ModrinthSearchResult>("search_modrinth_content", opts)),
  listModrinthContentVersions: (
    id: string,
    slug: string,
    projectType: string,
  ) =>
    unwrap(
      invoke<ModrinthContentVersions>("list_modrinth_content_versions", {
        id,
        slug,
        projectType,
      }),
    ),
  installModrinthContent: (
    id: string,
    slug: string,
    projectType: string,
    versionId?: string,
    optionalProjectIds?: string[],
  ) =>
    unwrap(
      invoke<InstallContentResult>("install_modrinth_content", {
        id,
        slug,
        projectType,
        versionId,
        optionalProjectIds,
      }),
    ),
  getFeaturedPack: () => unwrap(invoke<FeaturedPackInfo>("get_featured_pack")),
  installFeaturedPack: () => unwrap(invoke<Instance>("install_featured_pack")),
  duplicateInstance: (id: string) =>
    unwrap(invoke<Instance>("duplicate_instance", { id })),
  listWorlds: (id: string) => unwrap(invoke<WorldEntry[]>("list_worlds", { id })),
  deleteWorld: (id: string, folder: string) =>
    unwrap(invoke<WorldEntry[]>("delete_world", { id, folder })),
  copyWorld: (fromId: string, folder: string, toId: string) =>
    unwrap(invoke<WorldEntry[]>("copy_world", { fromId, folder, toId })),
  openWorldFolder: (id: string, folder: string) =>
    unwrap(invoke("open_world_folder", { id, folder })),
  listCrashReports: (id: string) =>
    unwrap(invoke<CrashReport[]>("list_crash_reports", { id })),
  openInstanceSubdir: (id: string, folder: string) =>
    unwrap(invoke("open_instance_subdir", { id, folder })),
  pickJavaExe: () => unwrap(invoke<string | null>("pick_java_exe")),
  pickDirectory: () => unwrap(invoke<string | null>("pick_directory")),
  scanCurseforgeInstances: (root?: string | null) =>
    unwrap(
      invoke<CurseForgeInstanceHit[]>("scan_curseforge_instances", {
        root: root ?? null,
      }),
    ),
  importCurseforgeInstance: (path: string) =>
    unwrap(invoke<Instance>("import_curseforge_instance", { path })),
  probeJavaPath: (path: string) =>
    unwrap(invoke<JavaRuntime | null>("probe_java_path", { path })),
  purgeCache: () => unwrap(invoke("purge_cache")),
  exportMrpack: (id: string) =>
    unwrap(invoke<string | null>("export_mrpack", { id })),
  checkContentUpdates: (id: string) =>
    unwrap(invoke<ContentUpdate[]>("check_content_updates", { id })),
  checkPackUpdate: (id: string) =>
    unwrap(invoke<PackUpdateInfo>("check_pack_update", { id })),
  listLocalServers: () => unwrap(invoke<LocalServerInfo[]>("list_local_servers")),
  getLocalServer: (id: string) =>
    unwrap(invoke<LocalServerInfo>("get_local_server", { id })),
    createLocalServer: (req: CreateLocalServer) =>
      unwrap(invoke<LocalServerInfo>("create_local_server", { req })),
    probeLocalServer: (opts: {
      software: LocalSoftware;
      gameVersion: string;
      loaderVersion?: string | null;
    }) =>
      unwrap(
        invoke("probe_local_server", {
          software: opts.software,
          gameVersion: opts.gameVersion,
          loaderVersion: opts.loaderVersion ?? null,
        }),
      ),
    listPaperVersions: () => unwrap(invoke<string[]>("list_paper_versions")),
  updateLocalServer: (id: string, patch: UpdateLocalServer) =>
    unwrap(invoke<LocalServerInfo>("update_local_server", { id, patch })),
  deleteLocalServer: (id: string) => unwrap(invoke("delete_local_server", { id })),
  installLocalServer: (id: string) =>
    unwrap(invoke<LocalServerInfo>("install_local_server", { id })),
  startLocalServer: (id: string) =>
    unwrap(invoke<LocalServerInfo>("start_local_server", { id })),
  stopLocalServer: (id: string) => unwrap(invoke("stop_local_server", { id })),
  sendLocalServerCommand: (id: string, command: string) =>
    unwrap(invoke("send_local_server_command", { id, command })),
  readLocalServerLog: (id: string) =>
    unwrap(invoke<string>("read_local_server_log", { id })),
  openLocalServerFolder: (id: string) =>
    unwrap(invoke("open_local_server_folder", { id })),
  openLocalServerBackups: (id: string) =>
    unwrap(invoke("open_local_server_backups", { id })),
  openLocalServerProperties: (id: string) =>
    unwrap(invoke("open_local_server_properties", { id })),
  backupLocalServerWorld: (id: string) =>
    unwrap(invoke<string>("backup_local_server_world", { id })),
  checkGithubRelease: () =>
    unwrap(invoke<GithubReleaseCheck>("check_github_release")),
  skinsLanUrl: () => unwrap(invoke<string | null>("skins_lan_url")),
  listScreenshots: (id: string) =>
    unwrap(invoke<ScreenshotEntry[]>("list_screenshots", { id })),
  readScreenshot: (id: string, name: string, full?: boolean) =>
    unwrap(invoke<string>("read_screenshot", { id, name, full: full ?? false })),
  saveScreenshotAs: (id: string, name: string) =>
    unwrap(invoke<string | null>("save_screenshot_as", { id, name })),
  listAllScreenshots: () =>
    unwrap(invoke<GlobalScreenshotEntry[]>("list_all_screenshots")),
  getAccountSkin: (uuid: string, refresh = false) =>
    unwrap(
      invoke<AccountSkin>("get_account_skin", { uuid, refresh }),
    ),
  getMojangSkinCatalog: () =>
    unwrap(invoke<CatalogGroup[]>("get_mojang_skin_catalog")),
  getMinecraftProfile: (uuid: string, refresh = false) =>
    unwrap(invoke<McPlayerProfile>("get_minecraft_profile", { uuid, refresh })),
  equipMojangSkin: (uuid: string, textureKey: string, variant: string) =>
    unwrap(
      invoke<McPlayerProfile>("equip_mojang_skin", { uuid, textureKey, variant }),
    ),
  uploadMojangSkin: (uuid: string, png: number[], variant: string) =>
    unwrap(invoke<McPlayerProfile>("upload_mojang_skin", { uuid, png, variant })),
  setMinecraftCape: (uuid: string, capeId: string | null) =>
    unwrap(invoke<McPlayerProfile>("set_minecraft_cape", { uuid, capeId })),
  getMojangTexturePreview: (textureKey: string) =>
    unwrap(invoke<string>("get_mojang_texture_preview", { textureKey })),
  getAvailableSkins: (uuid: string) =>
    unwrap(invoke<import("./skins").Skin[]>("get_available_skins", { uuid })),
  getAvailableCapes: (uuid: string) =>
    unwrap(invoke<import("./skins").Cape[]>("get_available_capes", { uuid })),
  equipSkin: (uuid: string, skin: import("./skins").Skin, png?: number[] | null) =>
    unwrap(
      invoke<McPlayerProfile | null>("equip_skin", {
        req: { uuid, skin, png: png ?? null },
      }),
    ),
  saveCustomSkin: (
    uuid: string,
    skin: import("./skins").Skin,
    variant: string,
    opts?: { capeId?: string | null; png?: number[] | null; replaceTexture?: boolean },
  ) =>
    unwrap(
      invoke<SkinLibraryEntry>("save_custom_skin", {
        req: {
          uuid,
          skin,
          variant,
          capeId: opts?.capeId ?? null,
          png: opts?.png ?? null,
          replaceTexture: opts?.replaceTexture ?? true,
        },
      }),
    ),
  removeCustomSkin: (uuid: string, skin: import("./skins").Skin) =>
    unwrap(invoke("remove_custom_skin", { uuid, skin })),
  normalizeSkinTexture: (texture: string | number[]) =>
    unwrap(invoke<number[]>("normalize_skin_texture", { texture })),
  flushPendingSkinChange: () => unwrap(invoke("flush_pending_skin_change")),
  fetchImageBase64: (url: string) =>
    unwrap(invoke<string>("fetch_image_base64", { url })),
  relayStart: (name: string) => unwrap(invoke("relay_start", { name })),
  relayStop: () => unwrap(invoke("relay_stop")),
  relayListPeers: () => unwrap(invoke<RelayPeerInfo[]>("relay_list_peers")),
  relaySend: (peerId: string, text: string) =>
    unwrap(invoke("relay_send", { peerId, text })),
  scanPrismInstances: () =>
    unwrap(invoke<LauncherInstanceHit[]>("scan_prism_instances")),
  scanMultimcInstances: (root?: string | null) =>
    unwrap(invoke<LauncherInstanceHit[]>("scan_multimc_instances", { root: root ?? null })),
  importLauncherInstance: (path: string, source: string) =>
    unwrap(invoke<Instance>("import_launcher_instance", { path, source })),
};
