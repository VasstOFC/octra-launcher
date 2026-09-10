<template>
	<div
		v-if="
			project.project_type !== 'plugin' ||
			project.loaders.some((x) => !tags.loaderData.allPluginLoaders.includes(x))
		"
		class="Lumen-app-section contents"
	>
		<div class="flex flex-col items-center">
			<ButtonLink
				type="colored"
				color="brand"
				class="!min-h-10 w-fit no-underline"
				:href="`Lumen://mod/${project.slug}`"
				@click="installWithApp"
			>
				<LumenIcon aria-hidden="true" />
				<span class="min-w-0 text-center">
					{{ formatMessage(messages.installWithLumenApp) }}
				</span>
			</ButtonLink>
			<Accordion ref="getLumenAppAccordion">
				<nuxt-link class="mt-2 flex justify-center text-brand-blue hover:underline" to="/app">
					{{ formatMessage(messages.dontHaveLumenApp) }}
				</nuxt-link>
			</Accordion>
		</div>

		<div class="flex items-center gap-4">
			<div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
			<span class="flex-shrink-0 text-sm font-medium text-secondary">
				{{ formatMessage(messages.downloadManually) }}
			</span>
			<div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@lumen/api-client'
import { LumenIcon } from '@lumen/assets'
import { ButtonLink, defineMessages, useVIntl } from '@lumen/ui'
import type { DisplayProjectType } from '@lumen/utils'
import { ref } from 'vue'

import Accordion from '~/components/ui/Accordion.vue'

defineOptions({
	name: 'InstallWithLumenApp',
})

type DownloadModalProject = Omit<Labrinth.Projects.v2.Project, 'project_type'> & {
	project_type: DisplayProjectType
	actualProjectType: Labrinth.Projects.v2.ProjectType
}

defineProps<{
	project: DownloadModalProject
}>()

const { formatMessage } = useVIntl()
const tags = useGeneratedState()
const getLumenAppAccordion = ref<InstanceType<typeof Accordion> | null>(null)

const messages = defineMessages({
	installWithLumenApp: {
		id: 'project.download.install-with-app',
		defaultMessage: 'Install with Lumen App',
	},
	dontHaveLumenApp: {
		id: 'project.download.no-app',
		defaultMessage: "Don't have Lumen App?",
	},
	downloadManually: {
		id: 'project.download.manually',
		defaultMessage: 'Download manually',
	},
})

function installWithApp() {
	setTimeout(() => {
		getLumenAppAccordion.value?.open()
	}, 1500)
}
</script>

<style lang="scss" scoped>
@media (hover: none) and (max-width: 767px) {
	.Lumen-app-section {
		display: none;
	}
}
</style>
