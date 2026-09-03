<script setup lang="ts">
import {
	defineMessages,
	injectNotificationManager,
	Input,
	Slider,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { ref, watch } from 'vue'

import {
	SettingsGroup,
	SettingsPanelHeader,
	SettingsRow,
	SettingsStack,
	SettingsTwoCol,
} from '@/components/ui/settings/_shared'
import useMemorySlider from '@/composables/useMemorySlider'
import { get, parseEnvVars, serializeEnvVars, set } from '@/helpers/settings.ts'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	panelTitle: {
		id: 'app.settings.launch-defaults.title',
		defaultMessage: 'Launch',
	},
	panelDescription: {
		id: 'app.settings.launch-defaults.description',
		defaultMessage:
			'Default window size, memory, Java arguments, and launch hooks for new instances.',
	},
	windowSectionTitle: {
		id: 'app.settings.default-instance-options.window.title',
		defaultMessage: 'Window',
	},
	javaAndMemorySectionTitle: {
		id: 'app.settings.default-instance-options.java-and-memory.title',
		defaultMessage: 'Java and memory',
	},
	launchHooksSectionTitle: {
		id: 'app.settings.default-instance-options.launch-hooks.title',
		defaultMessage: 'Launch hooks',
	},
	fullscreenTitle: {
		id: 'app.settings.default-instance-options.fullscreen.title',
		defaultMessage: 'Fullscreen',
	},
	fullscreenDescription: {
		id: 'app.settings.default-instance-options.fullscreen.description',
		defaultMessage: 'Start instances in fullscreen by updating their options.txt file.',
	},
	widthTitle: {
		id: 'app.settings.default-instance-options.width.title',
		defaultMessage: 'Width',
	},
	widthDescription: {
		id: 'app.settings.default-instance-options.width.description',
		defaultMessage: 'The width of the game window when launched.',
	},
	widthPlaceholder: {
		id: 'app.settings.default-instance-options.width.placeholder',
		defaultMessage: 'Enter width...',
	},
	heightTitle: {
		id: 'app.settings.default-instance-options.height.title',
		defaultMessage: 'Height',
	},
	heightDescription: {
		id: 'app.settings.default-instance-options.height.description',
		defaultMessage: 'The height of the game window when launched.',
	},
	heightPlaceholder: {
		id: 'app.settings.default-instance-options.height.placeholder',
		defaultMessage: 'Enter height...',
	},
	memoryAllocationTitle: {
		id: 'app.settings.default-instance-options.memory-allocation.title',
		defaultMessage: 'Memory allocation',
	},
	memoryAllocationDescription: {
		id: 'app.settings.default-instance-options.memory-allocation.description',
		defaultMessage: 'Maximum memory available to each instance.',
	},
	javaArgumentsTitle: {
		id: 'app.settings.default-instance-options.java-arguments.title',
		defaultMessage: 'Java arguments',
	},
	javaArgumentsPlaceholder: {
		id: 'app.settings.default-instance-options.java-arguments.placeholder',
		defaultMessage: 'Enter Java arguments...',
	},
	javaArgumentsDescription: {
		id: 'app.settings.default-instance-options.java-arguments.description',
		defaultMessage: 'Arguments passed to Java when launching an instance.',
	},
	environmentVariablesTitle: {
		id: 'app.settings.default-instance-options.environment-variables.title',
		defaultMessage: 'Environment variables',
	},
	environmentVariablesPlaceholder: {
		id: 'app.settings.default-instance-options.environment-variables.placeholder',
		defaultMessage: 'Enter environment variables...',
	},
	environmentVariablesDescription: {
		id: 'app.settings.default-instance-options.environment-variables.description',
		defaultMessage: 'Environment variables set when launching an instance.',
	},
	preLaunchHookTitle: {
		id: 'app.settings.default-instance-options.pre-launch-hook.title',
		defaultMessage: 'Pre-launch hook',
	},
	preLaunchHookPlaceholder: {
		id: 'app.settings.default-instance-options.pre-launch-hook.placeholder',
		defaultMessage: 'Enter pre-launch command...',
	},
	preLaunchHookDescription: {
		id: 'app.settings.default-instance-options.pre-launch-hook.description',
		defaultMessage: 'Runs before the instance starts.',
	},
	wrapperHookTitle: {
		id: 'app.settings.default-instance-options.wrapper-hook.title',
		defaultMessage: 'Wrapper hook',
	},
	wrapperHookPlaceholder: {
		id: 'app.settings.default-instance-options.wrapper-hook.placeholder',
		defaultMessage: 'Enter wrapper command...',
	},
	wrapperHookDescription: {
		id: 'app.settings.default-instance-options.wrapper-hook.description',
		defaultMessage: 'Command used to wrap the Minecraft launch process.',
	},
	postExitHookTitle: {
		id: 'app.settings.default-instance-options.post-exit-hook.title',
		defaultMessage: 'Post-exit hook',
	},
	postExitHookPlaceholder: {
		id: 'app.settings.default-instance-options.post-exit-hook.placeholder',
		defaultMessage: 'Enter post-exit command...',
	},
	postExitHookDescription: {
		id: 'app.settings.default-instance-options.post-exit-hook.description',
		defaultMessage: 'Runs after the game closes.',
	},
	hookVariablesDescription: {
		id: 'instance.settings.tabs.hooks.variables.description',
		defaultMessage:
			'Hooks run in the working directory of the instance, with the following variables:',
	},
	instanceNameDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-name.description',
		defaultMessage: '$INST_NAME: The name of the instance',
	},
	instanceIdDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-id.description',
		defaultMessage: "$INST_ID: The name of the instance's folder",
	},
	instanceDirDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-dir.description',
		defaultMessage: "$INST_DIR: The absolute path to the instance's folder",
	},
	instanceMcDirDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-mc-dir.description',
		defaultMessage: '$INST_MC_DIR: An alias for $INST_DIR',
	},
	instanceJavaDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-java.description',
		defaultMessage: '$INST_JAVA: The absolute path to the java binary',
	},
	instanceJavaArgsDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-java-args.description',
		defaultMessage: '$INST_JAVA_ARGS: The JVM Arguments provided to the game',
	},
})

