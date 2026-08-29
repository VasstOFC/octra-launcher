export type Loader = "vanilla" | "fabric" | "quilt" | "forge" | "neoforge";

export interface Instance {
  id: string;
  name: string;
  gameVersion: string;
  loader: Loader;
  loaderVersion?: string | null;
  versionId: string;
  createdAt: string;
  lastPlayed?: string | null;
  memoryMaxMb: number;
  memoryMinMb: number;
  javaPath?: string | null;
  javaArgs: string;
  joinServer?: string;
  iconColor?: string;
  iconSymbol?: string;
  iconPath?: string | null;
  wallpaperPath?: string | null;
  ledColor?: string;
  ledColor2?: string;
  playTimeSecs: number;
  linkedPack?: string | null;
  packLocked?: boolean;
  customJava?: boolean;
  customMemory?: boolean;
  customJavaArgs?: boolean;
  customEnv?: boolean;
  customWindow?: boolean;
  customHooks?: boolean;
  fullscreen?: boolean;
  windowWidth?: number;
  windowHeight?: number;
  envVars?: string;
  preLaunch?: string;
  wrapper?: string;
  postExit?: string;
}

export type ContentKind = "mods" | "resourcepacks" | "shaderpacks" | "datapacks";

export interface ContentFile {
  name: string;
  displayName: string;
  enabled: boolean;
  kind: ContentKind;
  size: number;
  slug?: string | null;
  projectId?: string | null;
}

export interface ContentUpdate {
  name: string;
  kind: ContentKind;
  fileName: string;
  slug?: string | null;
  projectTitle?: string | null;
  currentVersion?: string | null;
  latestVersion?: string | null;
}

export interface WorldEntry {
  name: string;
  folder: string;
  size: number;
}

export interface CurseForgeInstanceHit {
  path: string;
  name: string;
  gameVersion: string;
  loader: Loader;
  loaderVersion?: string | null;
}

export interface CrashReport {
  name: string;
  size: number;
  modified?: string | null;
}

export interface ScreenshotEntry {
  name: string;
  size: number;
  modified?: string | null;
}

export interface GlobalScreenshotEntry extends ScreenshotEntry {
  instanceId: string;
  instanceName: string;
}

export interface AccountSkin {
  uuid: string;
  model: string;
  textureUrl?: string | null;
  capeUrl?: string | null;
  pngBase64?: string | null;
  capePngBase64?: string | null;
  isPremium: boolean;
}

export interface CatalogSkin {
  id: string;
  name: string;
  textureKey: string;
  variant: string;
}

export interface CatalogGroup {
  id: string;
  title: string;
  skins: CatalogSkin[];
}

export interface McCape {
  id: string;
  state: string;
  url: string;
  alias?: string | null;
}

export interface McOwnedSkin {
  id: string;
  state: string;
  url: string;
  variant: string;
  alias?: string | null;
  textureKey?: string | null;
}

export interface McPlayerProfile {
  id: string;
  name: string;
  skins: McOwnedSkin[];
  capes: McCape[];
}

export interface RelayPeerInfo {
  id: string;
  name: string;
  addr: string;
  online: boolean;
}

export interface LauncherInstanceHit {
  path: string;
  name: string;
  gameVersion: string;
  loader: Loader;
  loaderVersion?: string | null;
  source: string;
}

export interface CreateInstance {
  name: string;
  gameVersion: string;
  loader: Loader;
  loaderVersion?: string | null;
  memoryMaxMb?: number;
}

export type ColorTheme = "dark" | "oled" | "light" | "system";

export type AccentPreset = "violet" | "cyber" | "ember" | "mono";

export type BrandAccent = "lilac" | "cyan" | "green" | "coral" | "gold";

export type AppChannel = "stable" | "beta" | "dev";

export interface AppInfo {
  version: string;
  channel: AppChannel;
  displayName: string;
  dataDir: string;
  updatesEnabled: boolean;
}

export interface Settings {
  memoryMaxMb: number;
  memoryMinMb: number;
  javaPath?: string | null;
  javaMode: string;
  azureClientId: string;
  showSnapshots: boolean;
  closeOnLaunch: boolean;
  dataDir?: string | null;
  featuredPack?: string;
  featuredPackTitle?: string;
  featuredServerName?: string;
  featuredServerAddress?: string;
  theme?: ColorTheme | string;
  accentColor?: BrandAccent | string;
  accentPreset?: AccentPreset | string;
  advancedRendering?: boolean;
  systemWindowFrame?: boolean;
  compactLibrary?: boolean;
  showPlayTime?: boolean;
  jumpInto?: boolean;
  warnUnknownMrpack?: boolean;
  skipNonEssentialWarnings?: boolean;
  defaultFullscreen?: boolean;
  defaultWindowWidth?: number;
  defaultWindowHeight?: number;
  defaultJavaArgs?: string;
  defaultEnvVars?: string;
  java8Path?: string | null;
  java17Path?: string | null;
  java21Path?: string | null;
  java25Path?: string | null;
  maxConcurrentDownloads?: number;
  maxConcurrentWrites?: number;
  skinsUrl?: string;
  hideToTray?: boolean;
  discordRpc?: boolean;
  autoCheckUpdates?: boolean;
}

