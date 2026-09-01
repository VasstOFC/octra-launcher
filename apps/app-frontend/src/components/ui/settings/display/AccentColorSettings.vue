<script setup lang="ts">
import { RadioButtonCheckedIcon, RadioButtonIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import {
	ACCENT_PRESETS,
	type AccentPresetId,
	normalizeHexColor,
} from '@/helpers/accent-colors.ts'

const props = defineProps<{
	preset: AccentPresetId
	customHex: string
}>()

const emit = defineEmits<{
	'update:preset': [value: AccentPresetId]
	'update:customHex': [value: string]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'app.settings.accent.title',
		defaultMessage: 'Accent color',
	},
	description: {
		id: 'app.settings.accent.description',
		defaultMessage: 'Change the accent color used for buttons, highlights, and glow effects.',
	},
	custom: {
		id: 'app.settings.accent.custom',
		defaultMessage: 'Custom',
	},
	customColorLabel: {
		id: 'app.settings.accent.custom-color-label',
		defaultMessage: 'Custom accent color',
	},
	presetOctra: {
		id: 'app.settings.accent.preset.octra',
		defaultMessage: 'Octra',
	},
	presetCobalt: {
		id: 'app.settings.accent.preset.cobalt',
		defaultMessage: 'Cobalt',
	},
	presetEmber: {
		id: 'app.settings.accent.preset.ember',
		defaultMessage: 'Ember',
	},
	presetMint: {
		id: 'app.settings.accent.preset.mint',
		defaultMessage: 'Mint',
	},
	presetRose: {
		id: 'app.settings.accent.preset.rose',
		defaultMessage: 'Rose',
	},
	presetAurora: {
		id: 'app.settings.accent.preset.aurora',
		defaultMessage: 'Aurora',
	},
})

const presetLabelMessages = {
	octra: messages.presetOctra,
	cobalt: messages.presetCobalt,
	ember: messages.presetEmber,
	mint: messages.presetMint,
	rose: messages.presetRose,
	aurora: messages.presetAurora,
} as const

function formatPresetLabel(preset: Exclude<AccentPresetId, 'custom'>) {
	return formatMessage(presetLabelMessages[preset])
}

const normalizedCustomHex = computed({
	get: () => normalizeHexColor(props.customHex),
	set: (value: string) => emit('update:customHex', normalizeHexColor(value)),
})

function selectPreset(preset: AccentPresetId) {
	emit('update:preset', preset)
}

function presetPreviewStyle(preset: Exclude<AccentPresetId, 'custom'>) {
	const definition = ACCENT_PRESETS.find((entry) => entry.id === preset)
	if (!definition) {
		return {}
	}

	return {
		background: `linear-gradient(135deg, ${definition.light} 0%, ${definition.dark} 100%)`,
		boxShadow: `0 0 0 1px ${definition.light}55, 0 0 18px ${definition.dark}66`,
	}
}
</script>

