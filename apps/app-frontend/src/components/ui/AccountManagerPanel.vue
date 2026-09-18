<script setup lang="ts">
import {
	LogInIcon,
	PlusIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	TrashIcon,
	UserPlusIcon,
} from '@lumen/assets'
import {
	Avatar,
	Button,
	defineMessages,
	EmptyState,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@lumen/ui'
import type { Ref } from 'vue'
import { computed, onMounted, ref } from 'vue'

import AddOfflineAccountModal from '@/components/ui/AddOfflineAccountModal.vue'
import LumenAccountModal from '@/components/ui/LumenAccountModal.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { useMinecraftAccountAvatar } from '@/composables/use-minecraft-account-avatar.ts'
import { trackEvent } from '@/helpers/analytics'
import {
	get_default_user,
	isOfflineAccount,
	login as loginMinecraft,
	remove_user,
	set_default_user,
	users as listMinecraftUsers,
} from '@/helpers/auth.js'
import { LumenAccountLogout, LumenAccountSession } from '@/helpers/lumen-account.js'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const emit = defineEmits<{
	change: []
}>()

type TabId = 'minecraft' | 'lumen'

type MinecraftCredential = {
	profile: {
		id: string
		name: string
	}
	is_offline?: boolean
}

const activeTab = ref<TabId>('minecraft')
const accounts: Ref<MinecraftCredential[]> = ref([])
const defaultUser = ref<string | undefined>()
const loaded = ref(false)
const loginDisabled = ref(false)
const lumenLoading = ref(false)
const lumenSession = ref<{
	username: string
	minecraft_nick: string
} | null>(null)
const adding = ref(false)
const listElement = ref<HTMLElement | null>(null)
const addOfflineModal = ref<InstanceType<typeof AddOfflineAccountModal>>()
const lumenAccountModal = ref<InstanceType<typeof LumenAccountModal>>()

const { refreshEquippedSkinAvatar, getAccountAvatarUrl } = useMinecraftAccountAvatar()

const messages = defineMessages({
	title: {
		id: 'account-manager.title',
		defaultMessage: 'Accounts',
	},
	tabMinecraft: {
		id: 'account-manager.tab.minecraft',
		defaultMessage: 'Minecraft',
	},
	tabLumen: {
		id: 'account-manager.tab.lumen',
		defaultMessage: 'Lumen',
	},
	emptyHint: {
		id: 'account-manager.empty-hint',
		defaultMessage: 'Add a Microsoft or offline account to start playing.',
	},
	addAccount: {
		id: 'minecraft-account.add-account',
		defaultMessage: 'Add account',
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
	removeAccount: {
		id: 'minecraft-account.remove-account',
		defaultMessage: 'Remove account',
	},
	notSignedIn: {
		id: 'minecraft-account.not-signed-in',
		defaultMessage: 'Not signed in',
	},
	lumenLogin: {
		id: 'lumen-account.login',
		defaultMessage: 'Log in',
	},
	lumenRegister: {
		id: 'lumen-account.register',
		defaultMessage: 'Connect',
	},
	lumenLogout: {
		id: 'lumen-account.logout',
		defaultMessage: 'Log out',
	},
	lumenHint: {
		id: 'lumen-account.hint',
		defaultMessage:
			'Lumen links skins to your Minecraft account — it does not create one. Add Microsoft or offline first.',
	},
})

const selectedAccount = computed(() =>
	accounts.value.find((account) => account.profile.id === defaultUser.value),
)
const previewAccounts = computed(() => accounts.value.slice(0, 4))
const hiddenAccountCount = computed(() => Math.max(0, accounts.value.length - 4))

async function refreshValues() {
	try {
		defaultUser.value = await get_default_user()
		const userList = await listMinecraftUsers()
		accounts.value = Array.isArray(userList) ? [...userList] : []
		accounts.value.sort((a, b) => (a.profile?.name ?? '').localeCompare(b.profile?.name ?? ''))
		lumenSession.value = await LumenAccountSession().catch(() => null)
		await refreshEquippedSkinAvatar(accounts.value)
	} catch (error) {
		handleError(error)
	} finally {
		loaded.value = true
	}
}

onMounted(() => {
	void refreshValues()
})

useAppEvent('process', async (event) => {
	if (event.event === 'launched') {
		await refreshValues()
	}
})

function avatarFor(account: MinecraftCredential, selected: boolean) {
	return getAccountAvatarUrl(account.profile.id, selected, isOfflineAccount(account))
}

async function setAccount(account: MinecraftCredential) {
	if (!account?.profile?.id || account.profile.id === defaultUser.value) return
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch(handleError)
	await refreshValues()
	emit('change')
}

async function login() {
	loginDisabled.value = true
	const loggedIn = await loginMinecraft().catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
	}

	trackEvent('AccountLogIn')
	loginDisabled.value = false
}

function addOffline() {
	adding.value = false
	addOfflineModal.value?.show()
}

async function onOfflineAdded(loggedIn: MinecraftCredential) {
	loginDisabled.value = true
	try {
		await setAccount(loggedIn)
		trackEvent('AccountLogIn', { source: 'offline' })
	} finally {
		loginDisabled.value = false
	}
}

async function removeAccount(account: MinecraftCredential, event?: MouseEvent) {
	event?.stopPropagation()
	if (!account?.profile?.id) return
	await remove_user(account.profile.id).catch(handleError)
	await refreshValues()
	if (!selectedAccount.value && accounts.value.length > 0) {
		await setAccount(accounts.value[0])
		return
	}
	emit('change')
}

function openLumenAccount(mode: 'login' | 'register') {
	lumenAccountModal.value?.show(mode)
}

async function onLumenAccountSuccess() {
	lumenLoading.value = true
	try {
		await refreshValues()
		emit('change')
	} finally {
		lumenLoading.value = false
	}
}

async function logoutLumen() {
	lumenLoading.value = true
	try {
		await LumenAccountLogout()
		lumenSession.value = null
		emit('change')
	} catch (error) {
		handleError(error)
	} finally {
		lumenLoading.value = false
	}
}

function focusRow(index: number) {
	const rows =
		listElement.value?.querySelectorAll<HTMLButtonElement>('[data-am-account-row]') ?? []
	rows[index]?.focus()
}

function onListKeydown(event: KeyboardEvent) {
	const rows =
		listElement.value?.querySelectorAll<HTMLButtonElement>('[data-am-account-row]') ?? []
	if (rows.length === 0) return

	const currentIndex = Array.from(rows).indexOf(document.activeElement as HTMLButtonElement)
	if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
		event.preventDefault()
		const next =
			event.key === 'ArrowDown'
				? Math.min(currentIndex + 1, rows.length - 1)
				: Math.max(currentIndex - 1, 0)
		focusRow(currentIndex === -1 && event.key === 'ArrowUp' ? rows.length - 1 : next)
	} else if (event.key === 'Home') {
		event.preventDefault()
		focusRow(0)
	} else if (event.key === 'End') {
		event.preventDefault()
		focusRow(rows.length - 1)
	}
}

