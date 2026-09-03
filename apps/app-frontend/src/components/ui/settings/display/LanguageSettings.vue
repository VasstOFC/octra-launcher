<template>
	<div :class="{ 'language-settings--embedded': embedded }">
		<SharedLanguageSettings
			ref="languageSettings"
			product="app"
			:persist-locale="persistLocale"
		/>
	</div>
</template>

<script setup lang="ts">
import { LanguageSettings as SharedLanguageSettings } from '@modrinth/ui'
import { computed, inject, onBeforeUnmount, onMounted, ref } from 'vue'

import { get, set } from '@/helpers/settings.ts'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'

const props = withDefaults(
	defineProps<{
		embedded?: boolean
	}>(),
	{
		embedded: false,
	},
)

const settingsModal = inject(appSettingsModalContextKey, null)
const languageSettings = ref<InstanceType<typeof SharedLanguageSettings> | null>(null)

onMounted(() => {
	if (props.embedded) return

	settingsModal?.registerUnsavedChangesController({
		hasChanges: () => languageSettings.value?.hasChanges ?? false,
		getOriginal: () => languageSettings.value?.originalState ?? {},
		getModified: () => languageSettings.value?.modifiedState ?? {},
		isSaving: () => languageSettings.value?.saving ?? false,
		reset: () => languageSettings.value?.reset(),
		save: () => languageSettings.value?.save(),
	})
})

onBeforeUnmount(() => {
	if (props.embedded) return
	settingsModal?.registerUnsavedChangesController(null)
})

async function persistLocale(locale: string): Promise<void> {
	const settings = await get()
	if (settings.locale === locale) return
	await set({ ...settings, locale })
}

defineExpose({
	hasChanges: computed(() => languageSettings.value?.hasChanges ?? false),
	originalState: computed(() => languageSettings.value?.originalState ?? {}),
	modifiedState: computed(() => languageSettings.value?.modifiedState ?? {}),
	saving: computed(() => languageSettings.value?.saving ?? false),
	reset: () => languageSettings.value?.reset(),
	save: () => languageSettings.value?.save(),
})
</script>

<style scoped lang="scss">
.language-settings--embedded {
	:deep(> div > h2) {
		display: none;
	}
}
</style>