<template>
	<section class="border-0 border-b border-solid border-divider pb-8">
		<div class="flex flex-col gap-1">
			<h2 class="m-0 text-xl font-semibold text-contrast">
				{{ formatMessage(messages.title) }}
			</h2>
			<p class="m-0 text-secondary">
				{{ formatMessage(messages.description) }}
			</p>
		</div>

		<div class="accent-options mt-4" role="group" :aria-label="formatMessage(messages.title)">
			<button
				v-for="presetOption in ACCENT_PRESETS"
				:key="presetOption.id"
				type="button"
				class="accent-option button-base"
				:class="{ selected: preset === presetOption.id }"
				:aria-pressed="preset === presetOption.id"
				@click="selectPreset(presetOption.id)"
			>
				<span
					class="accent-swatch"
					:style="presetPreviewStyle(presetOption.id)"
					aria-hidden="true"
				/>
				<span class="accent-label">
					<RadioButtonCheckedIcon
						v-if="preset === presetOption.id"
						class="radio shrink-0 text-brand"
						aria-hidden="true"
					/>
					<RadioButtonIcon v-else class="radio shrink-0 text-secondary" aria-hidden="true" />
					{{ formatPresetLabel(presetOption.id) }}
				</span>
			</button>

			<button
				type="button"
				class="accent-option button-base"
				:class="{ selected: preset === 'custom' }"
				:aria-pressed="preset === 'custom'"
				@click="selectPreset('custom')"
			>
				<span
					class="accent-swatch custom-swatch"
					:style="{
						background: normalizedCustomHex,
						boxShadow: `0 0 0 1px ${normalizedCustomHex}88, 0 0 18px ${normalizedCustomHex}66`,
					}"
					aria-hidden="true"
				/>
				<span class="accent-label">
					<RadioButtonCheckedIcon
						v-if="preset === 'custom'"
						class="radio shrink-0 text-brand"
						aria-hidden="true"
					/>
					<RadioButtonIcon v-else class="radio shrink-0 text-secondary" aria-hidden="true" />
					{{ formatMessage(messages.custom) }}
				</span>
			</button>
		</div>

		<div v-if="preset === 'custom'" class="custom-accent mt-4 flex flex-wrap items-end gap-4">
			<label class="flex flex-col gap-2">
				<span class="text-sm font-medium text-primary">
					{{ formatMessage(messages.customColorLabel) }}
				</span>
				<input
					v-model="normalizedCustomHex"
					type="color"
					class="accent-color-input"
					:aria-label="formatMessage(messages.customColorLabel)"
				/>
			</label>
			<label class="flex min-w-[7rem] flex-col gap-2">
				<span class="text-sm font-medium text-primary">HEX</span>
				<input
					:value="normalizedCustomHex"
					type="text"
					class="accent-hex-input"
					maxlength="7"
					spellcheck="false"
					@change="normalizedCustomHex = ($event.target as HTMLInputElement).value"
				/>
			</label>
		</div>
	</section>
</template>

<style scoped lang="scss">
.accent-options {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(8.5rem, 1fr));
	gap: var(--gap-lg);
}

.accent-option {
	display: flex;
	flex-direction: column;
	align-items: stretch;
	gap: 0.75rem;
	padding: 0.75rem;
	border: 1px solid var(--color-button-border);
	border-radius: var(--radius-lg);
	background: var(--color-button-bg);
	cursor: pointer;
	text-align: left;

	&.selected {
		border-color: var(--color-brand);
		box-shadow: 0 0 0 1px var(--color-brand-highlight);
	}
}

.accent-swatch {
	display: block;
	height: 3.5rem;
	border-radius: var(--radius-md);
}

.custom-swatch {
	background-image:
		linear-gradient(45deg, rgba(255, 255, 255, 0.12) 25%, transparent 25%),
		linear-gradient(-45deg, rgba(255, 255, 255, 0.12) 25%, transparent 25%),
		linear-gradient(45deg, transparent 75%, rgba(255, 255, 255, 0.12) 75%),
		linear-gradient(-45deg, transparent 75%, rgba(255, 255, 255, 0.12) 75%);
	background-size: 12px 12px;
	background-position:
		0 0,
		0 6px,
		6px -6px,
		-6px 0;
}

.accent-label {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	font-size: 0.875rem;
	font-weight: 500;
	color: var(--color-contrast);
}

.accent-color-input {
	width: 4.5rem;
	height: 2.75rem;
	padding: 0.125rem;
	border: 1px solid var(--color-button-border);
	border-radius: var(--radius-md);
	background: var(--color-button-bg);
	cursor: pointer;
}

.accent-hex-input {
	height: 2.75rem;
	padding: 0 0.75rem;
	border: 1px solid var(--color-button-border);
	border-radius: var(--radius-md);
	background: var(--color-button-bg);
	color: var(--color-contrast);
	font-family: var(--font-mono, monospace);
}
</style>
