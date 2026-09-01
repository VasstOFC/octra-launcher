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
			class="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-4 p-4"
		>
			<div class="min-w-0 flex flex-col gap-1">
				<div class="text-xs font-bold uppercase tracking-wide text-brand">
					{{ formatMessage(messages.kicker) }}
				</div>
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{ pack.title || formatMessage(messages.title) }}
				</h3>
				<p class="m-0 text-sm leading-5 text-secondary">
					{{ pack.blurb || formatMessage(messages.blurb) }}
				</p>
			</div>
			<Button
				type="colored"
				color="brand"
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
