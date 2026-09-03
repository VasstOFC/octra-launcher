<script setup>
import {
	AuthFeature,
	ModrinthApiError,
	NodeAuthFeature,
	nodeAuthState,
	PanelVersionFeature,
	TauriModrinthClient,
	VerboseLoggingFeature,
} from '@modrinth/api-client'
import {
	ArrowBigUpDashIcon,
	ChevronLeftIcon,
	ChevronRightIcon,
	CompassIcon,
	GlobeIcon,
	ImagesIcon,
	LogInIcon,
	LogOutIcon,
	MessageIcon,
	PlayIcon,
	PlusIcon,
	RefreshCwIcon,
	SettingsIcon,
	ShirtIcon,
	TrashIcon,
	UserIcon,
	UserPlusIcon,
	UsersIcon,
} from '@modrinth/assets'
import {
	AccountSwitchOverlay,
	Admonition,
	Avatar,
	commonMessages,
	ContentInstallModal,
	ContentUpdaterModal,
	CreationFlowModal,
	defineMessages,
	I18nDebugPanel,
	IconButton,
	LoadingBar,
	NotificationPanel,
	PopupNotificationPanel,
	provideModalBehavior,
	provideModrinthClient,
	provideNotificationManager,
	providePageContext,
	providePopupNotificationManager,
	TeleportOverflowMenu,
	useDebugLogger,
	useFormatBytes,
	useHostingIntercom,
	useVIntl,
} from '@modrinth/ui'
import { renderString } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { getVersion } from '@tauri-apps/api/app'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { openUrl } from '@tauri-apps/plugin-opener'
import { type } from '@tauri-apps/plugin-os'
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'
import { computed, nextTick, onMounted, onUnmounted, provide, ref, watch } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'

import OctraWordmark from '@/components/brand/OctraWordmark.vue'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import AddOfflineAccountModal from '@/components/ui/AddOfflineAccountModal.vue'
import AppActionBar from '@/components/ui/AppActionBar.vue'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import ErrorModal from '@/components/ui/ErrorModal.vue'
import OctraChatPanel from '@/components/ui/friends/OctraChatPanel.vue'
import OctraCommunityList from '@/components/ui/friends/OctraCommunityList.vue'
import HostingUpdateRequired from '@/components/ui/HostingUpdateRequired.vue'
import AddServerToInstanceModal from '@/components/ui/install_flow/AddServerToInstanceModal.vue'
import UnknownPackWarningModal from '@/components/ui/install_flow/UnknownPackWarningModal.vue'
import IconEditorModal from '@/components/ui/instance_settings/icon-editor-modal/index.vue'
import MinecraftAuthErrorModal from '@/components/ui/minecraft-auth-error-modal/MinecraftAuthErrorModal.vue'
import MinecraftRequiredModal from '@/components/ui/minecraft-required-modal/MinecraftRequiredModal.vue'
import AppSettingsModal from '@/components/ui/modal/AppSettingsModal.vue'
import InstallToPlayModal from '@/components/ui/modal/InstallToPlayModal.vue'
import ModpackAlreadyInstalledModal from '@/components/ui/modal/ModpackAlreadyInstalledModal.vue'
import ModrinthAccountRequiredModal from '@/components/ui/modal/ModrinthAccountRequiredModal.vue'
import UpdateToPlayModal from '@/components/ui/modal/UpdateToPlayModal.vue'
import NavButton from '@/components/ui/NavButton.vue'
import NewIconEditorNotification from '@/components/ui/new-icon-editor-notification/index.vue'
import { shouldShowNewIconEditorNotification } from '@/components/ui/new-icon-editor-notification/show-notification'
import OctraAccountModal from '@/components/ui/OctraAccountModal.vue'
import PromotionWrapper from '@/components/ui/PromotionWrapper.vue'
import QuickInstanceSwitcher from '@/components/ui/QuickInstanceSwitcher.vue'
import SharedInstanceInviteHandler from '@/components/ui/shared-instances/shared-instance-invite-handler/index.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import SurveyPopup from '@/components/ui/SurveyPopup.vue'
import WhatsNewModal from '@/components/ui/WhatsNewModal.vue'
import WindowControls from '@/components/ui/WindowControls.vue'
import { useCheckDisableMouseover } from '@/composables/macCssFix.js'
import { bootstrapAccent } from '@/composables/use-accent.ts'
import { useAppEvent } from '@/composables/use-app-event'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { useError } from '@/composables/use-error.js'
import { useMinecraftAccountAvatar } from '@/composables/use-minecraft-account-avatar.ts'
import { isDarkTheme, useTheme } from '@/composables/use-theme.ts'
import { config } from '@/config'
import { rememberAccountAppearance } from '@/helpers/account-appearance.ts'
import { hide_ads_window, release_ads_window_hold, take_ads_window_hold } from '@/helpers/ads.js'
import { trackEvent } from '@/helpers/analytics'
import {
	check_reachable,
	get_default_user,
	isOfflineAccount,
	login as loginMinecraft,
	remove_user,
	set_default_user,
	users as listMinecraftUsers,
} from '@/helpers/auth.js'
import { get_user, get_version } from '@/helpers/cache.js'
import { install_create_modpack_instance, install_get_modpack_preview } from '@/helpers/install'
import {
	can_current_user_use_shared_instances,
	get as getInstance,
	get_global_synced_options,
	list as listInstances,
	run,
} from '@/helpers/instance'
import { get as getCreds, removeUser } from '@/helpers/mr_auth.ts'
import {
	octraAccountLogout,
	octraAccountSession,
	octraChatChannels,
} from '@/helpers/octra-account.js'
import { get_all as getAllProcesses } from '@/helpers/process'
import { mergeUrlQuery, parseModrinthLink } from '@/helpers/project-links.ts'
import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import { get_opening_command, initialize_state } from '@/helpers/state'
import { parse_modrinth_user_link } from '@/helpers/users'
import {
	areUpdatesEnabled,
	enqueueUpdateForInstallation,
	getOS,
	getUpdateSize,
	isDev,
	isNetworkMetered,
	setRestartAfterPendingUpdate,
} from '@/helpers/utils.js'
import { start_join_server, start_join_singleplayer_world } from '@/helpers/worlds.ts'
import i18n, { DEFAULT_APP_LOCALE } from '@/i18n.config'
import { instanceKeys } from '@/pages/instance/query-options'
import {
	appUpdateState,
	downloadAvailableAppUpdate,
	getNextAppUpdatePopupTime,
	installAvailableAppUpdate,
	markAppUpdateActionable,
	markAppUpdatePopupShown,
	openAppUpdateChangelog,
	setAppUpdateActions,
} from '@/providers/app-update.ts'
import { createBreadcrumbManager, provideBreadcrumbManager } from '@/providers/breadcrumbs'
import { createContentInstall, provideContentInstall } from '@/providers/content-install'
import {
	provideAppUpdateDownloadProgress,
	subscribeToDownloadProgress,
} from '@/providers/download-progress.ts'
import { createServerInstall, provideServerInstall } from '@/providers/server-install'
import { setupProviders } from '@/providers/setup'
import { setupAppEventsProvider } from '@/providers/setup/app-events'
import { setupAuthProvider } from '@/providers/setup/auth'
import { setupLoadingStateProvider } from '@/providers/setup/loading-state'
import { setupAppUserPreferencesProvider } from '@/providers/setup/user-preferences.ts'
import { appMessages } from '@/utils/app-messages'

import { generateSkinPreviews } from './helpers/rendering/batch-skin-renderer'
import { get_available_capes, get_available_skins } from './helpers/skins'
import { AppNotificationManager } from './providers/app-notifications'
import { AppPopupNotificationManager } from './providers/app-popup-notifications'
import {
	appSettingsModalOpenLaunchDefaultsKey,
	appSettingsModalOpenProfileKey,
	appSettingsModalOpenSyncedOptionsKey,
} from './providers/app-settings-modal'

const appSettings = useAppSettings()
const appTheme = useTheme()
const router = useRouter()
const route = useRoute()
const { channel: appEventChannel, events: appEvents } = setupAppEventsProvider()
const breadcrumbManager = createBreadcrumbManager()
provideBreadcrumbManager(breadcrumbManager)
const canNavigateBack = ref(false)
const canNavigateForward = ref(false)

function updateHistoryNavigationState() {
	const historyState = window.history.state
	canNavigateBack.value = historyState?.back != null
	canNavigateForward.value = historyState?.forward != null
}

let fullscreenAdsWindowHold = false

async function handleFullscreenChange() {
	const fullscreen = document.fullscreenElement !== null
	if (fullscreen === fullscreenAdsWindowHold) return

	fullscreenAdsWindowHold = fullscreen
	try {
		if (fullscreen) {
			await take_ads_window_hold()
		} else {
			await release_ads_window_hold()
		}
	} catch (error) {
		fullscreenAdsWindowHold = !fullscreen
		handleError(error)
	}
}

updateHistoryNavigationState()

const APP_LEFT_NAV_WIDTH = '4rem'
const APP_SIDEBAR_WIDTH = 300
const INTERCOM_BUBBLE_DEFAULT_PADDING = 20
const credentials = ref()
let credentialsRefreshId = 0
const forceSidebar = computed(
	() => route.path.startsWith('/browse') || route.path.startsWith('/project'),
)
const SIDEBAR_STORAGE_KEY = 'octra.sidebarExpanded'
const sidebarExpandedPreference = ref(localStorage.getItem(SIDEBAR_STORAGE_KEY) === '1')
watch(sidebarExpandedPreference, (value) => {
	localStorage.setItem(SIDEBAR_STORAGE_KEY, value ? '1' : '0')
})
const sidebarVisible = computed(() => forceSidebar.value || sidebarExpandedPreference.value)
const chatPanelOpen = ref(false)
const octraChatPanel = ref(null)
const chatUnreadTotal = ref(0)
const lastPolledUnreadTotal = ref(0)
const chatUnreadReady = ref(false)
let chatUnreadPollTimer = null
const runningInstanceName = ref(null)
const whatsNewModal = ref(null)
const displayedAppVersion = ref('')

async function openOctraChatDm(userId) {
	chatPanelOpen.value = true
	await nextTick()
	await octraChatPanel.value?.openDm?.(userId)
}

function dismissChatPanelFromViewport() {
	if (chatPanelOpen.value) {
		chatPanelOpen.value = false
	}
}

function onChatUnreadChanged(total) {
	const next = Number(total) || 0
	if (!chatPanelOpen.value && chatUnreadReady.value && next > chatUnreadTotal.value) {
		addNotification({
			title: formatMessage(messages.chatNewMessage),
			text: '',
			type: 'success',
		})
	}
	chatUnreadTotal.value = next
	lastPolledUnreadTotal.value = next
	chatUnreadReady.value = true
}

async function pollChatUnread() {
	if (!octraSession.value) {
		chatUnreadTotal.value = 0
		lastPolledUnreadTotal.value = 0
		return
	}
	try {
		const channels = await octraChatChannels()
		const total = (channels || []).reduce((sum, channel) => sum + (channel.unread_count ?? 0), 0)
		if (!chatPanelOpen.value && chatUnreadReady.value && total > lastPolledUnreadTotal.value) {
			addNotification({
				title: formatMessage(messages.chatNewMessage),
				text: '',
				type: 'success',
			})
		}
		chatUnreadTotal.value = total
		lastPolledUnreadTotal.value = total
		chatUnreadReady.value = true
	} catch {
		// ignore transient chat poll failures
	}
}

function stopChatUnreadPoll() {
	if (chatUnreadPollTimer) {
		clearInterval(chatUnreadPollTimer)
		chatUnreadPollTimer = null
	}
}

function startChatUnreadPoll() {
	stopChatUnreadPoll()
	void pollChatUnread()
	chatUnreadPollTimer = setInterval(() => {
		void pollChatUnread()
	}, 15_000)
}

async function refreshRunningInstancePresence() {
	try {
		const processes = (await getAllProcesses().catch(() => [])) ?? []
		if (!Array.isArray(processes) || processes.length === 0) {
			runningInstanceName.value = null
			return
		}
		const instanceId = processes[0]?.instance_id
		if (!instanceId) {
			runningInstanceName.value = null
			return
		}
		const instances = (await listInstances().catch(() => [])) ?? []
		const match = instances.find((instance) => instance.id === instanceId)
		runningInstanceName.value = match?.name || ''
	} catch {
		runningInstanceName.value = null
	}
}

