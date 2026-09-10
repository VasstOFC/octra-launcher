<template>
	<div
		class="flex flex-col gap-2 bg-button-bg border border-solid border-surface-5 rounded-xl p-3 mt-2"
	>
		<div class="flex items-center justify-between gap-2 min-w-0">
			<div class="flex flex-col min-w-0">
				<span class="text-contrast font-semibold truncate">
					{{ formatMessage(messages.LumenAccount) }}
				</span>
				<span v-if="lumenSession" class="text-secondary text-xs truncate">
					{{ lumenSession.username }} Â· {{ lumenSession.minecraft_nick }}
				</span>
				<span v-else class="text-secondary text-xs">
					{{ formatMessage(messages.LumenAccountHint) }}
				</span>
			</div>
			<Button v-if="lumenSession" type="outlined" :disabled="lumenLoading" @click="logoutLumen">
				{{ formatMessage(messages.LumenLogout) }}
			</Button>
			<Button
				v-else
				type="colored"
				color="brand"
				:disabled="lumenLoading"
				@click="openLumenAccount('login')"
			>
				<LogInIcon />
				{{ formatMessage(messages.LumenLogin) }}
			</Button>
		</div>
		<Button
			v-if="!lumenSession"
			class="w-full"
			:disabled="lumenLoading"
			@click="openLumenAccount('register')"
		>
			<PlusIcon />
			{{ formatMessage(messages.LumenRegister) }}
		</Button>
	</div>
	<div
		v-if="accounts.length === 0"
		class="flex flex-col gap-3 bg-button-bg border border-solid border-surface-5 rounded-xl p-3 mt-2"
	>
		<span>{{ formatMessage(messages.notSignedIn) }}</span>
		<Button type="colored" color="brand" :disabled="loginDisabled" @click="login()">
			<LogInIcon v-if="!loginDisabled" />
			<SpinnerIcon v-else class="animate-spin" />
			{{ formatMessage(messages.addMicrosoftAccount) }}
		</Button>
		<Button :disabled="loginDisabled" @click="addOffline()">
			<PlusIcon />
			{{ formatMessage(messages.addOfflineAccount) }}
		</Button>
	</div>
	<Accordion
		v-else
		class="w-full mt-2 bg-button-bg border border-solid border-surface-5 rounded-xl overflow-clip"
		button-class="button-base w-full bg-transparent px-3 py-2 border-0 cursor-pointer"
		:open-by-default="true"
	>
		<template #title>
			<div class="flex gap-2 w-full min-w-0">
				<Avatar
					size="36px"
					:src="
						selectedAccount
							? avatarUrl
							: 'https://launcher-files.modrinth.com/assets/steve_head.png'
					"
				/>
				<div class="flex flex-col items-start w-full min-w-0">
					<span class="flex items-center gap-2 truncate w-full text-left">
						<span class="truncate">{{
							selectedAccount ? selectedAccount.profile.name : formatMessage(messages.selectAccount)
						}}</span>
						<span
							v-if="selectedAccount && isOfflineAccount(selectedAccount)"
							class="shrink-0 rounded-full bg-surface-3 px-1.5 py-0.5 text-[0.65rem] font-semibold leading-none text-secondary"
						>
							{{ formatMessage(messages.nonPremium) }}
						</span>
					</span>
					<span class="text-secondary text-xs">{{ formatMessage(messages.minecraftAccount) }}</span>
				</div>
			</div>
		</template>
		<div class="bg-button-bg pt-1 pb-2 border-0 border-t border-solid border-surface-5">
			<template v-if="accounts.length > 0">
				<div v-for="account in accounts" :key="account.profile.id" class="flex gap-1 items-center">
					<button
						class="flex items-center flex-shrink flex-grow overflow-clip gap-2 p-2 border-0 bg-transparent cursor-pointer button-base min-w-0"
						@click="setAccount(account)"
					>
						<RadioButtonCheckedIcon
							v-if="selectedAccount && selectedAccount.profile.id === account.profile.id"
							class="w-5 h-5 text-brand shrink-0"
						/>
						<RadioButtonIcon v-else class="w-5 h-5 text-secondary shrink-0" />
						<Avatar :src="getAccountAvatarUrlForList(account)" size="24px" />
						<p
							class="m-0 truncate min-w-0"
							:class="
								selectedAccount && selectedAccount.profile.id === account.profile.id
									? 'text-contrast font-semibold'
									: 'text-primary'
							"
						>
							{{ account.profile.name }}
						</p>
						<span
							v-if="isOfflineAccount(account)"
							class="shrink-0 rounded-full bg-surface-3 px-1.5 py-0.5 text-[0.65rem] font-semibold leading-none text-secondary"
						>
							{{ formatMessage(messages.nonPremium) }}
						</span>
					</button>
					<IconButton
						v-tooltip="formatMessage(messages.removeAccount)"
						type="quiet"
						color="red"
						:label="formatMessage(messages.removeAccount)"
						class="mr-2 !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)] hover:!bg-red focus-visible:!bg-red hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
						@click="logout(account.profile.id)"
					>
						<TrashIcon />
					</IconButton>
				</div>
			</template>
			<div class="flex flex-col gap-2 px-2 pt-2">
				<Button
					class="w-full !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
					:disabled="loginDisabled"
					@click="login()"
				>
					<PlusIcon />
					{{ formatMessage(messages.addMicrosoftAccount) }}
				</Button>
				<Button
					class="w-full !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
					:disabled="loginDisabled"
					@click="addOffline()"
				>
					<PlusIcon />
					{{ formatMessage(messages.addOfflineAccount) }}
				</Button>
			</div>
		</div>
	</Accordion>
	<AddOfflineAccountModal ref="addOfflineModal" @added="onOfflineAdded" />
			<LumenAccountModal ref="lumenAccountModal" @success="onLumenAccountSuccess" />