function selectTab(tab: TabId) {
	activeTab.value = tab
	adding.value = false
}
</script>

<template>
	<section class="account-manager" :aria-label="formatMessage(messages.title)">
		<header class="account-manager__header">
			<div class="account-manager__wall" aria-hidden="true">
				<Avatar
					v-for="account in previewAccounts"
					:key="account.profile.id"
					:src="avatarFor(account, account.profile.id === defaultUser)"
					size="1.5rem"
					circle
					no-shadow
					class="account-manager__wall-avatar"
				/>
				<span v-if="hiddenAccountCount > 0" class="account-manager__wall-more">
					+{{ hiddenAccountCount }}
				</span>
			</div>
			<div class="account-manager__titles">
				<h2 class="account-manager__title">
					{{ formatMessage(messages.title) }}
				</h2>
				<p class="account-manager__subtitle">
					{{ accounts.length }} · Minecraft<span v-if="lumenSession"> · Lumen</span>
				</p>
			</div>
		</header>

		<div class="account-manager__tabs" role="tablist">
			<button
				type="button"
				role="tab"
				:aria-selected="activeTab === 'minecraft'"
				class="account-manager__tab"
				:class="{ 'account-manager__tab--active': activeTab === 'minecraft' }"
				@click="selectTab('minecraft')"
			>
				{{ formatMessage(messages.tabMinecraft) }}
			</button>
			<button
				type="button"
				role="tab"
				:aria-selected="activeTab === 'lumen'"
				class="account-manager__tab"
				:class="{ 'account-manager__tab--active': activeTab === 'lumen' }"
				@click="selectTab('lumen')"
			>
				<span class="account-manager__tab-dot" :class="{ 'account-manager__tab-dot--on': !!lumenSession }" aria-hidden="true" />
				{{ formatMessage(messages.tabLumen) }}
			</button>
		</div>

		<div v-if="activeTab === 'minecraft'" role="tabpanel">
			<div v-if="!loaded" class="account-manager__skeleton" aria-hidden="true">
				<div v-for="index in 3" :key="index" class="account-manager__skeleton-row">
					<span class="account-manager__skeleton-avatar" />
					<span class="account-manager__skeleton-lines">
						<span class="account-manager__skeleton-line account-manager__skeleton-line--long" />
						<span class="account-manager__skeleton-line account-manager__skeleton-line--short" />
					</span>
				</div>
			</div>

			<EmptyState
				v-else-if="accounts.length === 0"
				type="empty"
				:heading="formatMessage(messages.notSignedIn)"
				:description="formatMessage(messages.emptyHint)"
			>
				<template #actions>
					<Button type="colored" color="brand" :disabled="loginDisabled" @click="login()">
						<LogInIcon />
						{{ formatMessage(messages.addMicrosoftAccount) }}
					</Button>
				</template>
			</EmptyState>

			<template v-else>
				<div
					ref="listElement"
					role="radiogroup"
					class="account-manager__list"
					@keydown="onListKeydown"
				>
					<div
						v-for="account in accounts"
						:key="account.profile.id"
						class="account-manager__row-wrap"
					>
						<button
							type="button"
							role="radio"
							:aria-checked="account.profile.id === defaultUser"
							data-am-account-row
							class="account-manager__row"
							:class="{
								'account-manager__row--active': account.profile.id === defaultUser,
							}"
							@click="setAccount(account)"
						>
							<RadioButtonCheckedIcon
								v-if="account.profile.id === defaultUser"
								class="account-manager__radio account-manager__radio--checked"
								aria-hidden="true"
							/>
							<RadioButtonIcon v-else class="account-manager__radio" aria-hidden="true" />
							<Avatar :src="avatarFor(account, account.profile.id === defaultUser)" size="2rem" circle no-shadow />
							<span class="account-manager__row-text">
								<span class="account-manager__row-name">{{ account.profile.name }}</span>
								<span
									v-if="isOfflineAccount(account)"
									class="account-manager__badge"
								>
									{{ formatMessage(messages.nonPremium) }}
								</span>
							</span>
							<IconButton
								v-tooltip="formatMessage(messages.removeAccount)"
								:label="formatMessage(messages.removeAccount)"
								type="quiet"
								color="red"
								class="account-manager__trash"
								@click="(event: MouseEvent) => removeAccount(account, event)"
							>
								<TrashIcon />
							</IconButton>
						</button>
					</div>
				</div>

				<div class="account-manager__add">
					<Button
						v-if="!adding"
						class="account-manager__add-button"
						:disabled="loginDisabled"
						@click="adding = true"
					>
						<PlusIcon />
						{{ formatMessage(messages.addAccount) }}
					</Button>
					<div v-else class="account-manager__add-choice">
						<Button
							type="colored"
							color="brand"
							class="account-manager__add-button"
							:disabled="loginDisabled"
							@click="login()"
						>
							<LogInIcon v-if="!loginDisabled" />
							{{ formatMessage(messages.addMicrosoftAccount) }}
						</Button>
						<Button
							class="account-manager__add-button"
							:disabled="loginDisabled"
							@click="addOffline()"
						>
							<UserPlusIcon />
							{{ formatMessage(messages.addOfflineAccount) }}
						</Button>
					</div>
				</div>
			</template>
		</div>

		<div v-else role="tabpanel">
			<div v-if="!loaded" class="account-manager__skeleton" aria-hidden="true">
				<div class="account-manager__skeleton-row">
					<span class="account-manager__skeleton-avatar" />
					<span class="account-manager__skeleton-lines">
						<span class="account-manager__skeleton-line account-manager__skeleton-line--long" />
						<span class="account-manager__skeleton-line account-manager__skeleton-line--short" />
					</span>
				</div>
			</div>

			<div v-else-if="lumenSession" class="account-manager__lumen-card">
				<span class="account-manager__lumen-dot" aria-hidden="true" />
				<span class="account-manager__lumen-text">
					<span class="account-manager__lumen-name">{{ lumenSession.username }}</span>
					<span class="account-manager__lumen-nick">{{ lumenSession.minecraft_nick }}</span>
				</span>
				<Button type="outlined" size="sm" :disabled="lumenLoading" @click="logoutLumen">
					{{ formatMessage(messages.lumenLogout) }}
				</Button>
			</div>

			<div v-else class="account-manager__lumen-empty">
				<p class="account-manager__lumen-hint">
					{{ formatMessage(messages.lumenHint) }}
				</p>
				<Button
					type="colored"
					color="brand"
					class="account-manager__add-button"
					:disabled="lumenLoading"
					@click="openLumenAccount('login')"
				>
					<LogInIcon />
					{{ formatMessage(messages.lumenLogin) }}
				</Button>
				<Button
					class="account-manager__add-button"
					:disabled="lumenLoading"
					@click="openLumenAccount('register')"
				>
					<UserPlusIcon />
					{{ formatMessage(messages.lumenRegister) }}
				</Button>
			</div>
		</div>

		<AddOfflineAccountModal ref="addOfflineModal" @added="onOfflineAdded" />
		<LumenAccountModal ref="lumenAccountModal" @success="onLumenAccountSuccess" />
	</section>