const canToggleSidebar = computed(() => !forceSidebar.value)
const showFriendsFab = computed(() => canToggleSidebar.value && !sidebarVisible.value)
const hostingRouteActive = computed(() => route.path.startsWith('/hosting'))
const hostingUpdateRequired = computed(
	() =>
		hostingRouteActive.value &&
		!!appUpdateState.availableUpdate.value &&
		appUpdateState.updatesEnabled.value,
)
const hostingIntercomIdentityKey = computed(() => {
	const rawServerId = route.params.id
	const serverId = Array.isArray(rawServerId) ? rawServerId[0] : rawServerId
	const userId = credentials.value?.user_id ?? credentials.value?.user?.id ?? 'anonymous'
	return `${userId}:${serverId ?? 'hosting'}`
})
const hostingIntercom = useHostingIntercom({
	enabled: computed(
		() => hostingRouteActive.value && !hostingUpdateRequired.value && !!credentials.value?.session,
	),
	appId: 'ykeritl9',
	fetchToken: fetchIntercomToken,
	identityKey: hostingIntercomIdentityKey,
	horizontalPadding: computed(() =>
		sidebarVisible.value
			? APP_SIDEBAR_WIDTH + INTERCOM_BUBBLE_DEFAULT_PADDING
			: INTERCOM_BUBBLE_DEFAULT_PADDING,
	),
})

const notificationManager = new AppNotificationManager()
provideNotificationManager(notificationManager)
const { handleError, addNotification } = notificationManager

useAppEvent(
	'warning',
	(event) =>
		addNotification({
			title: formatMessage(messages.warning),
			text: event.message,
			type: 'warning',
		}),
	appEvents,
)

const popupNotificationManager = new AppPopupNotificationManager()
providePopupNotificationManager(popupNotificationManager)
const { addPopupNotification } = popupNotificationManager
useAppEvent('ads_consent_required', handleAdsConsentRequired, appEvents)

const appVersion = getVersion()
const tauriApiClient = new TauriModrinthClient({
	userAgent: async () => `octra/theseus/${await appVersion}`,
	labrinthBaseUrl: config.labrinthBaseUrl,
	archonBaseUrl: config.archonBaseUrl,
	sharedInstancesBaseUrl: config.sharedInstancesBaseUrl,
	features: [
		new NodeAuthFeature({
			getAuth: () => nodeAuthState.getAuth?.() ?? null,
			refreshAuth: async () => {
				if (nodeAuthState.refreshAuth) {
					await nodeAuthState.refreshAuth()
				}
			},
		}),
		new AuthFeature({
			token: async () => (await getCreds())?.session,
		}),
		new PanelVersionFeature(),
		new VerboseLoggingFeature(),
	],
})
provideModrinthClient(tauriApiClient)
useQuery({
	queryKey: computed(() => ['authenticated-user', 'campaigns', credentials.value?.user?.id]),
	queryFn: () => tauriApiClient.labrinth.users_v3.getAuthenticated(),
	enabled: () => !!credentials.value?.session,
	retry: false,
})
useQuery({
	queryKey: computed(() => instanceKeys.sharedEligibility(credentials.value?.user?.id)),
	queryFn: can_current_user_use_shared_instances,
	enabled: () => !!credentials.value?.session && !!credentials.value?.user?.id,
	retry: false,
	staleTime: Infinity,
	refetchOnMount: false,
	refetchOnWindowFocus: false,
	refetchOnReconnect: false,
})
const hasPlus = computed(() => true)
const showAd = computed(() => false)
const adConsentAvailable = computed(() => false)
providePageContext({
	hierarchicalSidebarAvailable: ref(true),
	showAds: showAd,
	adConsentAvailable,
	floatingActionBarOffsets: {
		left: ref(APP_LEFT_NAV_WIDTH),
		right: computed(() => (sidebarVisible.value ? `${APP_SIDEBAR_WIDTH}px` : '0px')),
	},
	intercomBubble: hostingIntercom.intercomBubble,
	featureFlags: {
		serverRamAsBytesAlwaysOn: computed(() =>
			appSettings.getFeatureFlag('server_ram_as_bytes_always_on'),
		),
	},
	openExternalUrl: (url) => void openUrl(url),
})
provideModalBehavior({
	noblur: computed(() => !appTheme.advancedRendering),
	onShow: () => take_ads_window_hold(),
	onHide: () => release_ads_window_hold(),
})

const creationIconEditorModal = ref(null)
const creationGeneratedIcon = ref(null)
const creationIconTarget = ref('creation-flow')

const stateInitialized = ref(false)

const {
	installationModal,
	unknownPackWarningModal,
	fetchExistingInstanceNames,
	handleCreate,
	handleBrowseModpacks,
	searchProjects,
	getLoaderManifest,
	setModpackAlreadyInstalledModal,
	handleModpackDuplicateCreateAnyway,
	handleModpackDuplicateGoToInstance,
	onboardingChecklist,
} = setupProviders(
	tauriApiClient,
	notificationManager,
	popupNotificationManager,
	appEvents,
	stateInitialized,
	(iconPath) =>
		creationGeneratedIcon.value?.path === iconPath ? creationGeneratedIcon.value.config : null,
)
const { showChecklist } = onboardingChecklist

async function randomizeCreationIcon() {
	const generated = await creationIconEditorModal.value?.randomizeAndSave()
	if (!generated) return null

	creationGeneratedIcon.value = { path: generated.iconPath, config: generated.config }
	return {
		path: generated.iconPath,
		previewUrl: convertFileSrc(generated.iconPath),
	}
}

function customizeCreationIcon() {
	creationIconTarget.value = 'creation-flow'
	creationIconEditorModal.value?.show()
}

function customizeContentInstallIcon() {
	creationIconTarget.value = 'content-install'
	creationIconEditorModal.value?.show()
}

function onCreationIconSaved(iconPath, config) {
	creationGeneratedIcon.value = { path: iconPath, config }
	if (creationIconTarget.value === 'content-install') {
		modInstallModal.value?.setIcon(iconPath, convertFileSrc(iconPath))
		return
	}

	const context = installationModal.value?.ctx
	if (!context) return

	context.instanceIcon.value = null
	context.instanceIconUrl.value = convertFileSrc(iconPath)
	context.instanceIconPath.value = iconPath
}

const displayedServerInviteNotifications = new Set()
const serverInvitePopupNotificationIds = new Set()
let liveNotificationGeneration = 0
let liveNotificationsEnabled = true

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const nativeDecorations = ref(false)

const os = ref('')
const isDevEnvironment = ref(false)

const globalSyncedOptionsQuery = useQuery({
	queryKey: ['global-synced-options'],
	queryFn: get_global_synced_options,
	enabled: computed(() => stateInitialized.value),
})

const criticalErrorMessage = ref()

const isMaximized = ref(false)
const isFullscreen = ref(false)

watch([os, isFullscreen], ([osName, fullscreen]) => {
	document.documentElement.classList.toggle('mac-traffic-lights', osName === 'MacOS' && !fullscreen)
})

const authUnreachableDebug = useDebugLogger('AuthReachableChecker')
const authServerQuery = useQuery({
	queryKey: ['authServerReachability'],
	queryFn: async () => {
		await check_reachable()
		authUnreachableDebug('Auth servers are reachable')
		return true
	},
	refetchInterval: 5 * 60 * 1000, // 5 minutes
	retry: false,
	refetchOnWindowFocus: false,
})

const authUnreachable = computed(() => {
	if (authServerQuery.isError.value && !authServerQuery.isLoading.value) {
		console.warn('Failed to reach auth servers', authServerQuery.error.value)
		return true
	}
	return false
})

let unlistenEditMenu

function handleEditMenuAction(action) {
	const event = new CustomEvent(`edit-menu:${action}`, { cancelable: true })
	if (document.dispatchEvent(event)) document.execCommand(action)
}

onMounted(async () => {
	try {
		const listeners = await Promise.all([
			listen('edit-menu://undo', () => handleEditMenuAction('undo')),
			listen('edit-menu://redo', () => handleEditMenuAction('redo')),
		])
		unlistenEditMenu = () => listeners.forEach((unlisten) => unlisten())
	} catch (error) {
		handleError(error)
	}

	await useCheckDisableMouseover()

	document.querySelector('body').addEventListener('click', handleClick)
	document.querySelector('body').addEventListener('auxclick', handleAuxClick)
	document.querySelector('body').addEventListener('contextmenu', handleContextMenu)
	document.addEventListener('fullscreenchange', handleFullscreenChange)

	checkUpdates()
})

onUnmounted(async () => {
	document.querySelector('body').removeEventListener('click', handleClick)
	document.querySelector('body').removeEventListener('auxclick', handleAuxClick)
	document.querySelector('body').removeEventListener('contextmenu', handleContextMenu)
	document.removeEventListener('fullscreenchange', handleFullscreenChange)
	unlistenEditMenu?.()
	clearDelayedUpdatePopup()
	stopChatUnreadPoll()

	if (fullscreenAdsWindowHold) {
		fullscreenAdsWindowHold = false
		await release_ads_window_hold().catch(handleError)
	}
	await unlistenUpdateDownload?.()
})

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()

const RAIL_STORAGE_KEY = 'octra.railExpanded'
const railExpanded = ref(localStorage.getItem(RAIL_STORAGE_KEY) !== '0')
watch(railExpanded, (value) => {
	localStorage.setItem(RAIL_STORAGE_KEY, value ? '1' : '0')
})