export interface MojangNewsItem {
  id: string;
  title: string;
  link: string;
  summary: string;
  published: string;
}

export interface Account {
  uuid: string;
  name: string;
  xuid: string;
  kind?: "microsoft" | "offline";
}

export interface OfflineSkin {
  uuid: string;
  model: string;
  pngBase64?: string | null;
  uploadedAt?: string | null;
  hasCustom: boolean;
}

export interface AccountsFile {
  active?: string | null;
  accounts: Account[];
}

export interface ManifestVersion {
  id: string;
  versionType: string;
  url: string;
  time?: string;
  releaseTime?: string;
  sha1?: string;
}

export interface JavaRuntime {
  path: string;
  major: number;
  vendor: string;
  source: string;
}

export interface JavaStatus {
  runtimes: JavaRuntime[];
  memoryMb: number;
}

export interface InstallProgress {
  instanceId: string;
  stage: string;
  current: number;
  total: number;
  file?: string | null;
  message: string;
}

export interface DeviceCode {
  userCode: string;
  deviceCode: string;
  verificationUri: string;
  verificationUriComplete?: string | null;
  expiresIn: number;
  interval: number;
  message?: string | null;
}

export interface LoaderVersions {
  versions: string[];
  recommended?: string | null;
}

export interface ServerEntry {
  name: string;
  address: string;
}

export interface ModrinthPackHit {
  slug: string;
  projectId?: string | null;
  title: string;
  description: string;
  iconUrl?: string | null;
  downloads: number;
  follows: number;
  categories: string[];
  loaders: string[];
  gameVersions: string[];
  author?: string | null;
}

export interface FeaturedPackInfo {
  enabled: boolean;
  title: string;
  blurb: string;
  serverName: string;
  serverAddress: string;
}

export interface ModrinthSearchResult {
  hits: ModrinthPackHit[];
  offset: number;
  limit: number;
  totalHits: number;
}

export interface ModrinthContentDep {
  projectId?: string | null;
  versionId?: string | null;
  dependencyType: string;
  title?: string | null;
  slug?: string | null;
}

export interface ModrinthContentVersion {
  id: string;
  versionNumber: string;
  versionName: string;
  versionType: string;
  gameVersions: string[];
  loaders: string[];
  datePublished?: string | null;
  downloads: number;
  dependencies: ModrinthContentDep[];
}

export interface ModrinthContentVersions {
  projectTitle: string;
  projectSlug: string;
  projectType: string;
  versions: ModrinthContentVersion[];
}

export interface InstallContentResult {
  files: ContentFile[];
  warnings: string[];
}

export type LocalSoftware = "vanilla" | "paper" | "fabric";

export interface LocalServerInfo {
  id: string;
  name: string;
  software: LocalSoftware;
  gameVersion: string;
  loaderVersion?: string | null;
  memoryMb: number;
  port: number;
  motd: string;
  onlineMode: boolean;
  maxPlayers: number;
  difficulty: string;
  viewDistance: number;
  eulaAccepted: boolean;
  sourceInstanceId?: string | null;
  javaPath?: string | null;
  requiredJava: number;
  jarReady: boolean;
  status: string;
  pid?: number | null;
  address: string;
  lanIp?: string | null;
  createdAt: string;
}

export interface CreateLocalServer {
  name: string;
  gameVersion: string;
  software: LocalSoftware;
  loaderVersion?: string | null;
  memoryMb?: number;
  port?: number;
  motd?: string;
  onlineMode: boolean;
  maxPlayers?: number;
  difficulty?: string;
  viewDistance?: number;
  eulaAccepted: boolean;
  sourceInstanceId?: string | null;
  javaPath?: string | null;
}

export interface UpdateLocalServer {
  name?: string;
  memoryMb?: number;
  port?: number;
  motd?: string;
  onlineMode?: boolean;
  maxPlayers?: number;
  difficulty?: string;
  viewDistance?: number;
  javaPath?: string | null;
}

export interface LocalServerStatusEvent {
  serverId: string;
  name: string;
  status: string;
  port: number;
  pid?: number | null;
}

export interface LocalServerLogEvent {
  serverId: string;
  line: string;
}

export interface LocalServerProgressEvent {
  serverId: string;
  message: string;
}

export type GithubReleaseCheck =
  | { kind: "notFound" }
  | { kind: "current"; version: string; tagName: string; htmlUrl: string }
  | {
      kind: "newer";
      version: string;
      tagName: string;
      htmlUrl: string;
      installerUrl: string | null;
      installerName: string | null;
      notes: string;
      hasLatestJson: boolean;
    }
  | {
      kind: "unversioned";
      tagName: string;
      name: string;
      htmlUrl: string;
      installerUrl: string | null;
    };