const fetchSettings = await get()
fetchSettings.launchArgs = fetchSettings.extra_launch_args.join(' ')
fetchSettings.envVars = serializeEnvVars(fetchSettings.custom_env_vars)

const settings = ref(fetchSettings)

const { maxMemory, snapPoints } = (await useMemorySlider().catch(handleError)) as unknown as {
	maxMemory: number
	snapPoints: number[]
}

watch(
	settings,
	async () => {
		const setSettings = JSON.parse(JSON.stringify(settings.value))

		setSettings.extra_launch_args = setSettings.launchArgs.trim().split(/\s+/).filter(Boolean)
		setSettings.custom_env_vars = parseEnvVars(setSettings.envVars)
		delete setSettings.launchArgs
		delete setSettings.envVars

		if (!setSettings.custom_dir) {
			setSettings.custom_dir = null
		}

		await set(setSettings).catch(handleError)
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

		<SettingsGroup :label="formatMessage(messages.windowSectionTitle)">
			<SettingsRow
				control-id="fullscreen"
				:title="formatMessage(messages.fullscreenTitle)"
				:description="formatMessage(messages.fullscreenDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Toggle
						:id="controlId"
						v-model="settings.force_fullscreen"
						:aria-labelledby="labelledBy"
					/>
				</template>
			</SettingsRow>

			<SettingsTwoCol>
				<template #left>
					<SettingsStack
						control-id="width"
						:title="formatMessage(messages.widthTitle)"
						:description="formatMessage(messages.widthDescription)"
					>
						<template #default="{ labelledBy, controlId }">
							<Input
								:id="controlId"
								v-model="settings.game_resolution[0]"
								:disabled="settings.force_fullscreen"
								:aria-labelledby="labelledBy"
								autocomplete="off"
								type="number"
								:placeholder="formatMessage(messages.widthPlaceholder)"
								wrapper-class="w-full"
							/>
						</template>
					</SettingsStack>
				</template>
				<template #right>
					<SettingsStack
						control-id="height"
						:title="formatMessage(messages.heightTitle)"
						:description="formatMessage(messages.heightDescription)"
					>
						<template #default="{ labelledBy, controlId }">
							<Input
								:id="controlId"
								v-model="settings.game_resolution[1]"
								:disabled="settings.force_fullscreen"
								:aria-labelledby="labelledBy"
								autocomplete="off"
								type="number"
								:placeholder="formatMessage(messages.heightPlaceholder)"
								wrapper-class="w-full"
							/>
						</template>
					</SettingsStack>
				</template>
			</SettingsTwoCol>
		</SettingsGroup>

		<SettingsGroup :label="formatMessage(messages.javaAndMemorySectionTitle)">
			<SettingsStack
				control-id="max-memory"
				:title="formatMessage(messages.memoryAllocationTitle)"
				:description="formatMessage(messages.memoryAllocationDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Slider
						:id="controlId"
						v-model="settings.memory.maximum"
						:aria-labelledby="labelledBy"
						:min="512"
						:max="maxMemory"
						:step="64"
						:snap-points="snapPoints"
						:snap-range="512"
						unit="MB"
					/>
				</template>
			</SettingsStack>

			<SettingsStack
				control-id="java-args"
				:title="formatMessage(messages.javaArgumentsTitle)"
				:description="formatMessage(messages.javaArgumentsDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Input
						:id="controlId"
						v-model="settings.launchArgs"
						:aria-labelledby="labelledBy"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.javaArgumentsPlaceholder)"
						wrapper-class="w-full"
					/>
				</template>
			</SettingsStack>

			<SettingsStack
				control-id="env-vars"
				:title="formatMessage(messages.environmentVariablesTitle)"
				:description="formatMessage(messages.environmentVariablesDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Input
						:id="controlId"
						v-model="settings.envVars"
						:aria-labelledby="labelledBy"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.environmentVariablesPlaceholder)"
						wrapper-class="w-full"
					/>
				</template>
			</SettingsStack>
		</SettingsGroup>

		<SettingsGroup :label="formatMessage(messages.launchHooksSectionTitle)">
			<SettingsStack
				control-id="pre-launch"
				:title="formatMessage(messages.preLaunchHookTitle)"
				:description="formatMessage(messages.preLaunchHookDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Input
						:id="controlId"
						v-model="settings.hooks.pre_launch"
						:aria-labelledby="labelledBy"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.preLaunchHookPlaceholder)"
						wrapper-class="w-full"
					/>
				</template>
			</SettingsStack>

			<SettingsStack
				control-id="wrapper"
				:title="formatMessage(messages.wrapperHookTitle)"
				:description="formatMessage(messages.wrapperHookDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Input
						:id="controlId"
						v-model="settings.hooks.wrapper"
						:aria-labelledby="labelledBy"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.wrapperHookPlaceholder)"
						wrapper-class="w-full"
					/>
				</template>
			</SettingsStack>

			<SettingsStack
				control-id="post-exit"
				:title="formatMessage(messages.postExitHookTitle)"
				:description="formatMessage(messages.postExitHookDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Input
						:id="controlId"
						v-model="settings.hooks.post_exit"
						:aria-labelledby="labelledBy"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.postExitHookPlaceholder)"
						wrapper-class="w-full"
					/>
				</template>
			</SettingsStack>

			<div class="px-3 py-2.5 text-sm leading-snug text-secondary">
				{{ formatMessage(messages.hookVariablesDescription) }}
				<ul class="m-0 mt-2 list-disc pl-5">
					<li>{{ formatMessage(messages.instanceNameDescription) }}</li>
					<li>{{ formatMessage(messages.instanceIdDescription) }}</li>
					<li>{{ formatMessage(messages.instanceDirDescription) }}</li>
					<li>{{ formatMessage(messages.instanceMcDirDescription) }}</li>
					<li>{{ formatMessage(messages.instanceJavaDescription) }}</li>
					<li>{{ formatMessage(messages.instanceJavaArgsDescription) }}</li>
				</ul>
			</div>
		</SettingsGroup>
	</div>
</template>
