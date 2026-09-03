<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)" max-width="420px">
		<div class="flex flex-col gap-3">
			<div class="flex gap-2">
				<Button
					class="flex-1"
					:type="mode === 'login' ? 'colored' : 'outlined'"
					color="brand"
					:disabled="loading"
					@click="mode = 'login'"
				>
					{{ formatMessage(messages.loginTab) }}
				</Button>
				<Button
					class="flex-1"
					:type="mode === 'register' ? 'colored' : 'outlined'"
					color="brand"
					:disabled="loading"
					@click="switchToRegister"
				>
					{{ formatMessage(messages.registerTab) }}
				</Button>
			</div>

			<template v-if="mode === 'register'">
				<p class="m-0 text-sm text-secondary">
					{{ formatMessage(messages.registerHint) }}
				</p>
				<div
					v-if="linkedNick"
					class="rounded-xl border border-solid border-surface-5 bg-surface-3 px-3 py-2"
				>
					<p class="m-0 text-xs text-secondary">
						{{ formatMessage(messages.linkedMinecraft) }}
					</p>
					<p class="m-0 mt-0.5 truncate font-semibold text-contrast">
						{{ linkedNick }}
						<span
							v-if="linkedIsOffline"
							class="ml-1 rounded-full bg-surface-5 px-1.5 py-0.5 text-[0.65rem] font-semibold leading-none text-secondary"
						>
							{{ formatMessage(messages.nonPremium) }}
						</span>
					</p>
				</div>
				<p v-else class="m-0 text-sm text-red">
					{{ formatMessage(messages.noMinecraftAccount) }}
				</p>
				<Input
					v-model="password"
					type="password"
					:placeholder="formatMessage(messages.passwordPlaceholder)"
					:disabled="loading || !linkedNick"
					autocomplete="new-password"
					@keyup.enter="submit"
				/>
			</template>

			<template v-else>
				<Input
					v-model="username"
					:placeholder="formatMessage(messages.usernamePlaceholder)"
					:disabled="loading"
					autocomplete="username"
				/>
				<Input
					v-model="password"
					type="password"
					:placeholder="formatMessage(messages.passwordPlaceholder)"
					:disabled="loading"
					autocomplete="current-password"
					@keyup.enter="submit"
				/>
			</template>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" :disabled="loading" @click="hide">
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="loading || !canSubmit"
					@click="submit"
				>
					<SpinnerIcon v-if="loading" class="animate-spin" />
					<LogInIcon v-else />
					{{ formatMessage(mode === 'login' ? messages.loginTab : messages.registerTab) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { LogInIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	Input,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { handleSevereError } from '@/composables/use-error.js'
import {
	get_default_user,
	isOfflineAccount,
	users,
} from '@/helpers/auth'
import {
	octraAccountLogin,
	octraAccountRegister,
} from '@/helpers/octra-account.js'

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()

const emit = defineEmits<{
	success: []
}>()

const messages = defineMessages({
	title: {
		id: 'octra-account.title',
		defaultMessage: 'Octra account',
	},
	loginTab: {
		id: 'octra-account.login',
		defaultMessage: 'Log in',
	},
	registerTab: {
		id: 'octra-account.register',
		defaultMessage: 'Connect',
	},
	usernamePlaceholder: {
		id: 'octra-account.username',
		defaultMessage: 'Minecraft nickname',
	},
	passwordPlaceholder: {
		id: 'octra-account.password',
		defaultMessage: 'Password',
	},
	registerHint: {
		id: 'octra-account.register-hint',
		defaultMessage:
			'Octra links to your current Minecraft account for skins. It does not create a Minecraft account.',
	},
	linkedMinecraft: {
		id: 'octra-account.linked-minecraft',
		defaultMessage: 'Minecraft account',
	},
	noMinecraftAccount: {
		id: 'octra-account.no-minecraft',
		defaultMessage: 'Add a Microsoft or offline Minecraft account first.',
	},
	nonPremium: {
		id: 'minecraft-account.non-premium',
		defaultMessage: 'Non-premium',
	},
	loginSuccess: {
		id: 'octra-account.login-success',
		defaultMessage: 'Logged in to Octra',
	},
	registerSuccess: {
		id: 'octra-account.register-success',
		defaultMessage: 'Octra account linked',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const mode = ref<'login' | 'register'>('login')
const username = ref('')
const password = ref('')
const linkedNick = ref('')
const linkedIsOffline = ref(false)
const loading = ref(false)

const canSubmit = computed(() => {
	const pass = password.value
	if (pass.length < 8) return false
	if (mode.value === 'register') return !!linkedNick.value
	return username.value.trim().length >= 1
})

async function loadLinkedMinecraft() {
	linkedNick.value = ''
	linkedIsOffline.value = false
	try {
		const defaultId = await get_default_user()
		const userList = await users()
		const list = Array.isArray(userList) ? userList : []
		const selected =
			list.find((account) => account?.profile?.id === defaultId) || list[0] || null
		if (selected?.profile?.name) {
			linkedNick.value = selected.profile.name
			linkedIsOffline.value = isOfflineAccount(selected)
			if (mode.value === 'login' && !username.value) {
				username.value = selected.profile.name
			}
		}
	} catch (error) {
		handleError(error)
	}
}

function switchToRegister() {
	mode.value = 'register'
	void loadLinkedMinecraft()
}

async function show(initialMode: 'login' | 'register' = 'login') {
	mode.value = initialMode
	username.value = ''
	password.value = ''
	await loadLinkedMinecraft()
	if (initialMode === 'login' && linkedNick.value) {
		username.value = linkedNick.value
	}
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

async function submit() {
	if (!canSubmit.value || loading.value) return
	loading.value = true

	try {
		if (mode.value === 'register') {
			await octraAccountRegister(password.value)
			addNotification({
				title: formatMessage(messages.registerSuccess),
				text: '',
				type: 'success',
			})
		} else {
			await octraAccountLogin(username.value.trim(), password.value)
			addNotification({
				title: formatMessage(messages.loginSuccess),
				text: '',
				type: 'success',
			})
		}
		emit('success')
		hide()
	} catch (error) {
		handleSevereError(error)
	} finally {
		loading.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>