</template>

<style scoped lang="scss">
.account-manager {
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
	width: 19rem;
	max-width: 80vw;
}

.account-manager__header {
	display: flex;
	align-items: center;
	gap: 0.75rem;
}

.account-manager__wall {
	display: flex;
	align-items: center;
	flex-shrink: 0;
}

.account-manager__wall-avatar {
	margin-left: -0.5rem;
	border: 2px solid var(--surface-3);
	border-radius: 50%;

	&:first-child {
		margin-left: 0;
	}
}

.account-manager__wall-more {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	min-width: 1.5rem;
	height: 1.5rem;
	margin-left: -0.5rem;
	padding: 0 0.25rem;
	border-radius: 9999px;
	border: 2px solid var(--surface-3);
	background: var(--surface-4);
	color: var(--color-secondary);
	font-size: 0.65rem;
	font-weight: 700;
}

.account-manager__titles {
	display: flex;
	flex-direction: column;
	gap: 0.125rem;
	min-width: 0;
}

.account-manager__title {
	margin: 0;
	font-size: 1rem;
	font-weight: 600;
	color: var(--color-contrast);
}

.account-manager__subtitle {
	margin: 0;
	font-size: 0.75rem;
	color: var(--color-secondary);
}

.account-manager__tabs {
	display: grid;
	grid-template-columns: 1fr 1fr;
	border-bottom: 1px solid var(--shell-border);
}

