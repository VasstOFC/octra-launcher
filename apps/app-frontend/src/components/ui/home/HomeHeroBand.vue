<script setup lang="ts">
import { defineMessages, useVIntl } from '@modrinth/ui'

import OctraWordmark from '@/components/brand/OctraWordmark.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	tagline: {
		id: 'app.home.hero.tagline',
		defaultMessage: 'Your worlds. Your crew. One launcher.',
	},
})
</script>

<template>
	<section class="home-hero relative isolate overflow-hidden" aria-label="Octra">
		<div class="home-hero__glow" aria-hidden="true" />
		<div class="relative z-[1] min-w-0 px-6 py-5">
			<OctraWordmark class="h-7 w-auto text-contrast" />
			<p class="home-hero__tagline mt-2 mb-0 font-minecraft text-sm text-secondary">
				{{ formatMessage(messages.tagline) }}
			</p>
		</div>
	</section>
</template>

<style scoped lang="scss">
.home-hero {
	margin: -1.5rem -1.5rem 0;
	border-bottom: 1px solid var(--color-divider);
	background:
		linear-gradient(
			105deg,
			color-mix(in srgb, var(--color-brand) 18%, transparent) 0%,
			transparent 48%
		),
		linear-gradient(
			180deg,
			color-mix(in srgb, var(--color-brand) 10%, var(--color-raised-bg)) 0%,
			var(--color-bg) 100%
		);
}

.home-hero__glow {
	pointer-events: none;
	position: absolute;
	inset: auto -10% -60% 40%;
	height: 140%;
	background: radial-gradient(
		ellipse at center,
		color-mix(in srgb, var(--color-brand) 22%, transparent) 0%,
		transparent 70%
	);
	opacity: 0.55;
}

.home-hero__tagline {
	letter-spacing: 0.02em;
	line-height: 1.35;
}
</style>
