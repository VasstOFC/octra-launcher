<script setup lang="ts">
import { Settings2Icon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectNotificationManager,
	injectPageContext,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { ref, watch } from 'vue'

import {
	SettingsGroup,
	SettingsPanelHeader,
	SettingsRow,
} from '@/components/ui/settings/_shared'
import { open_ads_consent_preferences } from '@/helpers/ads.js'
import { get, set } from '@/helpers/settings.ts'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const { adConsentAvailable } = injectPageContext()
const settings = ref(await get())
if (settings.value.telemetry) {
	settings.value.telemetry = false
	await set(settings.value)
}

const messages = defineMessages({
	panelTitle: {
		id: 'app.settings.privacy.panel.title',
		defaultMessage: 'Privacy',
	},
	panelDescription: {
		id: 'app.settings.privacy.panel.description',
		defaultMessage: 'Ads preferences and data collection.',
	},
	adsGroup: {
		id: 'app.settings.privacy.ads.group',
		defaultMessage: 'Ads',
	},
	adsConsentTitle: {
		id: 'app.ads-consent.title',
		defaultMessage: 'Your privacy and how ads support Modrinth',
	},
	adsConsentIntro: {
		id: 'app.settings.privacy.ads-consent.intro',
		defaultMessage:
			'Ads make Modrinth possible and fund creator payouts. Our partners may store or access cookies in the app to personalize ads and measure performance. You can opt out or manage your preferences below.',
	},
	adsConsentManage: {
		id: 'app.ads-consent.manage',
		defaultMessage: 'Manage preferences',
	},
	dataGroup: {
		id: 'app.settings.privacy.data.group',
		defaultMessage: 'Data',
	},
	telemetryTitle: {
		id: 'app.settings.privacy.telemetry.title',
		defaultMessage: 'Telemetry',
	},
	telemetryDescription: {
		id: 'app.settings.privacy.telemetry.description',
		defaultMessage: 'Usage analytics stay off in Octra App.',
	},
})

async function manageAdsPreferences() {
	await open_ads_consent_preferences().catch(handleError)
}

watch(
	settings,
	async () => {
		await set({ ...settings.value, telemetry: false })
	},
	{ deep: true },
)
</script>

<template>
	<div>
		<SettingsPanelHeader
			:title="formatMessage(messages.panelTitle)"
			:description="formatMessage(messages.panelDescription)"
		/>

		<SettingsGroup
			v-if="adConsentAvailable"
			:label="formatMessage(messages.adsGroup)"
		>
			<SettingsRow
				:title="formatMessage(messages.adsConsentTitle)"
				:description="formatMessage(messages.adsConsentIntro)"
			>
				<Button @click="manageAdsPreferences">
					<Settings2Icon aria-hidden="true" />
					{{ formatMessage(messages.adsConsentManage) }}
				</Button>
			</SettingsRow>
		</SettingsGroup>

		<SettingsGroup :label="formatMessage(messages.dataGroup)">
			<SettingsRow
				control-id="telemetry"
				:title="formatMessage(messages.telemetryTitle)"
				:description="formatMessage(messages.telemetryDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Toggle
						:id="controlId"
						:model-value="false"
						disabled
						:aria-labelledby="labelledBy"
					/>
				</template>
			</SettingsRow>
		</SettingsGroup>
	</div>
</template>