.account-manager__tab {
	display: flex;
	align-items: center;
	justify-content: center;
	gap: 0.375rem;
	padding: 0.625rem 0.25rem;
	background: transparent;
	border: 0;
	border-bottom: 2px solid transparent;
	color: var(--color-secondary);
	font-size: 0.8rem;
	font-weight: 600;
	cursor: pointer;
	transition:
		color 0.15s ease,
		border-color 0.15s ease;
}

.account-manager__tab:hover {
	color: var(--color-primary);
}

.account-manager__tab--active {
	border-bottom-color: var(--emerus-primary);
	color: var(--emerus-accent-bright);
}

.account-manager__tab-dot {
	width: 6px;
	height: 6px;
	border-radius: 50%;
	background: var(--color-secondary);
	opacity: 0.4;
	flex-shrink: 0;
}

.account-manager__tab-dot--on {
	background: var(--emerus-accent-bright);
	opacity: 1;
}

.account-manager__list {
	display: flex;
	flex-direction: column;
	gap: 0.125rem;
	max-height: 16rem;
	overflow-y: auto;
}

.account-manager__row-wrap {
	display: flex;
}

.account-manager__row {
	display: flex;
	align-items: center;
	gap: 0.625rem;
	width: 100%;
	min-height: 3rem;
	padding: 0.375rem 0.5rem;
	background: transparent;
	border: 1px solid transparent;
	border-radius: 10px;
	cursor: pointer;
	text-align: left;
	transition:
		background-color 0.15s ease,
		border-color 0.15s ease;
}