const messages = defineMessages({
	warning: { id: 'app.notification.warning', defaultMessage: 'Warning' },
	goBack: { id: 'app.navigation.go-back', defaultMessage: 'Go back' },
	goForward: { id: 'app.navigation.go-forward', defaultMessage: 'Go forward' },
	nextImage: { id: 'app.navigation.next-image', defaultMessage: 'Next image' },
	updateDownloadMissingVersion: {
		id: 'app.update.download-error.missing-version',
		defaultMessage: 'Failed to download update: no version available',
	},
	updateInstalledToastTitle: {
		id: 'app.update.complete-toast.title',
		defaultMessage: 'Version {version} was successfully installed!',
	},
	updateInstalledToastText: {
		id: 'app.update.complete-toast.text',
		defaultMessage: 'Click here to view the changelog.',
	},
	authUnreachableHeader: {
		id: 'app.auth-servers.unreachable.header',
		defaultMessage: 'Cannot reach authentication servers',
	},
	authUnreachableBody: {
		id: 'app.auth-servers.unreachable.body',
		defaultMessage:
			'Minecraft authentication servers may be down right now. Check your internet connection and try again later.',
	},
	adsConsentTitle: {
		id: 'app.ads-consent.title',
		defaultMessage: 'Your privacy and how ads support Modrinth',
	},
	adsConsentBody: {
		id: 'app.ads-consent.body',
		defaultMessage:
			'Ads make Modrinth possible and fund creator payouts. Our partners may store or access cookies in the app to personalize ads and measure performance.',
	},
	adsConsentManage: {
		id: 'app.ads-consent.manage',
		defaultMessage: 'Manage preferences',
	},
	adsConsentReject: {
		id: 'app.ads-consent.reject',
		defaultMessage: 'Reject all',
	},
	adsConsentAccept: {
		id: 'app.ads-consent.accept',
		defaultMessage: 'Accept all',
	},
	home: {
		id: 'app.nav.home',
		defaultMessage: 'Home',
	},
	railStart: {
		id: 'app.nav.start',
		defaultMessage: 'Start',
	},
	locker: {
		id: 'app.nav.locker',
		defaultMessage: 'Locker',
	},
	packGallery: {
		id: 'app.nav.pack-gallery',
		defaultMessage: 'Pack gallery',
	},
	collapseRail: {
		id: 'app.nav.collapse-rail',
		defaultMessage: 'Collapse menu',
	},
	expandRail: {
		id: 'app.nav.expand-rail',
		defaultMessage: 'Expand menu',
	},
	collapseSidebar: {
		id: 'app.nav.collapse-sidebar',
		defaultMessage: 'Hide sidebar',
	},
	expandSidebar: {
		id: 'app.nav.expand-sidebar',
		defaultMessage: 'Show friends',
	},
	chat: {
		id: 'app.nav.chat',
		defaultMessage: 'Chat',
	},
	servers: {
		id: 'app.nav.servers',
		defaultMessage: 'Servers',
	},
	screenshots: {
		id: 'app.nav.screenshots',
		defaultMessage: 'Screenshots',
	},
	createNewInstance: {
		id: 'app.nav.create-new-instance',
		defaultMessage: 'Create new instance',
	},
	modrinthAccount: {
		id: 'app.nav.modrinth-account',
		defaultMessage: 'Octra account',
	},
	viewProfile: {
		id: 'app.nav.view-profile',
		defaultMessage: 'View profile',
	},
	addFriend: {
		id: 'friends.action.add-friend',
		defaultMessage: 'Add a friend',
	},
	signInToModrinthAccount: {
		id: 'app.nav.sign-in-to-modrinth-account',
		defaultMessage: 'Log in to Octra',
	},
	loadingProfile: {
		id: 'app.nav.loading-profile',
		defaultMessage: 'Loading profile...',
	},
	switchAccount: {
		id: 'app.nav.switch-account',
		defaultMessage: 'Switch account',
	},
	addAccount: {
		id: 'app.nav.add-account',
		defaultMessage: 'Add account',
	},
	removeAccount: {
		id: 'app.nav.remove-account',
		defaultMessage: 'Remove account',
	},
	restarting: {
		id: 'app.restarting',
		defaultMessage: 'Restarting...',
	},
	upgradeToModrinthPlus: {
		id: 'app.nav.upgrade-to-modrinth-plus',
		defaultMessage: 'Connect Octra account',
	},
	octraLogout: {
		id: 'octra-account.logout',
		defaultMessage: 'Log out',
	},
	news: {
		id: 'app.news.title',
		defaultMessage: 'News',
	},
	viewAllNews: {
		id: 'app.news.view-all',
		defaultMessage: 'View all news',
	},
	playingAs: {
		id: 'app.sidebar.playing-as',
		defaultMessage: 'Playing as',
	},
	minecraftAccount: {
		id: 'minecraft-account.label',
		defaultMessage: 'Minecraft account',
	},
	addMicrosoftAccount: {
		id: 'minecraft-account.add-microsoft',
		defaultMessage: 'Add Microsoft account',
	},
	addOfflineAccount: {
		id: 'minecraft-account.add-offline',
		defaultMessage: 'Add offline account',
	},
	nonPremium: {
		id: 'minecraft-account.non-premium',
		defaultMessage: 'Non-premium',
	},
	playOffline: {
		id: 'minecraft-account.play-offline',
		defaultMessage: 'Play offline',
	},
	identityMinecraftSection: {
		id: 'app.nav.identity-minecraft-section',
		defaultMessage: 'Minecraft',
	},
	identityOctraSection: {
		id: 'app.nav.identity-octra-section',
		defaultMessage: 'Octra',
	},
	identityOctraRow: {
		id: 'app.nav.identity-octra-row',
		defaultMessage: 'Octra · {name}',
	},
	identityOctraSignIn: {
		id: 'app.nav.identity-octra-sign-in',
		defaultMessage: 'Sign in to Octra',
	},
	identityTooltip: {
		id: 'app.nav.identity-tooltip',
		defaultMessage: 'MC · {minecraft} · Octra · {octra}',
	},
	octraLogin: {
		id: 'octra-account.login',
		defaultMessage: 'Log in',
	},
	octraRegister: {
		id: 'octra-account.register',
		defaultMessage: 'Connect',
	},
	chatNewMessage: {
		id: 'octra.chat.new-message-toast',
		defaultMessage: 'New chat message',
	},
	identityInGame: {
		id: 'app.nav.identity-in-game',
		defaultMessage: 'In game · {name}',
	},
	identityInGameUnknown: {
		id: 'app.nav.identity-in-game-unknown',
		defaultMessage: 'In game',
	},
})

function handleAdsConsentRequired(_required) {
	// Ads are disabled in Octra App.
}

function applyForcedLocale() {
	i18n.global.locale.value = DEFAULT_APP_LOCALE
	document.documentElement.lang = 'pl'
}

async function setupApp() {
	await onboardingChecklist.initialize()

	if (shouldShowNewIconEditorNotification(showChecklist.value)) {
		addPopupNotification({
			contentType: 'custom',
			component: NewIconEditorNotification,
			autoCloseMs: null,
		})
	}

	const {
		native_decorations,
		theme,
		locale,
		telemetry,
		hide_nametag_skins_page,
		advanced_rendering,
		sync_theme_across_devices,
		sync_behavior_across_devices,
		developer_mode,
		feature_flags,
		pending_update_toast_for_version,
	} = await getSettings()

	applyForcedLocale()
	if (locale !== DEFAULT_APP_LOCALE || telemetry) {
		const settings = await getSettings()
		settings.locale = DEFAULT_APP_LOCALE
		settings.telemetry = false
		await setSettings(settings)
	}

	Object.assign(appSettings.featureFlags, feature_flags)
	isMaximized.value = await getCurrentWindow().isMaximized()
	isFullscreen.value = await getCurrentWindow().isFullscreen()
	os.value = await getOS()
	const dev = await isDev()
	isDevEnvironment.value = dev
	const version = await getVersion()
	nativeDecorations.value = native_decorations
	if (os.value !== 'MacOS') await getCurrentWindow().setDecorations(native_decorations)

	appTheme.preferred = theme
	appTheme.advancedRendering = advanced_rendering
	appTheme.syncAcrossDevices = sync_theme_across_devices
	bootstrapAccent()
	appSettings.syncBehaviorAcrossDevices = sync_behavior_across_devices
	appSettings.hideNametagSkinsPage = hide_nametag_skins_page
	appSettings.devMode = developer_mode
	stateInitialized.value = true

	await getCurrentWindow().onResized(async () => {
		isMaximized.value = await getCurrentWindow().isMaximized()
		isFullscreen.value = await getCurrentWindow().isFullscreen()
	})

	const osType = await type()
	if (osType === 'macos') {
		document.getElementsByTagName('html')[0].classList.add('mac')
	} else {
		document.getElementsByTagName('html')[0].classList.add('windows')
	}

	fetch(`https://api.modrinth.com/appCriticalAnnouncement.json?version=${version}`)
		.then((response) => response.json())
		.then((res) => {
			if (res && res.header && res.body) {
				criticalErrorMessage.value = res
			}
		})
		.catch(() => {
			console.log(
				`No critical announcement found at https://api.modrinth.com/appCriticalAnnouncement.json?version=${version}`,
			)
		})

	get_opening_command().then(handleCommand)
	fetchCredentials()
	refreshOctraAccount()
	void refreshRunningInstancePresence()
	displayedAppVersion.value = version
	await nextTick()
	whatsNewModal.value?.show?.()

	try {
		const skins = (await get_available_skins()) ?? []
		const capes = (await get_available_capes()) ?? []
		generateSkinPreviews(skins, capes)
	} catch (error) {
		console.warn('Failed to generate skin previews in app setup.', error)
	}

	if (pending_update_toast_for_version !== null) {
		const settings = await getSettings()
		settings.pending_update_toast_for_version = null
		await setSettings(settings)
	}
}

const stateFailed = ref(false)
initialize_state(appEventChannel)
	.then(() => {
		setupApp().catch((err) => {
			stateFailed.value = true
			console.error(err)
			error.showError(err, null, false, 'state_init')
		})
	})
	.catch((err) => {
		stateFailed.value = true
		console.error('Failed to initialize app', err)
		error.showError(err, null, false, 'state_init')
	})

const handleClose = async () => {
	await saveWindowState(StateFlags.ALL)
	await getCurrentWindow().close()
}

const loading = setupLoadingStateProvider()
loading.setEnabled(false)
let initialLoadToken = loading.begin()
let routerToken = null
let suspenseToken = null

let suspensePending = false

const sidebarOverlayScrollbarsOptions = Object.freeze({
	overflow: {
		x: 'hidden',
		y: 'scroll',
	},
})

router.beforeEach(() => {
	suspensePending = false
	if (routerToken) loading.end(routerToken)
	routerToken = loading.begin()
})
router.afterEach((to, from, failure) => {
	updateHistoryNavigationState()
	trackEvent('PageView', {
		path: to.path,
		fromPath: from.path,
		failed: failure,
	})
	setTimeout(() => {
		if (!suspensePending && stateInitialized.value) {
			if (initialLoadToken) {
				loading.end(initialLoadToken)
				initialLoadToken = null
			}
			if (routerToken) {
				loading.end(routerToken)
				routerToken = null
			}
		}
	}, 100)
})

function onSuspensePending() {
	suspensePending = true
	if (suspenseToken) loading.end(suspenseToken)
	suspenseToken = loading.begin()
}

function onSuspenseResolve() {
	if (suspenseToken) {
		loading.end(suspenseToken)
		suspenseToken = null
	}
	if (routerToken) {
		loading.end(routerToken)
		routerToken = null
	}
}

const queryClient = useQueryClient()

watch(stateInitialized, (ready) => {
	if (ready) {
		if (initialLoadToken) {
			loading.end(initialLoadToken)
			initialLoadToken = null
		}
		if (routerToken) {
			loading.end(routerToken)
			routerToken = null
		}

		queryClient.prefetchQuery({
			queryKey: ['servers'],
			queryFn: async () => {
				const response = await tauriApiClient.archon.servers_v0.list({ limit: 100 })
				const hasMedalServers = response.servers.some((s) => s.is_medal)
				if (hasMedalServers) {
					const subscriptions = await tauriApiClient.labrinth.billing_internal.getSubscriptions()
					for (const server of response.servers) {
						if (server.is_medal) {
							const sub = subscriptions.find((s) => s.metadata?.id === server.server_id)
							if (sub) {
								server.medal_expires = new Date(
									new Date(sub.created).getTime() + 5 * 86400000,
								).toISOString()
							}
						}
					}
				}
				return response
			},
			staleTime: 30_000,
		})
		queryClient.prefetchQuery({
			queryKey: ['billing', 'subscriptions'],
			queryFn: () => tauriApiClient.labrinth.billing_internal.getSubscriptions(),
			staleTime: 30_000,
		})
		queryClient.prefetchQuery({
			queryKey: ['billing', 'payments'],
			queryFn: () => tauriApiClient.labrinth.billing_internal.getPayments(),
			staleTime: 30_000,
		})
	}
})

const error = useError()
const errorModal = ref()
const minecraftAuthErrorModal = ref()
const minecraftRequiredModal = ref()

const contentInstall = createContentInstall({ router, handleError, appEvents })
provideContentInstall(contentInstall)
const {
	instances: contentInstallInstances,
	compatibleLoaders: contentInstallLoaders,
	gameVersions: contentInstallGameVersions,
	loading: contentInstallLoading,
	defaultTab: contentInstallDefaultTab,
	preferredLoader: contentInstallPreferredLoader,
	preferredGameVersion: contentInstallPreferredGameVersion,
	releaseGameVersions: contentInstallReleaseGameVersions,
	projectInfo: contentInstallProjectInfo,
	handleInstallToInstance,
	handleCreateAndInstall,
	prepareNewInstance,
	handleNavigate: handleContentInstallNavigate,
	handleCancel: handleContentInstallCancel,
	setContentInstallModal,
	setModpackAlreadyInstalledModal: setContentInstallModpackAlreadyInstalledModal,
	handleModpackDuplicateCreateAnyway: handleContentInstallModpackDuplicateCreateAnyway,
	handleModpackDuplicateGoToInstance: handleContentInstallModpackDuplicateGoToInstance,
	setIncompatibilityWarningModal: setContentIncompatibilityWarningModal,
	incompatibilityWarningVersions: contentInstallIncompatibilityWarningVersions,
	incompatibilityWarningCurrentGameVersion: contentInstallIncompatibilityWarningCurrentGameVersion,
	incompatibilityWarningCurrentLoader: contentInstallIncompatibilityWarningCurrentLoader,
	incompatibilityWarningProjectType: contentInstallIncompatibilityWarningProjectType,
	incompatibilityWarningProjectIconUrl: contentInstallIncompatibilityWarningProjectIconUrl,
	incompatibilityWarningProjectName: contentInstallIncompatibilityWarningProjectName,
	incompatibilityWarningMessage: contentInstallIncompatibilityWarningMessage,
	incompatibilityWarningInstalling: contentInstallIncompatibilityWarningInstalling,
	handleIncompatibilityWarningInstall: handleContentInstallIncompatibilityWarningInstall,
	handleIncompatibilityWarningCancel: handleContentInstallIncompatibilityWarningCancel,
} = contentInstall

