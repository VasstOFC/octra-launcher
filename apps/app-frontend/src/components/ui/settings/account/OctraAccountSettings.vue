<script setup lang="ts">
import { LogInIcon, LogOutIcon, UserPlusIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import OctraAccountModal from '@/components/ui/OctraAccountModal.vue'
import {
	SettingsGroup,
	SettingsPanelHeader,
	SettingsStack,
} from '@/components/ui/settings/_shared'
import { octraAccountLogout, octraAccountSession } from '@/helpers/octra-account.js'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

type OctraSession = {
	username: string
	minecraft_nick: string
}

const loading = ref(false)
const session = ref<OctraSession | null>(null)
const octraAccountModal = ref<InstanceType<typeof OctraAccountModal>>()

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
		defaultMessage: 'Sign in to Octra for skins, community, and chat.',
	},
	sessionGroup: {
		id: 'app.settings.account.session.group',
		defaultMessage: 'Octra session',
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
		defaultMessage: '{username} · {minecraftNick}',
	},
	signedOutDescription: {
		id: 'octra-account.hint',
		defaultMessage:
			'Octra links skins to your Minecraft account — it does not create one. Add Microsoft or offline first.',
	},
	login: {
		id: 'octra-account.login',
		defaultMessage: 'Log in',
	},
	register: {
		id: 'octra-account.register',
		defaultMessage: 'Connect',
	},
	manage: {
		id: 'app.settings.account.manage',
		defaultMessage: 'Manage',
	},
})

async function refreshSession() {
	session.value = await octraAccountSession().catch(() => null)
}

function openOctraAccount(mode: 'login' | 'register' = 'login') {
	octraAccountModal.value?.show(mode)
}

async function onOctraAccountSuccess() {
	loading.value = true
	try {
		await refreshSession()
	} finally {
		loading.value = false
	}
}

async function logoutOctra() {
	loading.value = true
	try {
		await octraAccountLogout()
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
					signedIn
						? formatMessage(messages.signedInTitle)
						: formatMessage(messages.signedOutTitle)
				"
				:description="sessionDescription"
			>
				<div class="flex flex-wrap gap-2">
					<template v-if="signedIn">
						<Button type="outlined" :disabled="loading" @click="openOctraAccount('login')">
							{{ formatMessage(messages.manage) }}
						</Button>
						<Button
							type="outlined"
							:disabled="loading"
							@click="logoutOctra"
						>
							<LogOutIcon />
							{{ formatMessage(commonMessages.signOutButton) }}
						</Button>
					</template>
					<template v-else>
						<Button
							type="colored"
							color="brand"
							:disabled="loading"
							@click="openOctraAccount('login')"
						>
							<LogInIcon />
							{{ formatMessage(messages.login) }}
						</Button>
						<Button :disabled="loading" @click="openOctraAccount('register')">
							<UserPlusIcon />
							{{ formatMessage(messages.register) }}
						</Button>
					</template>
				</div>
			</SettingsStack>
		</SettingsGroup>

		<OctraAccountModal ref="octraAccountModal" @success="onOctraAccountSuccess" />
	</div>
</template>
