<script setup lang="ts">
import { LogInIcon, LogOutIcon, UserPlusIcon } from '@lumen/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	useVIntl,
} from '@lumen/ui'
import { computed, ref } from 'vue'

import LumenAccountModal from '@/components/ui/LumenAccountModal.vue'
import { SettingsGroup, SettingsPanelHeader, SettingsStack } from '@/components/ui/settings/_shared'
import { LumenAccountLogout, LumenAccountSession } from '@/helpers/lumen-account.js'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

type LumenSession = {
	username: string
	minecraft_nick: string
}

const loading = ref(false)
const session = ref<LumenSession | null>(null)
const lumenAccountModal = ref<InstanceType<typeof LumenAccountModal>>()

const signedIn = computed(() => !!session.value)

const sessionDescription = computed(() => {
	if (!session.value) {
		return formatMessage(messages.signedOutDescription)
	}
	return formatMessage(messages.signedInDescription, {
		username: session.value.username,
		minecraftNick: session.value.minecraft_nick,
	})
})

const messages = defineMessages({
	panelTitle: {
		id: 'app.settings.account.panel.title',
		defaultMessage: 'Account',
	},
	panelDescription: {
		id: 'app.settings.account.panel.description',
		defaultMessage: 'Sign in to Lumen for skins, community, and chat.',
	},
	sessionGroup: {
		id: 'app.settings.account.session.group',
		defaultMessage: 'Lumen session',
	},
	signedInTitle: {
		id: 'app.settings.account.session.signed-in',
		defaultMessage: 'Signed in',
	},
	signedOutTitle: {
		id: 'app.settings.account.session.signed-out',
		defaultMessage: 'Not signed in',
	},
	signedInDescription: {
		id: 'app.settings.account.session.signed-in-description',
		defaultMessage: '{username} Â· {minecraftNick}',
	},
	signedOutDescription: {
		id: 'Lumen-account.hint',
		defaultMessage:
			'Lumen links skins to your Minecraft account â€” it does not create one. Add Microsoft or offline first.',
	},
	login: {
		id: 'Lumen-account.login',
		defaultMessage: 'Log in',
	},
	register: {
		id: 'Lumen-account.register',
		defaultMessage: 'Connect',
	},
	manage: {
		id: 'app.settings.account.manage',
		defaultMessage: 'Manage',
	},
})

async function refreshSession() {
	session.value = await LumenAccountSession().catch(() => null)
}

function openLumenAccount(mode: 'login' | 'register' = 'login') {
	lumenAccountModal.value?.show(mode)
}

async function onLumenAccountSuccess() {
	loading.value = true
	try {
		await refreshSession()
	} finally {
		loading.value = false
	}
}

async function logoutLumen() {
	loading.value = true
	try {
		await LumenAccountLogout()
		session.value = null
	} catch (error) {
		handleError(error)
	} finally {
		loading.value = false
	}
}

await refreshSession()
</script>

<template>
	<div>
		<SettingsPanelHeader
			:title="formatMessage(messages.panelTitle)"
			:description="formatMessage(messages.panelDescription)"
		/>

		<SettingsGroup :label="formatMessage(messages.sessionGroup)">
			<SettingsStack
				:title="
					signedIn ? formatMessage(messages.signedInTitle) : formatMessage(messages.signedOutTitle)
				"
				:description="sessionDescription"
			>
				<div class="flex flex-wrap gap-2">
					<template v-if="signedIn">
						<Button type="outlined" :disabled="loading" @click="openLumenAccount('login')">
							{{ formatMessage(messages.manage) }}
						</Button>
						<Button type="outlined" :disabled="loading" @click="logoutLumen">
							<LogOutIcon />
							{{ formatMessage(commonMessages.signOutButton) }}
						</Button>
					</template>
					<template v-else>
						<Button
							type="colored"
							color="brand"
							:disabled="loading"
							@click="openLumenAccount('login')"
						>
							<LogInIcon />
							{{ formatMessage(messages.login) }}
						</Button>
						<Button :disabled="loading" @click="openLumenAccount('register')">
							<UserPlusIcon />
							{{ formatMessage(messages.register) }}
						</Button>
					</template>
				</div>
			</SettingsStack>
		</SettingsGroup>

		<LumenAccountModal ref="lumenAccountModal" @success="onLumenAccountSuccess" />
	</div>
</template>