async function prepareCreationProjectInstall(projectId, projectType) {
	if (projectType === 'modpack') {
		await contentInstall.install(
			projectId,
			null,
			null,
			'CreationModalProject',
			undefined,
			(instanceId) => void router.push(`/instance/${encodeURIComponent(instanceId)}`),
		)
		return null
	}

	await prepareNewInstance(projectId)
	const info = contentInstallProjectInfo.value
	if (!info) throw new Error(`Project information is unavailable: '${projectId}'`)

	return {
		projectId,
		title: info.title,
		iconUrl: info.iconUrl,
		link: info.link,
		owner: info.owner,
		compatibleLoaders: [...contentInstallLoaders.value],
		gameVersions: [...contentInstallGameVersions.value],
		releaseGameVersions: new Set(contentInstallReleaseGameVersions.value),
	}
}

const serverInstall = createServerInstall({
	router,
	handleError,
	popupNotificationManager,
	appEvents,
})
provideServerInstall(serverInstall)
const {
	setInstallToPlayModal: setServerInstallToPlayModal,
	setUpdateToPlayModal: setServerUpdateToPlayModal,
	setAddServerToInstanceModal: setServerAddServerToInstanceModal,
	playServerProject,
} = serverInstall

const modInstallModal = ref()
const modpackAlreadyInstalledModal = ref()
const contentInstallModpackAlreadyInstalledModal = ref()
const addServerToInstanceModal = ref()
const incompatibilityWarningModal = ref()
const installToPlayModal = ref()
const sharedInstanceInviteHandler = ref()
const updateToPlayModal = ref()

const modrinthLoginModal = ref()
const appSettingsModal = ref()
provide(appSettingsModalOpenProfileKey, () => appSettingsModal.value?.showAccount())
provide(appSettingsModalOpenSyncedOptionsKey, () => appSettingsModal.value?.showSyncedOptions())
provide(appSettingsModalOpenLaunchDefaultsKey, () => appSettingsModal.value?.showLaunchDefaults())

watch(incompatibilityWarningModal, (modal) => {
	if (modal) {
		setContentIncompatibilityWarningModal(modal)
	}
})

const authProvider = setupAuthProvider(credentials, async (_redirectPath, flow, options) => {
	if (options?.showModal === false) {
		await signIn(flow)
	} else {
		await requestSignIn(flow)
	}
})

const userPreferences = setupAppUserPreferencesProvider(authProvider, notificationManager)
let userPreferencesSync = Promise.resolve()

watch(
	[userPreferences.preferences, stateInitialized],
	([preferences, initialized]) => {
		if (!preferences || !initialized) return

		userPreferencesSync = userPreferencesSync
			.then(async () => {
				const settings = await getSettings()
				const selectedTheme = preferences.appearance.auto ? 'system' : preferences.appearance.theme
				const userId = credentials.value?.user_id ?? credentials.value?.user?.id
				if (userId) {
					rememberAccountAppearance(userId, preferences.appearance)
				}
				if (appTheme.syncAcrossDevices && isDarkTheme(preferences.appearance.theme)) {
					appTheme.preferredDark = preferences.appearance.theme
				}
				const behavior = preferences.behavior
				let settingsChanged = false

				if (appTheme.syncAcrossDevices && appTheme.preferred !== selectedTheme) {
					appTheme.preferred = selectedTheme
				}
				applyForcedLocale()

				if (appTheme.syncAcrossDevices && settings.theme !== selectedTheme) {
					settings.theme = selectedTheme
					settingsChanged = true
				}
				if (settings.locale !== DEFAULT_APP_LOCALE) {
					settings.locale = DEFAULT_APP_LOCALE
					settingsChanged = true
				}

				if (behavior && appSettings.syncBehaviorAcrossDevices) {
					const behaviorFeatureFlags = {
						worlds_in_home: behavior.show_jump_in,
						compact_instance_cards: behavior.compact_instance_cards,
						show_instance_play_time: behavior.show_play_time,
						skip_unknown_pack_warning: !behavior.warn_on_unknown_modpacks,
						skip_non_essential_warnings: behavior.skip_non_essential_warnings,
					}

					appSettings.hideNametagSkinsPage = behavior.hide_nametag
					Object.assign(appSettings.featureFlags, behaviorFeatureFlags)

					if (settings.hide_on_process_start !== behavior.minimize_app) {
						settings.hide_on_process_start = behavior.minimize_app
						settingsChanged = true
					}
					if (settings.hide_nametag_skins_page !== behavior.hide_nametag) {
						settings.hide_nametag_skins_page = behavior.hide_nametag
						settingsChanged = true
					}
					for (const [flag, value] of Object.entries(behaviorFeatureFlags)) {
						if (settings.feature_flags[flag] !== value) {
							settings.feature_flags[flag] = value
							settingsChanged = true
						}
					}
				}

				if (settingsChanged) {
					await setSettings(settings)
				}
			})
			.catch(handleError)
	},
	{ immediate: true },
)

async function validateSession(sessionToken) {
	try {
		const response = await tauriFetch(`${config.labrinthBaseUrl}/v2/user`, {
			method: 'GET',
			headers: { Authorization: sessionToken },
		})
		if (response.status === 401) return false
		return true
	} catch {
		return true
	}
}

async function fetchCredentials() {
	const hadSession = !!credentials.value?.session
	const refreshId = ++credentialsRefreshId
	credentials.value = undefined

	const creds = await getCreds().catch(handleError)
	if (refreshId !== credentialsRefreshId) return
	if (!creds && hadSession) clearLiveNotifications()

	if (creds && creds.user_id) {
		if (creds.session && !(await validateSession(creds.session))) {
			if (refreshId !== credentialsRefreshId) return

			clearLiveNotifications()
			await removeUser(creds.user_id).catch(handleError)
			if (refreshId !== credentialsRefreshId) return

			credentials.value = null
			liveNotificationsEnabled = false
			return
		}
		creds.user = await get_user(creds.user_id, 'bypass').catch(handleError)
		if (refreshId !== credentialsRefreshId) return
	}
	credentials.value = creds ?? null
	liveNotificationsEnabled = !!creds?.session
}

async function signIn(_flow = 'sign-in', _addAccount = false) {
	openOctraAccount('login')
}

async function requestSignIn(_flow = 'sign-in', addAccount = false) {
	openOctraAccount(addAccount ? 'register' : 'login')
}

async function requestModrinthAuth(flow = 'sign-in', addAccount = false) {
	await signIn(flow, addAccount)
	return !!credentials.value?.session
}

const identityOctraLabel = computed(() => {
	if (octraSessionLoading.value) return formatMessage(messages.loadingProfile)
	if (octraSession.value) return octraSession.value.username
	return formatMessage(messages.identityOctraSignIn)
})

const identityOctraRowText = computed(() => {
	if (octraSessionLoading.value) return formatMessage(messages.loadingProfile)
	if (octraSession.value) {
		return formatMessage(messages.identityOctraRow, { name: octraSession.value.username })
	}
	return formatMessage(messages.identityOctraSignIn)
})

const identityAccountTooltip = computed(() => {
	const minecraft =
		selectedMinecraftAccount.value?.profile?.name ?? formatMessage(messages.minecraftAccount)
	return formatMessage(messages.identityTooltip, {
		minecraft,
		octra: identityOctraLabel.value,
	})
})

const isSwitchingAccount = ref(false)

async function fetchIntercomToken() {
	const creds = await getCreds()
	if (!creds?.session) {
		throw new Error('Not authenticated')
	}

	const params = new URLSearchParams()
	const rawServerId = route.params.id
	const serverId = Array.isArray(rawServerId) ? rawServerId[0] : rawServerId
	if (route.path.startsWith('/hosting/manage/') && typeof serverId === 'string') {
		params.set('server_id', serverId)
	}
	const query = params.size > 0 ? `?${params.toString()}` : ''

	const response = await tauriFetch(`${config.siteUrl}/api/intercom/messenger-jwt${query}`, {
		method: 'GET',
		headers: {
			Authorization: `Bearer ${creds.session}`,
		},
	})
	if (!response.ok) {
		throw new Error(`Failed to fetch Intercom token: ${response.status}`)
	}
	return await response.json()
}

watch(
	[showAd, adConsentAvailable],
	async () => {
		await hide_ads_window(true)
	},
	{ immediate: true },
)

onMounted(() => {
	invoke('show_window')

	error.setErrorModal(errorModal.value)
	error.setMinecraftAuthErrorModal(minecraftAuthErrorModal.value)
	error.setMinecraftRequiredModal(minecraftRequiredModal.value)

	setContentIncompatibilityWarningModal(incompatibilityWarningModal.value)
	setContentInstallModal(modInstallModal.value)
	setContentInstallModpackAlreadyInstalledModal(contentInstallModpackAlreadyInstalledModal.value)
	setModpackAlreadyInstalledModal(modpackAlreadyInstalledModal.value)
	setServerAddServerToInstanceModal(addServerToInstanceModal.value)
	setServerInstallToPlayModal(installToPlayModal.value)
	setServerUpdateToPlayModal(updateToPlayModal.value)
})

const accounts = ref(null)
const octraAccountModal = ref(null)
const octraSession = ref(null)
const octraSessionLoading = ref(true)

async function refreshOctraAccount() {
	octraSessionLoading.value = true
	try {
		octraSession.value = await octraAccountSession()
	} catch {
		octraSession.value = null
	} finally {
		octraSessionLoading.value = false
	}
}

function openOctraAccount(mode = 'login') {
	octraAccountModal.value?.show(mode)
}

async function onOctraAccountSuccess() {
	await refreshOctraAccount()
	await accounts.value?.refreshValues?.()
}

async function logoutOctraAccount() {
	await octraAccountLogout().catch(handleError)
	octraSession.value = null
	await accounts.value?.refreshValues?.()
}
provide('accountsCard', accounts)

watch(
	() => octraSession.value?.username ?? null,
	(username) => {
		chatUnreadReady.value = false
		if (username) {
			startChatUnreadPoll()
		} else {
			stopChatUnreadPoll()
			chatUnreadTotal.value = 0
			lastPolledUnreadTotal.value = 0
		}
	},
	{ immediate: true },
)

watch(chatPanelOpen, (open) => {
	if (open) {
		chatUnreadTotal.value = 0
	} else {
		void pollChatUnread()
	}
})

void refreshRunningInstancePresence()

const { refreshEquippedSkinAvatar, getAccountAvatarUrl } = useMinecraftAccountAvatar()
const minecraftAccounts = ref([])
const minecraftDefaultUser = ref()

async function refreshMinecraftAccounts() {
	try {
		minecraftDefaultUser.value = await get_default_user()
		const list = await listMinecraftUsers()
		minecraftAccounts.value = Array.isArray(list) ? [...list] : []
		minecraftAccounts.value.sort((a, b) =>
			(a.profile?.name ?? '').localeCompare(b.profile?.name ?? ''),
		)
		await refreshEquippedSkinAvatar(minecraftAccounts.value)
	} catch {
		minecraftAccounts.value = []
	}
}

const selectedMinecraftAccount = computed(() =>
	minecraftAccounts.value.find((account) => account.profile?.id === minecraftDefaultUser.value),
)

const minecraftAccountAvatar = computed(() =>
	getAccountAvatarUrl(
		selectedMinecraftAccount.value?.profile?.id,
		true,
		selectedMinecraftAccount.value ? isOfflineAccount(selectedMinecraftAccount.value) : false,
	),
)

const minecraftAccountSwitcherAccounts = computed(() =>
	minecraftAccounts.value.map((account) => ({
		...account,
		optionId: `mc-account-${account.profile.id}`,
		offline: isOfflineAccount(account),
		avatarUrl: getAccountAvatarUrl(
			account.profile.id,
			account.profile.id === minecraftDefaultUser.value,
			isOfflineAccount(account),
		),
	})),
)

const addOfflineAccountModal = ref(null)

