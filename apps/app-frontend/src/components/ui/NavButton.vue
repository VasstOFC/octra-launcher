<template>
	<RouterLink
		v-if="typeof to === 'string'"
		:to="to"
		v-bind="$attrs"
		:active-class="isSubpage ? '' : undefined"
		:class="{
			'router-link-active': isPrimary && isPrimary(route),
			'subpage-active': isSubpage && isSubpage(route),
			disabled: disabled,
			expanded: expanded,
		}"
		class="nav-button relative text-primary flex items-center bg-transparent hover:bg-button-bg hover:text-contrast"
	>
		<slot />
		<span v-if="expanded && label" class="min-w-0 truncate text-[13px] font-medium leading-none">
			{{ label }}
		</span>
	</RouterLink>
	<button
		v-else
		v-bind="$attrs"
		class="nav-button button-animation relative border-none text-primary cursor-pointer flex items-center bg-transparent hover:bg-button-bg hover:text-contrast"
		:class="{ disabled: disabled, expanded: expanded }"
		:disabled="disabled"
		@click="to"
	>
		<slot />
		<span v-if="expanded && label" class="min-w-0 truncate text-[13px] font-medium leading-none">
			{{ label }}
		</span>
	</button>
</template>

<script setup lang="ts">
import type { RouteLocationNormalizedLoaded } from 'vue-router'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

type RouteFunction = (route: RouteLocationNormalizedLoaded) => boolean

withDefaults(
	defineProps<{
		to: (() => void) | string
		isPrimary?: RouteFunction
		isSubpage?: RouteFunction
		highlightOverride?: boolean
		disabled?: boolean
		expanded?: boolean
		label?: string
	}>(),
	{
		disabled: false,
		expanded: false,
		label: undefined,
	},
)

defineOptions({
	inheritAttrs: false,
})
</script>

<style lang="scss" scoped>
.nav-button {
	width: 2.5rem;
	height: 2.5rem;
	border-radius: 9999px;
	justify-content: center;
	font-size: 1.5rem;
	transition: all 0.2s ease;

	&:hover:not(.router-link-active):not(.disabled) {
		transform: scale(1.08);
		background: color-mix(in srgb, var(--color-brand) 10%, transparent);
		box-shadow: 0 0 15px color-mix(in srgb, var(--color-brand) 12%, transparent);
	}

	&:active:not(.disabled) {
		transform: scale(0.95);
	}

	&.expanded {
		width: 100%;
		height: 2.5rem;
		border-radius: var(--radius-md);
		justify-content: flex-start;
		gap: 0.75rem;
		padding: 0 0.75rem;
		font-size: 1.25rem;
	}

	@media (prefers-reduced-motion: reduce) {
		transition: none;
	}
}

.router-link-active,
.subpage-active {
	svg {
		filter: none;
	}
}

.router-link-active {
	color: var(--color-brand) !important;
	background: color-mix(in srgb, var(--color-brand) 14%, transparent) !important;
	box-shadow: 0 0 15px color-mix(in srgb, var(--color-brand) 18%, transparent);
	animation: nav-glow 2s ease-in-out infinite;
}

@keyframes nav-glow {
	0%, 100% {
		box-shadow: 0 0 12px color-mix(in srgb, var(--color-brand) 14%, transparent);
	}
	50% {
		box-shadow: 0 0 20px color-mix(in srgb, var(--color-brand) 24%, transparent);
	}
}

.router-link-active.expanded,
button.expanded.router-link-active {
	color: var(--color-brand) !important;
	background: color-mix(in srgb, var(--color-brand) 14%, transparent) !important;
	box-shadow: 0 0 15px color-mix(in srgb, var(--color-brand) 18%, transparent);
}

.subpage-active {
	color: var(--color-contrast) !important;
	background: var(--surface-3) !important;
}

.router-link-active::before {
	content: '';
	position: absolute;
	left: 0;
	top: 50%;
	transform: translateY(-50%);
	height: 1.25rem;
	width: 2px;
	border-radius: 1px;
	background: var(--color-brand);
	box-shadow: 0 0 8px color-mix(in srgb, var(--color-brand) 60%, transparent);
	opacity: 1;
	transition: opacity 0.2s ease;
}

.router-link-active.expanded::before {
	left: 0;
	height: 1.15rem;
}

@media (prefers-reduced-motion: reduce) {
	.router-link-active::before {
		transition: none;
	}

	.nav-button:hover:not(.router-link-active):not(.disabled) {
		transform: none;
	}

	.router-link-active {
		animation: none;
	}
}
</style>
