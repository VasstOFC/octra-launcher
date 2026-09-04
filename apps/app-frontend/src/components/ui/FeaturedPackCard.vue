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
		defaultMessage: 'Staff pick',
	},
	title: {
		id: 'app.featured-pack.title',
		defaultMessage: 'Cobblemon Vasst',
	},
	blurb: {
		id: 'app.featured-pack.blurb',
		defaultMessage: 'Catch, battle, explore — install once and jump straight in.',
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
					: formatMessage(messages.install) + ' ' + formatMessage(messages.title)
			}}
		</Button>
		<aside v-else class="featured-promo" :aria-label="formatMessage(messages.kicker)">
			<div class="featured-promo__accent" aria-hidden="true" />
			<div class="featured-promo__body min-w-0">
				<p class="featured-promo__kicker m-0">
					{{ formatMessage(messages.kicker) }}
				</p>
				<div class="featured-promo__copy min-w-0">
					<p class="featured-promo__title m-0 truncate">
						{{ formatMessage(messages.title) }}
					</p>
					<p class="featured-promo__blurb m-0 truncate">
						{{ formatMessage(messages.blurb) }}
					</p>
				</div>
			</div>
			<Button
				type="colored"
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
		</aside>
	</div>
</template>

<style scoped lang="scss">
.featured-promo {
	align-items: center;
	background:
		linear-gradient(
			100deg,
			color-mix(in srgb, var(--color-brand) 14%, transparent) 0%,
			transparent 42%
		),
		color-mix(in srgb, var(--color-brand) 4%, var(--surface-2));
	border: 1px solid color-mix(in srgb, var(--color-brand) 26%, var(--surface-5));
	border-radius: var(--radius-md);
	display: flex;
	gap: 0.875rem;
	justify-content: space-between;
	overflow: hidden;
	padding: 0.65rem 0.85rem 0.65rem 0;
	position: relative;
}

.featured-promo__accent {
	align-self: stretch;
	background: linear-gradient(
		180deg,
		var(--color-brand) 0%,
		color-mix(in srgb, var(--color-brand) 45%, transparent) 100%
	);
	border-radius: 0 2px 2px 0;
	flex-shrink: 0;
	margin-right: 0.15rem;
	width: 3px;
}

.featured-promo__body {
	display: flex;
	flex: 1;
	flex-direction: column;
	gap: 0.2rem;
	min-width: 0;
	padding-left: 0.15rem;
}

.featured-promo__kicker {
	color: var(--color-brand);
	font-size: 0.6875rem;
	font-weight: 700;
	letter-spacing: 0.08em;
	text-transform: uppercase;
}

.featured-promo__copy {
	display: flex;
	flex-direction: column;
	gap: 0.1rem;
	min-width: 0;
}

.featured-promo__title {
	color: var(--color-contrast);
	font-size: 0.9375rem;
	font-weight: 600;
	line-height: 1.25;
}

.featured-promo__blurb {
	color: var(--color-secondary);
	font-size: 0.8125rem;
	line-height: 1.3;
}
</style>
