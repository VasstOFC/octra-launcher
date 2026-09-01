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
					@click="mode = 'register'"
				>
					{{ formatMessage(messages.registerTab) }}
				</Button>
			</div>
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
			<Input
				v-if="mode === 'register'"
				v-model="minecraftNick"
				maxlength="16"
				:placeholder="formatMessage(messages.minecraftNickPlaceholder)"
				:disabled="loading"
				@keyup.enter="submit"
			/>
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
		defaultMessage: 'Register',
	},
	usernamePlaceholder: {
		id: 'octra-account.username',
		defaultMessage: 'Username',
	},
	passwordPlaceholder: {
		id: 'octra-account.password',
		defaultMessage: 'Password',
	},
	minecraftNickPlaceholder: {
		id: 'octra-account.minecraft-nick',
		defaultMessage: 'Minecraft nickname',
	},
	loginSuccess: {
		id: 'octra-account.login-success',
		defaultMessage: 'Logged in to Octra',
	},
	registerSuccess: {
		id: 'octra-account.register-success',
		defaultMessage: 'Octra account created',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const mode = ref<'login' | 'register'>('login')
const username = ref('')
const password = ref('')
const minecraftNick = ref('')
const loading = ref(false)

const validNick = computed(() => {
	const name = minecraftNick.value.trim()
	return name.length >= 1 && name.length <= 16 && /^[A-Za-z0-9_]+$/.test(name)
})

const canSubmit = computed(() => {
	const user = username.value.trim()
	const pass = password.value
	if (user.length < 3 || pass.length < 8) return false
	if (mode.value === 'register') return validNick.value
	return true
})

function show(initialMode: 'login' | 'register' = 'login') {
	mode.value = initialMode
	username.value = ''
	password.value = ''
	minecraftNick.value = ''
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
			await octraAccountRegister(
				username.value.trim(),
				password.value,
				minecraftNick.value.trim(),
			)
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