async function setMinecraftAccount(account) {
	if (!account?.profile?.id || account.profile.id === minecraftDefaultUser.value) return
	await set_default_user(account.profile.id).catch(handleError)
	await refreshMinecraftAccounts()
	await accounts.value?.refreshValues?.()
}

async function addMicrosoftMinecraftAccount() {
	accounts.value?.setLoginDisabled?.(true)
	try {
		const loggedIn = await loginMinecraft().catch(handleError)
		if (loggedIn) {
			await set_default_user(loggedIn.profile.id).catch(handleError)
			await refreshMinecraftAccounts()
			await accounts.value?.refreshValues?.()
		}
	} finally {
		accounts.value?.setLoginDisabled?.(false)
	}
}

async function onOfflineMinecraftAccountAdded(loggedIn) {
	if (!loggedIn?.profile?.id) return
	await set_default_user(loggedIn.profile.id).catch(handleError)
	await refreshMinecraftAccounts()
	await accounts.value?.refreshValues?.()
}

async function removeMinecraftAccount(account) {
	if (!account?.profile?.id) return
	await remove_user(account.profile.id).catch(handleError)
	await refreshMinecraftAccounts()
	await accounts.value?.refreshValues?.()
}

async function signOutMinecraftAccount() {
	const selected = selectedMinecraftAccount.value
	if (!selected?.profile?.id) return
	await removeMinecraftAccount(selected)
}

const identityAccountMenuOptions = computed(() => {
	const options = [
		{
			type: 'heading',
			id: 'identity-minecraft-heading',
			label: formatMessage(messages.identityMinecraftSection),
		},
	]

	for (const account of minecraftAccountSwitcherAccounts.value) {
		options.push({
			id: account.optionId,
			label: account.profile.name,
			selected: account.profile.id === minecraftDefaultUser.value,
			remainOpen: true,
			action: () => setMinecraftAccount(account),
			trailingAction: {
				label: formatMessage(messages.removeAccount),
				icon: TrashIcon,
				color: 'red',
				action: (event) => {
					event.stopPropagation()
					removeMinecraftAccount(account)
				},
			},
		})
	}

	if (minecraftAccountSwitcherAccounts.value.length > 0) {
		options.push({ type: 'divider' })
	}

	options.push({
		id: 'add-microsoft',
		label: formatMessage(messages.addMicrosoftAccount),
		icon: PlusIcon,
		action: () => addMicrosoftMinecraftAccount(),
	})
	options.push({
		id: 'add-offline',
		label: formatMessage(messages.addOfflineAccount),
		icon: PlusIcon,
		action: () => addOfflineAccountModal.value?.show(),
	})

	if (selectedMinecraftAccount.value?.profile?.id) {
		options.push({ type: 'divider' })
		options.push({
			id: 'sign-out-minecraft',
			label: formatMessage(commonMessages.signOutButton),
			icon: LogOutIcon,
			tone: 'red',
			action: () => signOutMinecraftAccount(),
		})
	}

	options.push({ type: 'divider' })
	options.push({
		type: 'heading',
		id: 'identity-octra-heading',
		label: formatMessage(messages.identityOctraSection),
	})

	if (octraSession.value) {
		options.push({
			id: 'octra-logout',
			label: formatMessage(messages.octraLogout),
			icon: LogOutIcon,
			tone: 'red',
			action: () => logoutOctraAccount(),
		})
	} else {
		options.push({
			id: 'octra-login',
			label: formatMessage(messages.octraLogin),
			icon: LogInIcon,
			action: () => openOctraAccount('login'),
		})
		options.push({
			id: 'octra-register',
			label: formatMessage(messages.octraRegister),
			icon: UserPlusIcon,
			action: () => openOctraAccount('register'),
		})
	}

	return options
})

watch(
	stateInitialized,
	(value) => {
		if (value) {
			void refreshMinecraftAccounts()
		}
	},
	{ immediate: true },
)

useAppEvent(
	'process',
	async (e) => {
		if (e.event === 'launched') {
			await refreshMinecraftAccounts()
		}
		void refreshRunningInstancePresence()
	},
	appEvents,
)

useAppEvent('command', handleCommand, appEvents)
useAppEvent('notification', handleLiveNotification, appEvents)

async function markLiveNotificationRead(notification) {
	try {
		await tauriApiClient.labrinth.notifications_v2.markAsRead(notification.id)
	} catch (error) {
		if (error instanceof ModrinthApiError && error.statusCode === 404) {
			console.warn(`notification ${notification.id} could not be marked as read`, error)
			return
		}
		throw error
	}
}

async function respondToServerInvite(notification, action) {
	const serverId = notification.body?.server_id
	if (typeof serverId !== 'string') {
		throw new Error('Missing server ID for invite notification.')
	}

	await tauriApiClient.request(`/servers/${serverId}/invites/${action}`, {
		api: 'archon',
		version: 1,
		method: 'POST',
	})
	await markLiveNotificationRead(notification)

	return serverId
}

async function acceptServerInviteNotification(notification) {
	try {
		const serverId = await respondToServerInvite(notification, 'accept')
		await router.push(`/hosting/manage/${encodeURIComponent(serverId)}`)
		queryClient.invalidateQueries({ queryKey: ['servers'] })
	} catch (error) {
		handleError(error)
	}
}

async function declineServerInviteNotification(notification) {
	try {
		await respondToServerInvite(notification, 'decline')
	} catch (error) {
		handleError(error)
	}
}

function openServerInviteInviterProfile(inviterName) {
	if (!inviterName) return
	void router.push(`/user/${encodeURIComponent(inviterName)}`)
}

async function handleLiveNotification(notification) {
	if (!liveNotificationsEnabled || !notification?.body || notification.read) return
	if (await sharedInstanceInviteHandler.value?.handleNotification(notification)) return

	if (notification.body.type === 'server_invite') {
		if (displayedServerInviteNotifications.has(notification.id)) return

		const generation = liveNotificationGeneration
		displayedServerInviteNotifications.add(notification.id)

		const serverName =
			typeof notification.body.server_name === 'string' ? notification.body.server_name : 'a server'
		const inviterId = notification.body.invited_by
		const invitedBy =
			typeof inviterId === 'string' ? await get_user(inviterId, 'bypass').catch(() => null) : null
		if (generation !== liveNotificationGeneration) return

		const popupNotification = addPopupNotification({
			contentType: 'toast',
			title: serverName,
			type: 'server-invite',
			actorName: invitedBy?.username ?? null,
			actorAvatarUrl: invitedBy?.avatar_url ?? null,
			entityName: serverName,
			autoCloseMs: null,
			onAccept: () => acceptServerInviteNotification(notification),
			onDecline: () => declineServerInviteNotification(notification),
			onOpenActor: () => openServerInviteInviterProfile(invitedBy?.username ?? null),
		})
		serverInvitePopupNotificationIds.add(popupNotification.id)
	}
}

function clearLiveNotifications() {
	liveNotificationGeneration++
	liveNotificationsEnabled = false
	for (const id of serverInvitePopupNotificationIds) {
		popupNotificationManager.removeNotification(id)
	}
	displayedServerInviteNotifications.clear()
	serverInvitePopupNotificationIds.clear()
	sharedInstanceInviteHandler.value?.clearNotifications()
}

async function handleCommand(e) {
	if (!e) return

	if (e.event === 'RunMRPack') {
		// RunMRPack should directly install a local mrpack given a path
		if (e.path.endsWith('.mrpack')) {
			const location = { type: 'fromFile', path: e.path }
			const preview = await install_get_modpack_preview(location).catch(handleError)
			if (preview?.unknownFile || preview?.externalFilesInModpack.length > 0) {
				const splitPath = e.path.split(/[\\/]/)
				const fileName = splitPath ? splitPath[splitPath.length - 1] : e.path
				unknownPackWarningModal.value?.show(
					() => install_create_modpack_instance(location).then(() => undefined),
					fileName,
					preview.externalFilesInModpack,
				)
			} else {
				await install_create_modpack_instance(location).catch(handleError)
			}
			trackEvent('InstanceCreate', {
				source: 'CreationModalFileDrop',
			})
		}
	} else if (e.event === 'LaunchInstance') {
		const instance = await getInstance(e.id).catch(handleError)
		if (!instance || instance.quarantined) return

		if (e.server) {
			await start_join_server(e.id, e.server).catch(handleError)
		} else if (e.singleplayer_world) {
			await start_join_singleplayer_world(e.id, e.singleplayer_world).catch(handleError)
		} else {
			await run(e.id).catch(handleError)
		}
	} else if (e.event === 'InstallSharedInstanceInvite') {
		await sharedInstanceInviteHandler.value?.installFromInviteId(e.invite_id)
	} else if (e.event === 'InstallServer') {
		await router.push(`/project/${e.id}`)
		await playServerProject(e.id).catch(handleError)
	} else if (e.event === 'InstallVersion') {
		const version = await get_version(e.id, 'must_revalidate').catch(handleError)
		if (version) {
			await contentInstall
				.install(version.project_id, version.id, null, 'URLConfirmModal', undefined, undefined, {
					showProjectInfo: true,
				})
				.catch(handleError)
		}
	} else {
		await contentInstall
			.install(e.id, null, null, 'URLConfirmModal', undefined, undefined, { showProjectInfo: true })
			.catch(handleError)
	}
}

const appUpdateDownload = {
	progress: appUpdateState.progress,
	version: ref(),
}
let unlistenUpdateDownload

const {
	metered,
	finishedDownloading,
	downloading,
	restarting,
	availableUpdate,
	updateSize,
	updatesEnabled,
} = appUpdateState
let delayedUpdatePopupTimeout = null

const updatePopupMessages = defineMessages({
	updateAvailable: {
		id: 'app.update-popup.title',
		defaultMessage: 'Update available',
	},
	downloadComplete: {
		id: 'app.update-popup.download-complete',
		defaultMessage: 'Download complete',
	},
	meteredBody: {
		id: 'app.update-popup.body.metered',
		defaultMessage: `Octra App v{version} is available now! Since you're on a metered network, we didn't automatically download it.`,
	},
	downloadedBody: {
		id: 'app.update-popup.body.download-complete',
		defaultMessage: `Octra App v{version} has finished downloading. Reload to update now, or automatically when you close Octra App.`,
	},
	linuxBody: {
		id: 'app.update-popup.body.linux',
		defaultMessage:
			'Octra App v{version} is available. Use your package manager to update for the latest features and fixes!',
	},
	reload: {
		id: 'app.update-popup.reload',
		defaultMessage: 'Reload to update',
	},
	download: {
		id: 'app.update-popup.download',
		defaultMessage: 'Download ({size})',
	},
	changelog: {
		id: 'app.update-popup.changelog',
		defaultMessage: 'Changelog',
	},
})

function clearDelayedUpdatePopup() {
	if (delayedUpdatePopupTimeout !== null) {
		clearTimeout(delayedUpdatePopupTimeout)
		delayedUpdatePopupTimeout = null
	}
}

function getCurrentUpdatePromptStage() {
	return finishedDownloading.value ? 'downloaded' : 'available'
}

function scheduleDelayedUpdatePopup() {
	clearDelayedUpdatePopup()

	const version = availableUpdate.value?.version
	if (!version) {
		return
	}

	const nextPopupTime = getNextAppUpdatePopupTime(version, getCurrentUpdatePromptStage())
	if (nextPopupTime === null) {
		return
	}

	const delay = nextPopupTime - Date.now()
	if (delay <= 0) {
		showDelayedUpdatePopup()
		return
	}

	delayedUpdatePopupTimeout = setTimeout(showDelayedUpdatePopup, Math.min(delay, 2_147_483_647))
}

