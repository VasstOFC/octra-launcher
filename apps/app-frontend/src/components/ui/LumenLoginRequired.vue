<template>
	<div
		class="lumen-login-required fixed inset-0 z-[100] flex items-center justify-center"
		data-tauri-drag-region
	>
		<div class="login-bg-image" />
		<div class="login-bg-overlay" />
		<div class="login-drag-bar" data-tauri-drag-region />
		<div class="relative flex flex-col items-center gap-6 w-full max-w-sm px-6 login-card">
			<div class="flex items-center gap-3" data-tauri-drag-region>
				<img src="@/assets/brand/lumen-mark.png" alt="" class="h-8 w-8" />
				<span class="text-lg font-bold text-contrast">Lumen App</span>
			</div>

			<p class="text-center text-secondary text-sm m-0">
				{{ formatMessage(messages.description) }}
			</p>

			<div class="flex flex-col gap-3 w-full">
				<template v-if="mode === 'login'">
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
				<template v-else>
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
			</div>

			<div class="flex flex-col gap-2 w-full">
				<Button
					type="colored"
					color="brand"
					:disabled="loading || !canSubmit"
					class="w-full"
					@click="submit"
				>
					<SpinnerIcon v-if="loading" class="animate-spin" />
					<LogInIcon v-else />
					{{ formatMessage(mode === 'login' ? messages.loginButton : messages.registerButton) }}
				</Button>
				<Button type="outlined" :disabled="loading" class="w-full" @click="toggleMode">
					{{ formatMessage(mode === 'login' ? messages.switchToRegister : messages.switchToLogin) }}
				</Button>
			</div>

			<p class="text-center text-xs text-secondary m-0 mt-2">
				{{ formatMessage(messages.footer) }}
			</p>
		</div>
	</div>
</template>

<script setup lang="ts">
import { LogInIcon, SpinnerIcon } from '@lumen/assets'
import { Button, defineMessages, Input, useVIntl } from '@lumen/ui'
import { computed, ref } from 'vue'

import { handleSevereError } from '@/composables/use-error.js'
import { get_default_user, isOfflineAccount, users } from '@/helpers/auth'
import { LumenAccountLogin, LumenAccountRegister } from '@/helpers/lumen-account.js'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	success: []
}>()

const messages = defineMessages({
	description: {
		id: 'Lumen-login-required.description',
		defaultMessage: 'Zaloguj się do konta Lumen, aby uruchomić launcher.',
	},
	usernamePlaceholder: {
		id: 'Lumen-login-required.username',
		defaultMessage: 'Nazwa gracza Minecraft',
	},
	passwordPlaceholder: {
		id: 'Lumen-login-required.password',
		defaultMessage: 'Hasło',
	},
	loginButton: {
		id: 'Lumen-login-required.login-button',
		defaultMessage: 'Zaloguj się',
	},
	registerButton: {
		id: 'Lumen-login-required.register-button',
		defaultMessage: 'Utwórz konto',
	},
	switchToRegister: {
		id: 'Lumen-login-required.switch-to-register',
		defaultMessage: 'Nie masz konta? Utwórz je',
	},
	switchToLogin: {
		id: 'Lumen-login-required.switch-to-login',
		defaultMessage: 'Masz już konto? Zaloguj się',
	},
	registerHint: {
		id: 'Lumen-login-required.register-hint',
		defaultMessage:
			'Lumen łączy się z Twoim kontem Minecraft w celu wyświetlania skórek. Nie tworzy konta Minecraft.',
	},
	linkedMinecraft: {
		id: 'Lumen-login-required.linked-minecraft',
		defaultMessage: 'Konto Minecraft',
	},
	noMinecraftAccount: {
		id: 'Lumen-login-required.no-minecraft',
		defaultMessage: 'Najpierw dodaj konto Microsoft lub offline Minecraft.',
	},
	nonPremium: {
		id: 'minecraft-account.non-premium',
		defaultMessage: 'Non-premium',
	},
	footer: {
		id: 'Lumen-login-required.footer',
		defaultMessage: 'Lumen App - Launcher do Minecrafta',
	},
})

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
		const selected = list.find((account) => account?.profile?.id === defaultId) || list[0] || null
		if (selected?.profile?.name) {
			linkedNick.value = selected.profile.name
			linkedIsOffline.value = isOfflineAccount(selected)
			if (mode.value === 'login' && !username.value) {
				username.value = selected.profile.name
			}
		}
	} catch (error) {
		console.error('Failed to load linked Minecraft account', error)
	}
}

function toggleMode() {
	mode.value = mode.value === 'login' ? 'register' : 'login'
	void loadLinkedMinecraft()
}

async function submit() {
	if (!canSubmit.value || loading.value) return
	loading.value = true

	try {
		if (mode.value === 'register') {
			await LumenAccountRegister(password.value)
		} else {
			await LumenAccountLogin(username.value.trim(), password.value)
		}
		emit('success')
	} catch (error) {
		handleSevereError(error)
	} finally {
		loading.value = false
	}
}

void loadLinkedMinecraft()
</script>

<style scoped>
.lumen-login-required {
	background: #0a0a0f;
}

.login-bg-image {
	position: absolute;
	inset: 0;
	background-image: url('@/assets/login-bg.png');
	background-size: cover;
	background-position: center;
	opacity: 0.35;
	pointer-events: none;
}

.login-bg-overlay {
	position: absolute;
	inset: 0;
	background: linear-gradient(180deg, rgba(10, 10, 15, 0.6) 0%, rgba(10, 10, 15, 0.85) 100%);
	pointer-events: none;
}

.login-drag-bar {
	position: absolute;
	top: 0;
	left: 0;
	right: 0;
	height: 36px;
}

.login-card {
	z-index: 1;
	background: rgba(15, 15, 25, 0.7);
	backdrop-filter: blur(20px);
	-webkit-backdrop-filter: blur(20px);
	border: 1px solid rgba(255, 255, 255, 0.06);
	border-radius: 16px;
	padding: 2.5rem 2rem;
	box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);

	:deep(input),
	:deep(textarea),
	:deep(select) {
		background: transparent !important;
		border: 1px solid rgba(255, 255, 255, 0.08) !important;
		box-shadow: none !important;
		border-radius: 10px;

		&:focus,
		&:focus-visible {
			border-color: var(--emerus-primary) !important;
			box-shadow: 0 0 0 2px var(--emerus-primary-glow) !important;
		}
	}
}
</style>
