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
		class="nav-button relative text-primary flex items-center transition-all bg-transparent hover:bg-button-bg hover:text-contrast"
	>
		<slot />
		<span v-if="expanded && label" class="min-w-0 truncate text-[13px] font-medium leading-none">
			{{ label }}
		</span>
	</RouterLink>
	<button
		v-else
		v-bind="$attrs"
		class="nav-button button-animation relative border-none text-primary cursor-pointer flex items-center transition-all bg-transparent hover:bg-button-bg hover:text-contrast"
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
	width: 3rem;
	height: 3rem;
	border-radius: 9999px;
	justify-content: center;
	font-size: 1.5rem;

	&.expanded {
		width: 100%;
		height: 2.75rem;
		border-radius: 0.75rem;
		justify-content: flex-start;
		gap: 0.75rem;
		padding: 0 0.75rem;
		font-size: 1.25rem;
	}
}

.router-link-active,
.subpage-active {
	svg {
		filter: drop-shadow(0 0 0.5rem black);
	}
}

.router-link-active {
	@apply text-[--color-button-text-selected] bg-[--color-button-bg-selected];
}

.router-link-active.expanded,
button.expanded.router-link-active {
	color: var(--color-brand);
	background: var(--color-brand-highlight);
	box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-brand) 28%, transparent);
}

.subpage-active {
	@apply text-contrast bg-button-bg;
}

.router-link-active:not(.expanded)::before {
	content: '';
	position: absolute;
	left: 0.15rem;
	top: 50%;
	transform: translateY(-50%);
	height: 1.35rem;
	width: 0.15rem;
	border-radius: 999px;
	background: var(--color-brand);
}
</style>