function showDelayedUpdatePopup() {
	const update = availableUpdate.value
	if (!update) {
		return
	}

	const stage = getCurrentUpdatePromptStage()
	const nextPopupTime = getNextAppUpdatePopupTime(update.version, stage)
	if (nextPopupTime === null) {
		return
	}

	if (Date.now() < nextPopupTime) {
		scheduleDelayedUpdatePopup()
		return
	}

	if (metered.value && !finishedDownloading.value) {
		addPopupNotification({
			contentType: 'standard',
			title: formatMessage(updatePopupMessages.updateAvailable),
			text: formatMessage(updatePopupMessages.meteredBody, { version: update.version }),
			type: 'info',
			autoCloseMs: null,
			buttons: [
				{
					label: formatMessage(updatePopupMessages.download, {
						size: formatBytes(updateSize.value ?? 0),
					}),
					action: () => downloadAvailableAppUpdate(),
					color: 'brand',
				},
				{
					label: formatMessage(updatePopupMessages.changelog),
					action: () => openAppUpdateChangelog(),
					keepOpen: true,
				},
			],
		})
	} else if (finishedDownloading.value) {
		addPopupNotification({
			contentType: 'standard',
			title: formatMessage(updatePopupMessages.downloadComplete),
			text: formatMessage(updatePopupMessages.downloadedBody, {
				version: update.version,
			}),
			type: 'success',
			autoCloseMs: null,
			buttons: [
				{
					label: formatMessage(updatePopupMessages.reload),
					action: () => installAvailableAppUpdate(),
					color: 'brand',
				},
				{
					label: formatMessage(updatePopupMessages.changelog),
					action: () => openAppUpdateChangelog(),
					keepOpen: true,
				},
			],
		})
	} else {
		scheduleDelayedUpdatePopup()
		return
	}

	markAppUpdatePopupShown(update.version, stage)
}

async function checkUpdates() {
	if (!(await areUpdatesEnabled())) {
		console.log('Skipping update check as updates are disabled in this build or environment')
		updatesEnabled.value = false

		if (os.value === 'Linux' && !isDevEnvironment.value) {
			checkLinuxUpdates()
			setInterval(checkLinuxUpdates, 5 * 60 * 1000)
		}
		return
	}

	await performUpdateCheck()
	setTimeout(
		() => {
			checkUpdates()
		},
		5 /* min */ * 60 /* sec */ * 1000 /* ms */,
	)
}

async function performUpdateCheck() {
	const update = await invoke('plugin:updater|check')
	if (!update) {
		console.log('No update available')
		return false
	}

	const isExistingUpdate = update.version === availableUpdate.value?.version

	if (isExistingUpdate) {
		console.log('Update is already known')
		scheduleDelayedUpdatePopup()
		return true
	}

	appUpdateDownload.progress.value = 0
	finishedDownloading.value = false
	downloading.value = false
	updateSize.value = null
	availableUpdate.value = update

	console.log(`Update ${update.version} is available.`)

	metered.value = await isNetworkMetered()
	if (!metered.value) {
		console.log('Starting download of update')
		downloadUpdate(update)
	} else {
		console.log(`Metered connection detected, not auto-downloading update.`)
		markAppUpdateActionable(update.version)
		scheduleDelayedUpdatePopup()
	}

	getUpdateSize(update.rid).then((size) => (updateSize.value = size))
	return true
}

async function manualCheckForUpdates() {
	if (!(await areUpdatesEnabled())) {
		return 'disabled'
	}

	try {
		const hasUpdate = await performUpdateCheck()
		return hasUpdate ? 'available' : 'latest'
	} catch (error) {
		console.error('Failed to check for updates manually:', error)
		return 'error'
	}
}

async function checkLinuxUpdates() {
	try {
		const [response, currentVersion] = await Promise.all([
			fetch('https://launcher-files.modrinth.com/updates.json'),
			getVersion(),
		])
		const updates = await response.json()
		const latestVersion = updates?.version

		if (latestVersion && latestVersion !== currentVersion) {
			markAppUpdateActionable(latestVersion)
			const nextPopupTime = getNextAppUpdatePopupTime(latestVersion)
			if (nextPopupTime !== null && Date.now() >= nextPopupTime) {
				addPopupNotification({
					contentType: 'standard',
					title: formatMessage(updatePopupMessages.updateAvailable),
					text: formatMessage(updatePopupMessages.linuxBody, { version: latestVersion }),
					type: 'info',
					autoCloseMs: null,
				})
				markAppUpdatePopupShown(latestVersion)
			}
		}
	} catch (e) {
		console.error('Failed to check for updates:', e)
	}
}

async function downloadAvailableUpdate() {
	return downloadUpdate(availableUpdate.value)
}

async function downloadUpdate(versionToDownload) {
	if (!versionToDownload) {
		handleError(formatMessage(messages.updateDownloadMissingVersion))
		return
	}

	if (downloading.value || appUpdateDownload.progress.value !== 0) {
		console.error(`Update ${versionToDownload.version} already downloading`)
		return
	}

	console.log(`Downloading update ${versionToDownload.version}`)
	downloading.value = true

	try {
		enqueueUpdateForInstallation(versionToDownload.rid)
			.then(() => {
				downloading.value = false
				finishedDownloading.value = true
				unlistenUpdateDownload?.()
				unlistenUpdateDownload = null
				console.log('Finished downloading!')
				markAppUpdateActionable(versionToDownload.version, 'downloaded')
				scheduleDelayedUpdatePopup()
			})
			.catch((e) => {
				downloading.value = false
				appUpdateDownload.progress.value = 0
				handleError(e)
			})
		unlistenUpdateDownload = await subscribeToDownloadProgress(
			appEvents,
			appUpdateDownload,
			versionToDownload.version,
		)
	} catch (e) {
		downloading.value = false
		appUpdateDownload.progress.value = 0
		handleError(e)
	}
}

async function installUpdate() {
	restarting.value = true

	try {
		await setRestartAfterPendingUpdate(true)
	} catch (e) {
		restarting.value = false
		handleError(e)
		return
	}
	setTimeout(async () => {
		await handleClose()
	}, 250)
}

setAppUpdateActions({
	download: downloadAvailableUpdate,
	install: installUpdate,
	changelog: () => openUrl('https://github.com/VasstOFC/octra-launcher/releases'),
	check: manualCheckForUpdates,
})

async function openModrinthProjectLinkInApp(parsed) {
	const { slug, pathSuffix, url } = parsed
	const loadToken = loading.begin()
	try {
		const { id } = await tauriApiClient.labrinth.projects_v2.check(slug)
		const query = mergeUrlQuery(route.query, url)
		await router.push({
			path: `/project/${id}${pathSuffix}`,
			query,
			hash: url.hash || undefined,
		})
	} catch (err) {
		if (err instanceof ModrinthApiError && err.statusCode === 404) {
			openUrl(url.href)
		} else {
			handleError(err)
		}
	} finally {
		loading.end(loadToken)
	}
}

function handleClick(e) {
	let target = e.target
	while (target != null) {
		if (target.matches('a')) {
			if (
				target.href &&
				['http://', 'https://', 'mailto:', 'tel:'].some((v) => target.href.startsWith(v)) &&
				!target.classList.contains('router-link-active') &&
				!target.href.startsWith('http://localhost') &&
				!target.href.startsWith('https://tauri.localhost') &&
				!target.href.startsWith('http://tauri.localhost')
			) {
				const userPath = parse_modrinth_user_link(target.href)
				const parsed = parseModrinthLink(target.href)
				if (userPath) {
					void router.push(userPath)
				} else if (target.target !== '_blank' && parsed) {
					void openModrinthProjectLinkInApp(parsed)
				} else {
					openUrl(target.href)
				}
			}
			e.preventDefault()
			break
		}
		target = target.parentElement
	}
}

function handleAuxClick(e) {
	// disables middle click -> new tab
	if (e.button === 1) {
		e.preventDefault()
		// instead do a left click
		const event = new MouseEvent('click', {
			view: window,
			bubbles: true,
			cancelable: true,
		})
		e.target.dispatchEvent(event)
	}
}

function handleContextMenu(event) {
	const target = event.target
	if (target instanceof Element) {
		if (target.closest('img, textarea, [contenteditable="true"]')) return
		const input = target.closest('input')
		if (
			input &&
			!['button', 'checkbox', 'radio', 'submit', 'reset', 'file', 'range'].includes(input.type)
		) {
			return
		}
	}

	const selection = window.getSelection()
	if (
		target instanceof Node &&
		selection &&
		!selection.isCollapsed &&
		selection.containsNode(target, true)
	) {
		return
	}

	event.preventDefault()
}

provideAppUpdateDownloadProgress(appUpdateDownload)
</script>

