<script setup lang="ts">
import { PlayIcon, PlusIcon } from '@modrinth/assets'
import { ContextMenu, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed, inject, onActivated, ref } from 'vue'

import FeaturedPackCard from '@/components/ui/FeaturedPackCard.vue'
import HomeHeroBand from '@/components/ui/home/HomeHeroBand.vue'
import LibrarySection from '@/components/ui/library/index.vue'
import WelcomeScreen from '@/components/ui/WelcomeScreen.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { toError } from '@/helpers/errors'
import { list } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'
import { injectOnboardingChecklist } from '@/providers/onboarding-checklist'

defineOptions({
	name: 'LibraryPage',
})

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const { hasCreatedInstance, isReady } = injectOnboardingChecklist()
const showCreationModal = inject<() => void>('showCreationModal')
const pageOptions = ref<InstanceType<typeof ContextMenu>>()

const messages = defineMessages({
	home: {
		id: 'app.navigation.home',
		defaultMessage: 'Home',
	},
	newInstance: {
		id: 'app.library.context-menu.create-instance',
		defaultMessage: 'New instance',
	},
	libraryActionsLabel: {
		id: 'app.library.actions.label',
		defaultMessage: 'Library actions',
	},
})

const homeBreadcrumb = useRootBreadcrumb({
	slot: 'root',
	id: 'home',
	label: formatMessage(messages.home),
	to: '/',
	visual: { type: 'icon', component: PlayIcon },
})
onActivated(homeBreadcrumb.reset)

const instances = ref<GameInstance[]>([])
let latestInstanceFetch = 0

const continueInstance = computed(() => {
	if (instances.value.length === 0) return null
	return instances.value
		.slice()
		.sort((a, b) => dayjs(b.last_played ?? b.created).diff(dayjs(a.last_played ?? a.created)))[0]
})

const hasFeaturedPack = computed(() =>
	instances.value.some((instance) => instance.name.toLowerCase() === 'cobblemon vasst'),
)

async function fetchInstances() {
	const fetchId = ++latestInstanceFetch
	try {
		const nextInstances = await list()
		if (fetchId === latestInstanceFetch) {
			instances.value = nextInstances
		}
	} catch (error: unknown) {
		if (fetchId === latestInstanceFetch) {
			handleError(toError(error))
		}
	}
}

if (hasCreatedInstance.value) {
	await fetchInstances()
}

useAppEvent('instance', fetchInstances)
useAppEvent('instance_groups_changed', fetchInstances)

function openPageContextMenu(event: MouseEvent) {
	if (
		!(event.target instanceof HTMLElement) ||
		!event.target.hasAttribute('data-library-page-background')
	) {
		return
	}

	event.preventDefault()
	event.stopPropagation()
	pageOptions.value?.open(event, [
		{
			id: 'new_instance',
			label: formatMessage(messages.newInstance),
			icon: PlusIcon,
			action: () => showCreationModal?.(),
		},
	])
}
</script>

<template>
	<WelcomeScreen v-if="isReady && !hasCreatedInstance" />
	<div
		v-else-if="isReady"
		data-library-page-background
		class="flex flex-col gap-4 p-6"
		@contextmenu="openPageContextMenu"
	>
		<HomeHeroBand :instance="continueInstance" />
		<FeaturedPackCard :installed="hasFeaturedPack" @installed="fetchInstances" />
		<LibrarySection :instances="instances" />
		<ContextMenu ref="pageOptions" :label="formatMessage(messages.libraryActionsLabel)" />
	</div>
</template>