.account-manager__row:hover {
	background: color-mix(in srgb, var(--color-brand) 8%, transparent);
}

.account-manager__row:focus-visible {
	outline: 2px solid var(--emerus-primary);
	outline-offset: 1px;
}

.account-manager__row--active {
	background: color-mix(in srgb, var(--color-brand) 10%, transparent);
	border-color: color-mix(in srgb, var(--color-brand) 30%, transparent);
}

.account-manager__radio {
	width: 1.125rem;
	height: 1.125rem;
	flex-shrink: 0;
}

.account-manager__radio--checked {
	color: var(--color-brand);
}

.account-manager__row:not(.account-manager__row--active) .account-manager__radio {
	color: var(--color-secondary);
}

.account-manager__row-text {
	display: flex;
	flex-direction: column;
	align-items: flex-start;
	gap: 0.125rem;
	flex: 1;
	min-width: 0;
}

.account-manager__row-name {
	font-size: 0.875rem;
	font-weight: 600;
	color: var(--color-contrast);
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	max-width: 100%;
}

.account-manager__badge {
	font-size: 0.65rem;
	font-weight: 600;
	line-height: 1;
	padding: 0.25rem 0.5rem;
	border-radius: 9999px;
	color: var(--color-secondary);
	background: var(--surface-4);
	border: 1px solid var(--shell-border);
	white-space: nowrap;
}

.account-manager__trash {
	flex-shrink: 0;
	opacity: 0;
}

.account-manager__row:hover .account-manager__trash,
.account-manager__row:focus-within .account-manager__trash {
	opacity: 1;
}

@media (hover: none) {
	.account-manager__trash {
		opacity: 1;
	}
}

.account-manager__add {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
	margin-top: 0.5rem;
}

.account-manager__add-button {
	width: 100%;
}

.account-manager__add-choice {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
}

.account-manager__skeleton {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
	padding: 0.25rem 0;
}

.account-manager__skeleton-row {
	display: flex;
	align-items: center;
	gap: 0.625rem;
	animation: account-manager-skeleton 1.5s ease-in-out infinite;
}

.account-manager__skeleton-avatar {
	width: 2rem;
	height: 2rem;
	border-radius: 50%;
	background: var(--shell-border);
	flex-shrink: 0;
}

.account-manager__skeleton-lines {
	display: flex;
	flex-direction: column;
	gap: 0.25rem;
	flex: 1;
}

.account-manager__skeleton-line {
	height: 0.5rem;
	border-radius: 4px;
	background: var(--shell-border);
}

.account-manager__skeleton-line--long {
	width: 70%;
}

.account-manager__skeleton-line--short {
	width: 40%;
}

@keyframes account-manager-skeleton {
	0%, 100% {
		opacity: 1;
	}
	50% {
		opacity: 0.4;
	}
}

.account-manager__lumen-card {
	display: flex;
	align-items: center;
	gap: 0.625rem;
	padding: 0.625rem 0.75rem;
	border-radius: 10px;
	background: var(--shell-card);
	border: 1px solid var(--shell-border);
}

.account-manager__lumen-dot {
	width: 8px;
	height: 8px;
	border-radius: 50%;
	background: var(--emerus-accent-bright);
	flex-shrink: 0;
}

.account-manager__lumen-text {
	display: flex;
	flex-direction: column;
	min-width: 0;
	flex: 1;
}

.account-manager__lumen-name {
	font-size: 0.875rem;
	font-weight: 600;
	color: var(--color-contrast);
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.account-manager__lumen-nick {
	font-size: 0.75rem;
	color: var(--color-secondary);
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.account-manager__lumen-empty {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
}

.account-manager__lumen-hint {
	margin: 0;
	font-size: 0.8rem;
	line-height: 1.4;
	color: var(--color-secondary);
}
</style>