<template>
	<SplashScreen v-if="!stateFailed" ref="splashScreen" data-tauri-drag-region />
	<div id="teleports"></div>
	<AccountSwitchOverlay :show="isSwitchingAccount" />
	<div
		v-if="stateInitialized"
		class="app-grid-layout relative"
		:class="{
			'disable-advanced-rendering': !appTheme.advancedRendering,
			'rail-expanded': railExpanded,
		}"
	>
		<Transition name="fade">
			<div
				v-if="restarting"
				data-tauri-drag-region
				class="inset-0 fixed bg-black/80 backdrop-blur z-[200] flex items-center justify-center"
			>
				<span
					data-tauri-drag-region
					class="flex items-center gap-4 text-contrast font-semibold text-xl select-none cursor-default"
				>
					<RefreshCwIcon data-tauri-drag-region class="animate-spin w-6 h-6" />
					{{ formatMessage(messages.restarting) }}
				</span>
			</div>
		</Transition>
		<Suspense>
			<AppSettingsModal ref="appSettingsModal" />
		</Suspense>
		<Suspense>
			<ModrinthAccountRequiredModal ref="modrinthLoginModal" :request-auth="requestModrinthAuth" />
		</Suspense>
		<CreationFlowModal
			ref="installationModal"
			type="instance"
			show-snapshot-toggle
			:fetch-existing-instance-names="fetchExistingInstanceNames"
			:search-projects="searchProjects"
			:prepare-project-install="prepareCreationProjectInstall"
			:create-project-install="handleCreateAndInstall"
			:get-loader-manifest="getLoaderManifest"
			:randomize-instance-icon="randomizeCreationIcon"
			:customize-instance-icon="customizeCreationIcon"
			@create="handleCreate"
			@browse-modpacks="handleBrowseModpacks"
		/>
		<IconEditorModal
			ref="creationIconEditorModal"
			:config="creationGeneratedIcon?.config"
			@saved="onCreationIconSaved"
		/>
		<UnknownPackWarningModal ref="unknownPackWarningModal" />
		<div
			class="app-grid-navbar flex flex-col p-2 gap-1 w-[--left-bar-width] overflow-hidden"
			:class="railExpanded ? 'items-stretch' : 'items-center'"
		>
			<NavButton
				v-tooltip.right="railExpanded ? undefined : formatMessage(messages.railStart)"
				to="/"
				:is-primary="(route) => route.path === '/'"
				:is-subpage="
					() =>
						(route.path.startsWith('/browse') || route.path.startsWith('/project')) && route.query.i
				"
				:expanded="railExpanded"
				:label="formatMessage(messages.railStart)"
			>
				<PlayIcon class="ml-0.5 size-5 shrink-0" />
			</NavButton>
			<NavButton
				v-tooltip.right="railExpanded ? undefined : formatMessage(appMessages.skinSelectorLabel)"
				to="/skins"
				:expanded="railExpanded"
				:label="formatMessage(messages.locker)"
			>
				<ShirtIcon class="size-5 shrink-0" />
			</NavButton>
			<div class="my-1.5 h-px shrink-0 bg-surface-5" :class="railExpanded ? 'mx-1' : 'w-8'" />
			<NavButton
				v-if="globalSyncedOptionsQuery.data.value?.screenshots"
				v-tooltip.right="railExpanded ? undefined : formatMessage(messages.screenshots)"
				to="/screenshots"
				:expanded="railExpanded"
				:label="formatMessage(messages.screenshots)"
			>
				<ImagesIcon class="size-5 shrink-0" />
			</NavButton>
			<NavButton
				v-tooltip.right="railExpanded ? undefined : formatMessage(messages.servers)"
				to="/servers"
				:is-primary="(r) => r.path === '/servers' || r.path.startsWith('/servers/')"
				:expanded="railExpanded"
				:label="formatMessage(messages.servers)"
			>
				<GlobeIcon class="size-5 shrink-0" />
			</NavButton>
			<button
				type="button"
				class="nav-button nav-rail-slot flex items-center border-none cursor-pointer relative"
				:class="[
					railExpanded
						? 'h-10 w-full gap-3 rounded-lg px-3'
						: 'h-10 w-10 justify-center rounded-full',
					chatPanelOpen
						? 'nav-rail-slot--active text-brand'
						: 'text-secondary hover:bg-button-bg hover:text-contrast bg-transparent',
				]"
				:aria-label="formatMessage(messages.chat)"
				:title="formatMessage(messages.chat)"
				:aria-pressed="chatPanelOpen"
				@click="chatPanelOpen = !chatPanelOpen"
			>
				<MessageIcon class="size-5 shrink-0" />
				<span v-if="railExpanded" class="truncate text-[13px] font-medium">
					{{ formatMessage(messages.chat) }}
				</span>
				<span
					v-if="chatUnreadTotal > 0 && !chatPanelOpen"
					class="absolute top-1 right-1 flex min-w-[1rem] items-center justify-center rounded-full bg-brand px-1 text-[10px] font-semibold leading-4 text-[var(--color-accent-contrast)]"
					:class="railExpanded ? '!right-2' : ''"
				>
					{{ chatUnreadTotal > 99 ? '99+' : chatUnreadTotal }}
				</span>
			</button>
			<NavButton
				v-tooltip.right="railExpanded ? undefined : formatMessage(messages.createNewInstance)"
				:to="() => installationModal?.show()"
				:disabled="offline"
				:expanded="railExpanded"
				:label="formatMessage(messages.createNewInstance)"
			>
				<PlusIcon class="size-5 shrink-0" />
			</NavButton>
			<div class="flex min-h-0 flex-1 flex-col overflow-hidden">
				<Suspense>
					<QuickInstanceSwitcher :expanded="railExpanded" />
				</Suspense>
			</div>
			<div class="nav-rail-footer flex w-full flex-col gap-1">
				<span
					v-tooltip.right="railExpanded ? undefined : identityAccountTooltip"
					class="inline-flex"
					:class="{ 'w-full': railExpanded }"
				>
					<TeleportOverflowMenu
						type="quiet"
						size="xl"
						:icon-only="!railExpanded"
						:circular="!railExpanded"
						:label="identityAccountTooltip"
						:options="identityAccountMenuOptions"
						placement="right-end"
						:distance="4"
						class="nav-rail-account nav-rail-identity brightness-100 hover:!brightness-100 focus-visible:!brightness-100"
						:class="
							railExpanded
								? 'nav-rail-account--expanded nav-rail-identity--expanded !w-full justify-start gap-3 px-3'
								: ''
						"
					>
						<Avatar
							:src="minecraftAccountAvatar"
							alt=""
							size="24px"
							circle
							no-shadow
							class="pointer-events-none !size-6 shrink-0"
						/>
						<span
							v-if="railExpanded"
							class="nav-rail-identity-text flex min-w-0 flex-1 flex-col items-start gap-0.5"
						>
							<span class="min-w-0 truncate text-[13px] font-medium text-primary">
								{{
									selectedMinecraftAccount?.profile?.name ??
									formatMessage(messages.minecraftAccount)
								}}
							</span>
							<span
								class="flex min-w-0 items-center gap-1 text-[11px] leading-tight text-secondary"
							>
								<UserIcon class="size-3 shrink-0 opacity-80" />
								<span class="min-w-0 truncate">{{ identityOctraRowText }}</span>
							</span>
							<span
								v-if="runningInstanceName !== null"
								class="flex min-w-0 items-center gap-1 text-[10px] leading-tight text-brand-green"
							>
								<span class="size-1.5 shrink-0 rounded-full bg-brand-green" />
								<span class="min-w-0 truncate">
									{{
										runningInstanceName
											? formatMessage(messages.identityInGame, { name: runningInstanceName })
											: formatMessage(messages.identityInGameUnknown)
									}}
								</span>
							</span>
						</span>
						<template
							v-for="account in minecraftAccountSwitcherAccounts"
							:key="account.optionId"
							#[account.optionId]
						>
							<Avatar :src="account.avatarUrl" size="1.25rem" aria-hidden="true" circle />
							<span class="min-w-0 truncate">{{ account.profile.name }}</span>
							<span
								v-if="account.offline"
								class="shrink-0 rounded-full bg-surface-3 px-1.5 py-0.5 text-[0.65rem] font-semibold leading-none text-secondary"
							>
								{{ formatMessage(messages.nonPremium) }}
							</span>
						</template>
					</TeleportOverflowMenu>
				</span>
				<NavButton
					v-tooltip.right="
						railExpanded ? undefined : formatMessage(commonMessages.discoverContentLabel)
					"
					to="/browse/modpack"
					:is-primary="() => route.path.startsWith('/browse') && !route.query.i && !route.query.sid"
					:is-subpage="
						(route) => route.path.startsWith('/project') && !route.query.i && !route.query.sid
					"
					:expanded="railExpanded"
					:label="formatMessage(messages.packGallery)"
				>
					<CompassIcon class="size-5 shrink-0" />
				</NavButton>
				<NavButton
					v-tooltip.right="railExpanded ? undefined : formatMessage(commonMessages.settingsLabel)"
					:to="() => appSettingsModal?.show()"
					:expanded="railExpanded"
					:label="formatMessage(commonMessages.settingsLabel)"
				>
					<SettingsIcon class="size-5 shrink-0" />
				</NavButton>
			</div>
			<div class="my-1 h-px shrink-0 bg-surface-5" :class="railExpanded ? 'mx-1' : 'w-8'" />
			<button
				type="button"
				class="nav-rail-collapse flex items-center border-none bg-transparent text-secondary cursor-pointer hover:bg-button-bg hover:text-contrast"
				:class="
					railExpanded
						? 'h-10 w-full gap-3 rounded-lg px-3'
						: 'h-10 w-10 justify-center rounded-full'
				"
				:aria-label="formatMessage(railExpanded ? messages.collapseRail : messages.expandRail)"
				:title="formatMessage(railExpanded ? messages.collapseRail : messages.expandRail)"
				@click="railExpanded = !railExpanded"
			>
				<ChevronLeftIcon v-if="railExpanded" class="size-5 shrink-0" />
				<ChevronRightIcon v-else class="size-5 shrink-0" />
				<span v-if="railExpanded" class="truncate text-[13px] font-medium">
					{{ formatMessage(messages.collapseRail) }}
				</span>
			</button>
		</div>
		<div data-tauri-drag-region class="app-grid-statusbar h-[--top-bar-height] flex">
			<div data-tauri-drag-region class="flex min-w-0 flex-1 items-center overflow-hidden p-2">
				<OctraWordmark class="h-7 w-auto shrink-0 pointer-events-none" />
				<div data-tauri-drag-region class="ml-2 flex shrink-0 items-center gap-2">
					<IconButton
						type="outlined"
						:label="formatMessage(messages.goBack)"
						class="!h-7 !min-w-7 !w-7 !border !border-surface-4 !p-0 !opacity-100"
						:disabled="!canNavigateBack"
						@click="router.back()"
					>
						<ChevronLeftIcon
							class="!size-4 !text-primary"
							:class="{ 'opacity-20': !canNavigateBack }"
						/>
					</IconButton>
					<IconButton
						type="outlined"
						:label="formatMessage(messages.goForward)"
						class="!h-7 !min-w-7 !w-7 !border !border-surface-4 !p-0 !opacity-100"
						:disabled="!canNavigateForward"
						@click="router.forward()"
					>
						<ChevronRightIcon
							class="!size-4 !text-primary"
							:class="{ 'opacity-20': !canNavigateForward }"
						/>
					</IconButton>
				</div>
				<Breadcrumbs />
			</div>
			<section data-tauri-drag-region class="flex shrink-0 ml-auto items-center">
				<div class="flex mr-3">
					<Suspense>
						<AppActionBar />
					</Suspense>
				</div>
				<WindowControls />
			</section>
		</div>
	</div>
	<div
		v-if="stateInitialized"
		class="app-contents flex"
		:class="{
			'rail-expanded': railExpanded,
			'sidebar-enabled': sidebarVisible,
			'disable-advanced-rendering': !appTheme.advancedRendering,
		}"
	>
		<OctraChatPanel
			ref="octraChatPanel"
			:session="octraSession"
			:open="chatPanelOpen"
			@close="chatPanelOpen = false"
			@sign-in="openOctraAccount('login')"
			@unread-changed="onChatUnreadChanged"
		/>
		<div
			class="app-viewport flex-grow router-view min-w-0"
			:class="{ 'sidebar-open': sidebarVisible }"
			@pointerdown.capture="dismissChatPanelFromViewport"
		>
			<SurveyPopup />
			<div
				class="loading-indicator-container h-8 fixed z-50 pointer-events-none"
				:style="{
					top: 'calc(var(--top-bar-height))',
					left: 'calc(var(--left-bar-width))',
					width: 'calc(100% - var(--left-bar-width) - var(--right-bar-width))',
				}"
			>
				<LoadingBar position="absolute" />
			</div>
			<div
				v-if="appSettings.featureFlags.page_path"
				class="absolute bottom-0 left-0 m-2 bg-tooltip-bg text-tooltip-text font-semibold rounded-full px-2 py-1 text-xs z-50"
			>
				{{ route.fullPath }}
			</div>
			<div
				id="background-teleport-target"
				class="absolute h-full -z-10 rounded-tl-[--radius-lg] overflow-hidden"
				:style="{
					width: 'calc(100% - var(--right-bar-width))',
				}"
			></div>
			<Admonition
				v-if="criticalErrorMessage"
				type="critical"
				:header="criticalErrorMessage.header"
				class="m-6 mb-0"
			>
				<div
					class="markdown-body text-primary"
					v-html="renderString(criticalErrorMessage.body ?? '')"
				></div>
			</Admonition>
			<Admonition
				v-if="authUnreachable"
				type="warning"
				:header="formatMessage(messages.authUnreachableHeader)"
				class="m-6 mb-0"
			>
				{{ formatMessage(messages.authUnreachableBody) }}
			</Admonition>
			<HostingUpdateRequired v-if="hostingUpdateRequired" />
			<RouterView v-else v-slot="{ Component }">
				<template v-if="Component">
					<Suspense @pending="onSuspensePending" @resolve="onSuspenseResolve">
						<KeepAlive include="LibraryPage">
							<component :is="Component"></component>
						</KeepAlive>
					</Suspense>
				</template>
			</RouterView>
		</div>
		<aside
			class="app-sidebar mt-px flex flex-col border-0 border-l-[1px] border-solid border-surface-5"
			:class="{ 'has-plus': hasPlus, open: sidebarVisible }"
			:aria-hidden="sidebarVisible ? undefined : 'true'"
		>
			<div
				v-overlay-scrollbars="sidebarOverlayScrollbarsOptions"
				class="app-sidebar-scrollable flex-grow shrink relative"
				:class="{ 'pb-12': !hasPlus }"
				data-overlayscrollbars-initialize
			>
				<div class="hidden">
					<Suspense>
						<AccountsCard ref="accounts" @change="refreshMinecraftAccounts" />
					</Suspense>
				</div>
				<div id="sidebar-teleport-target" class="sidebar-teleport-content"></div>
				<div class="px-3 py-3">
					<OctraCommunityList
						:session="octraSession"
						:loading-session="octraSessionLoading"
						@sign-in="openOctraAccount('login')"
						@register="openOctraAccount('register')"
						@message-player="openOctraChatDm"
					/>
				</div>
			</div>
			<button
				v-if="canToggleSidebar"
				type="button"
				class="nav-rail-collapse m-2 flex items-center border-none bg-transparent text-secondary cursor-pointer hover:bg-button-bg hover:text-contrast h-11 w-[calc(100%-1rem)] gap-3 rounded-lg px-3"
				:aria-label="formatMessage(messages.collapseSidebar)"
				:title="formatMessage(messages.collapseSidebar)"
				@click="sidebarExpandedPreference = false"
			>
				<ChevronRightIcon class="size-5 shrink-0" />
				<span class="truncate text-[13px] font-medium">
					{{ formatMessage(messages.collapseSidebar) }}
				</span>
			</button>
			<template v-if="false">
				<a
					href="https://modrinth.plus?app"
					class="absolute bottom-[250px] w-full flex justify-center items-center gap-1 px-4 py-3 text-brand font-medium hover:underline z-10"
					target="_blank"
				>
					<ArrowBigUpDashIcon class="text-2xl" />
					{{ formatMessage(messages.upgradeToModrinthPlus) }}
				</a>
				<PromotionWrapper />
			</template>
		</aside>
	</div>
	<Transition name="friends-fab">
		<button
			v-if="stateInitialized && showFriendsFab"
			type="button"
			class="friends-fab fixed z-40 flex size-12 items-center justify-center rounded-full border-none bg-button-bg text-secondary shadow-raised cursor-pointer hover:bg-button-bg hover:text-contrast hover:brightness-[--hover-brightness]"
			:class="{ 'friends-fab--presence': !!octraSession }"
			:aria-label="formatMessage(messages.expandSidebar)"
			:title="formatMessage(messages.expandSidebar)"
			@click="sidebarExpandedPreference = true"
		>
			<UsersIcon class="size-5" />
		</button>
	</Transition>
	<I18nDebugPanel />
	<NotificationPanel :has-sidebar="sidebarVisible" />
	<PopupNotificationPanel :has-sidebar="sidebarVisible" />
	<ErrorModal ref="errorModal" />
	<MinecraftAuthErrorModal ref="minecraftAuthErrorModal" />
	<MinecraftRequiredModal
		ref="minecraftRequiredModal"
		@accounts-changed="refreshMinecraftAccounts"
	/>
	<AddOfflineAccountModal ref="addOfflineAccountModal" @added="onOfflineMinecraftAccountAdded" />
	<OctraAccountModal ref="octraAccountModal" @success="onOctraAccountSuccess" />
	<WhatsNewModal ref="whatsNewModal" :version="displayedAppVersion" />
	<ContentInstallModal
		ref="modInstallModal"
		:instances="contentInstallInstances"
		:compatible-loaders="contentInstallLoaders"
		:game-versions="contentInstallGameVersions"
		:loading="contentInstallLoading"
		:default-tab="contentInstallDefaultTab"
		:preferred-loader="contentInstallPreferredLoader"
		:preferred-game-version="contentInstallPreferredGameVersion"
		:release-game-versions="contentInstallReleaseGameVersions"
		:project-info="contentInstallProjectInfo"
		:randomize-icon="randomizeCreationIcon"
		:customize-icon="customizeContentInstallIcon"
		@install="handleInstallToInstance"
		@create-and-install="handleCreateAndInstall"
		@navigate="handleContentInstallNavigate"
		@cancel="handleContentInstallCancel"
	/>
	<ModpackAlreadyInstalledModal
		ref="modpackAlreadyInstalledModal"
		@create-anyway="handleModpackDuplicateCreateAnyway"
		@go-to-instance="handleModpackDuplicateGoToInstance"
	/>
	<AddServerToInstanceModal ref="addServerToInstanceModal" />
	<ContentUpdaterModal
		ref="incompatibilityWarningModal"
		mode="incompatibility-warning"
		:versions="contentInstallIncompatibilityWarningVersions"
		:current-game-version="contentInstallIncompatibilityWarningCurrentGameVersion"
		:current-loader="contentInstallIncompatibilityWarningCurrentLoader"
		current-version-id=""
		:is-app="true"
		:project-type="contentInstallIncompatibilityWarningProjectType"
		:project-icon-url="contentInstallIncompatibilityWarningProjectIconUrl"
		:project-name="contentInstallIncompatibilityWarningProjectName"
		:warning="contentInstallIncompatibilityWarningMessage"
		:action-loading="contentInstallIncompatibilityWarningInstalling"
		@update="handleContentInstallIncompatibilityWarningInstall"
		@cancel="handleContentInstallIncompatibilityWarningCancel"
	/>
	<ModpackAlreadyInstalledModal
		ref="contentInstallModpackAlreadyInstalledModal"
		@create-anyway="handleContentInstallModpackDuplicateCreateAnyway"
		@go-to-instance="handleContentInstallModpackDuplicateGoToInstance"
	/>
	<SharedInstanceInviteHandler ref="sharedInstanceInviteHandler" />
	<InstallToPlayModal ref="installToPlayModal" :show-external-warnings="false" />
	<UpdateToPlayModal ref="updateToPlayModal" :show-external-warnings="false" />
