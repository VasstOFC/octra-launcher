<script setup lang="ts">
import { DownloadIcon, SpinnerIcon } from '@modrinth/assets'
import { Button, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { onMounted, ref } from 'vue'

import { toError } from '@/helpers/errors'
import {
	type FeaturedPackInfo,
	install_featured_pack,
	install_get_featured_pack,
} from '@/helpers/install'

const props = withDefaults(
	defineProps<{
		compact?: boolean
		installed?: boolean
	}>(),
	{
		compact: false,
		installed: false,
	},
)

const emit = defineEmits<{
	installed: []
}>()

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const installing = ref(false)
const pack = ref<FeaturedPackInfo | null>(null)

const messages = defineMessages({
	kicker: {
		id: 'app.featured-pack.kicker',
		defaultMessage: 'Featured pack',
	},
	title: {
		id: 'app.featured-pack.title',
		defaultMessage: 'Cobblemon Vasst',
	},
	blurb: {
		id: 'app.featured-pack.blurb',
		defaultMessage: 'Author pack — a couple of clicks and you can play.',
	},
	install: {
		id: 'app.featured-pack.install',
		defaultMessage: 'Install',
	},
	installing: {
		id: 'app.featured-pack.installing',
		defaultMessage: 'Installing…',
	},
})

onMounted(async () => {
	try {
		pack.value = await install_get_featured_pack()
	} catch {
		pack.value = {
			enabled: true,
			title: formatMessage(messages.title),
			blurb: formatMessage(messages.blurb),
		}
	}
})

async function install() {
	if (installing.value) return
	installing.value = true
	try {
		await install_featured_pack()
		emit('installed')
	} catch (error) {
		handleError(toError(error))
	} finally {
		installing.value = false
	}
}
</script>

<template>
	<div v-if="pack?.enabled && !props.installed" :class="compact ? 'w-72' : undefined">
		<Button
			v-if="compact"
			type="colored"
			color="brand"
			size="lg"
			class="!shadow-none w-full"
			:loading="installing"
			@click="install"
		>
			<SpinnerIcon v-if="installing" class="animate-spin" />
			<DownloadIcon v-else />
			{{
				installing
					? formatMessage(messages.installing)
					: formatMessage(messages.install) + ' ' + (pack.title || formatMessage(messages.title))
			}}
		</Button>
		<div
			v-else
			class="featured-strip flex items-center justify-between gap-3 rounded-lg bg-surface-3 px-3 py-2"
		>
			<div class="min-w-0 flex items-baseline gap-2">
				<span class="shrink-0 text-xs font-semibold uppercase tracking-wide text-brand">
					{{ formatMessage(messages.kicker) }}
				</span>
				<span class="truncate text-sm font-medium text-contrast">
					{{ pack.title || formatMessage(messages.title) }}
				</span>
				<span class="hidden truncate text-sm text-secondary sm:inline">
					{{ pack.blurb || formatMessage(messages.blurb) }}
				</span>
			</div>
			<Button
				type="quiet"
				color="brand"
				size="sm"
				class="!shadow-none shrink-0"
				:loading="installing"
				@click="install"
			>
				<SpinnerIcon v-if="installing" class="animate-spin" />
				<DownloadIcon v-else />
				{{ installing ? formatMessage(messages.installing) : formatMessage(messages.install) }}
			</Button>
		</div>
	</div>
</template>
