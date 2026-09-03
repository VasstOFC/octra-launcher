<script setup lang="ts">
import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

import JavaSelector from '@/components/ui/JavaSelector.vue'
import {
	SettingsGroup,
	SettingsPanelHeader,
	SettingsStack,
	SettingsTwoCol,
} from '@/components/ui/settings/_shared'
import { get_java_versions, set_java_version } from '@/helpers/jre'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	panelTitle: {
		id: 'app.settings.java-installations.title',
		defaultMessage: 'Java',
	},
	panelDescription: {
		id: 'app.settings.java-installations.description',
		defaultMessage: 'Choose which Java installations Octra uses for each major version.',
	},
	installationsGroup: {
		id: 'app.settings.java-installations.group',
		defaultMessage: 'Installations',
	},
	javaLocation: {
		id: 'app.settings.java-installations.location.title',
		defaultMessage: 'Java {version, number} location',
	},
})

const javaVersions = ref(await get_java_versions().catch(handleError))

const javaPairs: Array<[number, number]> = [
	[25, 21],
	[17, 8],
]

async function updateJavaVersion(version: { path?: string } | null | undefined) {
	if (version?.path === '') {
		version.path = undefined
	}

	if (version?.path) {
		version.path = version.path.replace('java.exe', 'javaw.exe')
	}

	await set_java_version(version).catch(handleError)
}
</script>

<template>
	<div>
		<SettingsPanelHeader
			:title="formatMessage(messages.panelTitle)"
			:description="formatMessage(messages.panelDescription)"
		/>

		<SettingsGroup :label="formatMessage(messages.installationsGroup)">
			<SettingsTwoCol v-for="([leftVersion, rightVersion], pairIndex) in javaPairs" :key="pairIndex">
				<template #left>
					<SettingsStack
						:control-id="`java-selector-${leftVersion}`"
						:title="formatMessage(messages.javaLocation, { version: leftVersion })"
					>
						<template #default="{ controlId }">
							<JavaSelector
								:id="controlId"
								v-model="javaVersions[leftVersion]"
								:version="leftVersion"
								@update:model-value="updateJavaVersion"
							/>
						</template>
					</SettingsStack>
				</template>
				<template #right>
					<SettingsStack
						:control-id="`java-selector-${rightVersion}`"
						:title="formatMessage(messages.javaLocation, { version: rightVersion })"
					>
						<template #default="{ controlId }">
							<JavaSelector
								:id="controlId"
								v-model="javaVersions[rightVersion]"
								:version="rightVersion"
								@update:model-value="updateJavaVersion"
							/>
						</template>
					</SettingsStack>
				</template>
			</SettingsTwoCol>
		</SettingsGroup>
	</div>
</template>