</template>

<style lang="scss" scoped>
.app-grid-layout,
.app-contents {
	--top-bar-height: 3rem;
	--left-bar-width: 4rem;
	--right-bar-width: 0px;
	--shell-motion: 0.28s cubic-bezier(0.32, 0.72, 0, 1);

	&.rail-expanded {
		--left-bar-width: 15.5rem;
	}
}

.app-contents.sidebar-enabled {
	--right-bar-width: 300px;
}

.app-grid-layout {
	display: grid;
	grid-template: 'status status' 'nav dummy';
	grid-template-columns: auto 1fr;
	grid-template-rows: auto 1fr;
	position: relative;
	background-color: var(--color-raised-bg);
	height: 100vh;
}

.app-grid-navbar {
	grid-area: nav;
	position: relative;
	z-index: 2;
	transition: width var(--shell-motion);
	border-right: 1px solid var(--color-divider);
	background: var(--surface-2);

	@media (prefers-reduced-motion: reduce) {
		transition: none;
	}
}

.nav-rail-slot {
	transition:
		background-color var(--shell-motion),
		color var(--shell-motion),
		box-shadow var(--shell-motion),
		transform var(--shell-motion);

	@media (prefers-reduced-motion: reduce) {
		transition: none;
	}
}

.nav-rail-slot--active {
	background: var(--surface-3);
	box-shadow: none;

	&::before {
		content: '';
		position: absolute;
		left: 0;
		top: 50%;
		transform: translateY(-50%);
		height: 1.25rem;
		width: 2px;
		border-radius: 1px;
		background: var(--color-brand);
	}
}

.nav-rail-footer {
	margin-top: 0.25rem;
	padding-top: 0.35rem;
	border-top: 1px solid color-mix(in srgb, var(--surface-5) 80%, transparent);
}

.nav-rail-account {
	height: 2.5rem !important;
	min-height: 2.5rem !important;
	background: transparent !important;
	box-shadow: none !important;
	color: var(--color-primary) !important;
	transition:
		background-color var(--shell-motion),
		color var(--shell-motion),
		box-shadow var(--shell-motion);

	&:hover,
	&:focus-visible {
		background: var(--color-button-bg) !important;
		color: var(--color-contrast) !important;
	}

	&:not(.nav-rail-account--expanded) {
		width: 2.5rem !important;
		min-width: 2.5rem !important;
		padding: 0 !important;
	}

	&.nav-rail-account--expanded {
		border-radius: var(--radius-md) !important;
	}

	@media (prefers-reduced-motion: reduce) {
		transition: none;
	}
}

.nav-rail-identity {
	&.nav-rail-identity--expanded {
		height: auto !important;
		min-height: 2.75rem !important;
		padding-top: 0.375rem !important;
		padding-bottom: 0.375rem !important;
		border-radius: var(--radius-md) !important;
	}
}

.app-grid-statusbar {
	grid-area: status;
	padding-right: var(--window-controls-width, 0px);
	position: relative;
	z-index: 2;
	background: var(--color-raised-bg);
	border-bottom: 1px solid var(--surface-5);
	box-shadow: none;
}

[data-tauri-drag-region-exclude] {
	-webkit-app-region: no-drag;
}

.app-contents {
	position: absolute;
	z-index: 1;
	left: var(--left-bar-width);
	top: var(--top-bar-height);
	right: 0;
	bottom: 0;
	height: calc(100vh - var(--top-bar-height));
	background-color: var(--surface-1);
	border-top-left-radius: var(--radius-lg);
	overflow: hidden;
}

.loading-indicator-container {
	border-top-left-radius: var(--radius-lg);
	overflow: hidden;
}

.app-viewport {
	flex-grow: 1;
	height: 100%;
	overflow: auto;
	overflow-x: hidden;
	scrollbar-gutter: stable;
	position: relative;
	background: var(--surface-1);
	transition: margin-right var(--shell-motion);

	&.sidebar-open {
		margin-right: 300px;
	}

	@media (prefers-reduced-motion: reduce) {
		transition: none;
	}
}

.app-sidebar {
	position: absolute;
	top: 0;
	right: 0;
	bottom: 0;
	z-index: 5;
	overflow: hidden;
	width: 300px;
	height: 100%;
	background: var(--color-raised-bg);
	transform: translateX(100%);
	pointer-events: none;
	transition: transform var(--shell-motion);

	&.open {
		transform: translateX(0);
		pointer-events: auto;
	}

	@media (prefers-reduced-motion: reduce) {
		transition: none;
	}
}

.app-sidebar::after {
	content: '';
	position: absolute;
	bottom: 250px;
	left: 0;
	right: 0;
	height: 5rem;
	background: var(--brand-gradient-fade-out-color);
	pointer-events: none;
}

.app-sidebar.has-plus::after {
	display: none;
}

.friends-fab {
	right: 1.25rem;
	bottom: 1.25rem;
}

.friends-fab--presence {
	color: var(--color-brand);
	box-shadow: inset 0 0 0 2px var(--color-brand);
}

.friends-fab-enter-active,
.friends-fab-leave-active {
	transition:
		opacity var(--shell-motion),
		transform var(--shell-motion);
}

.friends-fab-enter-from,
.friends-fab-leave-to {
	opacity: 0;
	transform: scale(0.82);
}

@media (prefers-reduced-motion: no-preference) {
	.friends-fab {
		transition:
			transform 0.15s ease,
			background-color var(--shell-motion),
			color var(--shell-motion),
			box-shadow var(--shell-motion);
	}

	.friends-fab:hover {
		transform: scale(1.03);
	}

	.friends-fab:active {
		transform: scale(0.97);
	}
}

@media (prefers-reduced-motion: reduce) {
	.friends-fab-enter-active,
	.friends-fab-leave-active {
		transition: none;
	}
}

.disable-advanced-rendering {
	.app-sidebar::before {
		box-shadow: none;
	}

	&.app-contents::before {
		box-shadow: none;
	}

	*,
	:deep(*) {
		box-shadow: none !important;
		--tw-drop-shadow:;
	}
}

.app-sidebar::before {
	content: none;
}

.app-contents::before {
	z-index: 30;
	content: '';
	position: fixed;
	left: var(--left-bar-width);
	top: var(--top-bar-height);
	right: calc(-1 * var(--left-bar-width));
	bottom: calc(-1 * var(--left-bar-width));
	border-radius: var(--radius-lg);
	box-shadow: none;
	border-color: var(--surface-5);
	border-width: 1px;
	border-style: solid;
	pointer-events: none;
}

.sidebar-teleport-content {
	display: contents;
}

.sidebar-default-content {
	display: none;
}

.sidebar-teleport-content:empty + .sidebar-default-content.sidebar-enabled {
	display: contents;
}

@media (prefers-reduced-motion: no-preference) {
	.nav-button-animated-enter-active {
		transition: all 0.5s cubic-bezier(0.15, 1.4, 0.64, 0.96);
	}

	.nav-button-animated-leave-active {
		transition: all 0.25s ease;
	}

	.nav-button-animated-enter-active {
		position: relative;
	}

	.nav-button-animated-enter-active::before {
		content: '';
		inset: 0;
		border-radius: 100vw;
		background-color: var(--color-brand-highlight);
		position: absolute;
		animation: pop 0.5s ease-in forwards;
		opacity: 0;
	}

	@keyframes pop {
		0% {
			scale: 0.5;
		}
		50% {
			opacity: 0.5;
		}
		100% {
			scale: 1.5;
		}
	}

	.nav-button-animated-enter-from {
		scale: 0.5;
		translate: -2rem 0;
		opacity: 0;
	}

	.nav-button-animated-leave-to {
		scale: 0.75;
		opacity: 0;
	}

	.fade-enter-active {
		transition: 0.25s ease-in-out;
	}

	.fade-enter-from {
		opacity: 0;
	}
}
</style>
<style>
.os-theme-dark,
.os-theme-light {
	--os-handle-bg: var(--color-scrollbar) !important;
	--os-handle-bg-hover: var(--color-scrollbar) !important;
	--os-handle-bg-active: var(--color-scrollbar) !important;
}

.mac-traffic-lights {
	.app-grid-statusbar {
		padding-left: 5rem;
	}
}

.windows {
	.fake-appbar {
		height: 2.5rem !important;
	}

	.info-card {
		right: 22rem;
	}

	.profile-card {
		right: 8rem;
	}
}
</style>