</template>

<script setup lang="ts">
import {
	LogInIcon,
	PlusIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	SpinnerIcon,
	TrashIcon,
} from '@lumen/assets'
import {
	Accordion,
	Avatar,
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@lumen/ui'
import type { Ref } from 'vue'
import { computed, ref } from 'vue'

import AddOfflineAccountModal from '@/components/ui/AddOfflineAccountModal.vue'
import LumenAccountModal from '@/components/ui/LumenAccountModal.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { useMinecraftAccountAvatar } from '@/composables/use-minecraft-account-avatar.ts'
import { trackEvent } from '@/helpers/analytics'
import {
	get_default_user,
	isOfflineAccount,
	login as login_flow,
	remove_user,
	set_default_user,
	users,
} from '@/helpers/auth'
import { LumenAccountLogout, LumenAccountSession } from '@/helpers/lumen-account.js'
import type { Skin } from '@/helpers/skins'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const emit = defineEmits<{
	change: []
}>()

type MinecraftCredential = {
	profile: {
		id: string
		name: string
	}
	is_offline?: boolean
	refresh_token?: string
}

const accounts: Ref<MinecraftCredential[]> = ref([])
const loginDisabled = ref(false)
const lumenLoading = ref(false)
const lumenSession = ref<{
	username: string
	minecraft_nick: string
} | null>(null)
const defaultUser = ref<string | undefined>()
const addOfflineModal = ref<InstanceType<typeof AddOfflineAccountModal>>()
const lumenAccountModal = ref<InstanceType<typeof LumenAccountModal>>()

const { refreshEquippedSkinAvatar, setEquippedSkinAvatar, getAccountAvatarUrl } =
	useMinecraftAccountAvatar()

async function refreshValues() {
	defaultUser.value = await get_default_user().catch(handleError)
	const userList = await users().catch(handleError)
	accounts.value = Array.isArray(userList) ? [...userList] : []
	accounts.value.sort((a, b) => (a.profile?.name ?? '').localeCompare(b.profile?.name ?? ''))
	lumenSession.value = await LumenAccountSession().catch(() => null)
	await refreshEquippedSkinAvatar(accounts.value)
}

async function setEquippedSkin(skin: Skin) {
	await setEquippedSkinAvatar(skin)
}

function setLoginDisabled(value: boolean) {
	loginDisabled.value = value
}

function addOffline() {
	addOfflineModal.value?.show()
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
	} catch (error) {
		handleError(error)
	} finally {
		lumenLoading.value = false
	}
}

defineExpose({
	refreshValues,
	setEquippedSkin,
	setLoginDisabled,
	login,
	addOffline,
	loginDisabled,
})

await refreshValues()

const selectedAccount = computed(() =>
	accounts.value.find((account) => account.profile.id === defaultUser.value),
)

const avatarUrl = computed(() =>
	getAccountAvatarUrl(
		selectedAccount.value?.profile?.id,
		true,
		selectedAccount.value ? isOfflineAccount(selectedAccount.value) : false,
	),
)

function getAccountAvatarUrlForList(account: MinecraftCredential) {
	return getAccountAvatarUrl(
		account.profile.id,
		account.profile.id === selectedAccount.value?.profile?.id,
		isOfflineAccount(account),
	)
}

async function setAccount(account: MinecraftCredential) {
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch(handleError)
	await refreshValues()
	emit('change')
}

async function login() {
	loginDisabled.value = true
	const loggedIn = await login_flow().catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
	}

	trackEvent('AccountLogIn')
	loginDisabled.value = false
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

async function logout(id: string) {
	await remove_user(id).catch(handleError)
	await refreshValues()
	if (!selectedAccount.value && accounts.value.length > 0) {
		await setAccount(accounts.value[0])
	} else {
		emit('change')
	}
	trackEvent('AccountLogOut')
}

useAppEvent('process', async (e) => {
	if (e.event === 'launched') {
		await refreshValues()
	}
})

const messages = defineMessages({
	notSignedIn: {
		id: 'minecraft-account.not-signed-in',
		defaultMessage: 'Not signed in',
	},
	addAccount: {
		id: 'minecraft-account.add-account',
		defaultMessage: 'Add account',
	},
	addOfflineAccount: {
		id: 'minecraft-account.add-offline',
		defaultMessage: 'Add offline account',
	},
	removeAccount: {
		id: 'minecraft-account.remove-account',
		defaultMessage: 'Remove account',
	},
	selectAccount: {
		id: 'minecraft-account.select-account',
		defaultMessage: 'Select account',
	},
	minecraftAccount: {
		id: 'minecraft-account.label',
		defaultMessage: 'Minecraft account',
	},
	signInToLumen: {
		id: 'minecraft-account.sign-in',
		defaultMessage: 'Log in to Lumen',
	},
	addMicrosoftAccount: {
		id: 'minecraft-account.add-microsoft',
		defaultMessage: 'Add Microsoft account',
	},
	nonPremium: {
		id: 'minecraft-account.non-premium',
		defaultMessage: 'Non-premium',
	},
	LumenAccount: {
			id: 'lumen-account.label',
		defaultMessage: 'Lumen account',
	},
	LumenAccountHint: {
			id: 'lumen-account.hint',
		defaultMessage:
			'Lumen links skins to your Minecraft account â€” it does not create one. Add Microsoft or offline first.',
	},
	LumenLogin: {
			id: 'lumen-account.login',
		defaultMessage: 'Log in',
	},
	LumenRegister: {
			id: 'lumen-account.register',
		defaultMessage: 'Connect',
	},
	LumenLogout: {
			id: 'lumen-account.logout',
		defaultMessage: 'Log out',
	},
})
</script>
