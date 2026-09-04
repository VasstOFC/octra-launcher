<script lang="ts"></script>

<script setup lang="ts">
import { RightArrowIcon, SpinnerIcon } from '@modrinth/assets'
import { type Component, type ComponentPublicInstance, computed, nextTick, ref, watch } from 'vue'

import { type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { useScrollIndicator } from '../../composables/scroll-indicator'
import { truncatedTooltip } from '../../utils/truncate'
import NewModal from './NewModal.vue'
export interface Tab {
	name: MessageDescriptor
	category?: MessageDescriptor
	icon: Component
	content?: Component
	href?: string
	badge?: MessageDescriptor
	shown?: boolean
}

const { formatMessage } = useVIntl()

const props = withDefaults(
	defineProps<{
		tabs: Tab[]
		header?: string
		maxWidth?: string
		width?: string
		closable?: boolean
		onHide?: () => void
		onShow?: () => void
		beforeHide?: () => boolean
		beforeTabChange?: (fromIndex: number, toIndex: number) => boolean
		floatingActionBarShown?: boolean
		variant?: 'default' | 'emberslot'
		navLayout?: 'side' | 'top'
	}>(),
	{
		header: undefined,
		maxWidth: undefined,
		width: undefined,
		closable: true,
		onHide: undefined,
		onShow: undefined,
		beforeHide: undefined,
		beforeTabChange: undefined,
		floatingActionBarShown: false,
		variant: 'default',
		navLayout: 'side',
	},
)

const isEmberslot = computed(() => props.variant === 'emberslot')
const isTopNav = computed(() => props.navLayout === 'top')

const selectedTabClasses = computed(() => {
	if (isTopNav.value && isEmberslot.value) {
		return 'emberslot-tab emberslot-tab--active emberslot-tab--top text-brand'
	}
	if (isEmberslot.value) {
		return 'emberslot-tab emberslot-tab--active text-brand'
	}
	return 'bg-button-bgSelected text-button-textSelected'
})

const idleTabClasses = computed(() =>
	isEmberslot.value
		? 'emberslot-tab bg-transparent text-button-text hover:bg-button-bg hover:text-contrast'
		: 'bg-transparent text-button-text hover:bg-button-bg hover:text-contrast',
)

const visibleTabs = computed(() => props.tabs.filter((tab) => tab.shown !== false))

const selectedTab = ref(0)
const tabLabelRefs = ref<Record<number, HTMLElement | null>>({})

function setTabLabelRef(index: number, element: Element | ComponentPublicInstance | null) {
	tabLabelRefs.value[index] = element instanceof HTMLElement ? element : null
}

function tabLabelTooltip(index: number, label: string) {
	return truncatedTooltip(tabLabelRefs.value[index], label)
}

const scrollContainer = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, checkScrollState, forceCheck } =
	useScrollIndicator(scrollContainer)

const sidebarScrollContainer = ref<HTMLElement | null>(null)
const {
	showTopFade: showSidebarTopFade,
	showBottomFade: showSidebarBottomFade,
	checkScrollState: checkSidebarScrollState,
} = useScrollIndicator(sidebarScrollContainer)

const modal = ref<InstanceType<typeof NewModal> | null>(null)

type CategoryGroup = {
	key: string
	category?: MessageDescriptor
	indices: number[]
}

const categoryGroups = computed<CategoryGroup[]>(() => {
	const groups: CategoryGroup[] = []
	for (let index = 0; index < visibleTabs.value.length; index++) {
		const tab = visibleTabs.value[index]
		const key = tab.category?.id ?? `__uncategorized-${index}`
		const last = groups[groups.length - 1]
		if (last && last.key === key) {
			last.indices.push(index)
		} else {
			groups.push({
				key,
				category: tab.category,
				indices: [index],
			})
		}
	}
	return groups
})

const activeCategoryKey = computed(() => {
	const tab = visibleTabs.value[selectedTab.value]
	if (!tab) return categoryGroups.value[0]?.key ?? ''
	return tab.category?.id ?? `__uncategorized-${selectedTab.value}`
})

const tabsInActiveCategory = computed(() => {
	const group = categoryGroups.value.find((item) => item.key === activeCategoryKey.value)
	return group?.indices ?? []
})

function setTab(index: number) {
	if (index === selectedTab.value) return
	if (props.beforeTabChange?.(selectedTab.value, index) === false) return
	selectedTab.value = index
	nextTick(() => forceCheck())
}

function selectCategory(group: CategoryGroup) {
	const first = group.indices[0]
	if (first == null) return
	setTab(first)
}

function show(event?: MouseEvent) {
	modal.value?.show(event)
}

function hide(): boolean {
	return modal.value?.hide() ?? false
}

function startsCategory(index: number) {
	const category = visibleTabs.value[index]?.category
	return !!category && category.id !== visibleTabs.value[index - 1]?.category?.id
}

watch(visibleTabs, (tabs) => {
	if (selectedTab.value >= tabs.length) {
		selectedTab.value = Math.max(0, tabs.length - 1)
	}
})

defineExpose({ show, hide, selectedTab, setTab })
</script>
<template>
	<NewModal
		ref="modal"
		:header="header"
		:max-width="maxWidth"
		:width="width"
		:closable="closable"
		:on-hide="onHide"
		:on-show="onShow"
		:before-hide="beforeHide"
		no-padding
		noblur
	>
		<template v-if="$slots.title" #title>
			<slot name="title" />
		</template>

		<!-- Top navbar layout -->
		<div
			v-if="isTopNav"
			class="flex flex-col"
			:class="isEmberslot ? 'emberslot-panel gap-3 p-4 pb-3' : 'p-6 pb-3'"
		>
			<nav
				class="emberslot-topnav flex shrink-0 flex-col gap-2 border-0 border-b border-solid border-divider pb-3"
			>
				<div class="flex flex-wrap gap-1.5">
					<button
						v-for="group in categoryGroups"
						:key="group.key"
						type="button"
						class="emberslot-category rounded-lg border border-solid px-3 py-2 text-xs font-bold uppercase tracking-wide cursor-pointer transition-colors"
						:class="
							activeCategoryKey === group.key
								? 'emberslot-category--active border-brand bg-brand-highlight text-brand'
								: 'border-transparent bg-transparent text-secondary hover:bg-button-bg hover:text-contrast'
						"
						@click="selectCategory(group)"
					>
						{{ group.category ? formatMessage(group.category) : '—' }}
					</button>
				</div>
				<div class="flex flex-wrap gap-1">
					<template v-for="index in tabsInActiveCategory" :key="index">
						<component
							:is="visibleTabs[index].href ? 'a' : 'button'"
							:href="visibleTabs[index].href ?? undefined"
							:target="visibleTabs[index].href ? '_blank' : undefined"
							:rel="visibleTabs[index].href ? 'noopener noreferrer' : undefined"
							:class="`relative flex min-w-0 shrink-0 gap-2 items-center text-left rounded-lg px-3 py-2 border-none font-semibold cursor-pointer active:scale-[0.97] transition-all no-underline ${!visibleTabs[index].href && selectedTab === index ? selectedTabClasses : idleTabClasses}`"
							@click="!visibleTabs[index].href && setTab(index)"
						>
							<component :is="visibleTabs[index].icon" class="h-4 w-4 flex-shrink-0" />
							<span
								:ref="(element) => setTabLabelRef(index, element)"
								v-tooltip="tabLabelTooltip(index, formatMessage(visibleTabs[index].name))"
								class="min-w-0 truncate"
							>
								{{ formatMessage(visibleTabs[index].name) }}
							</span>
							<span
								v-if="visibleTabs[index].badge"
								class="shrink-0 rounded-full bg-brand-highlight px-1.5 py-0.5 text-xs font-bold text-brand-green"
							>
								{{ formatMessage(visibleTabs[index].badge) }}
							</span>
							<RightArrowIcon v-if="visibleTabs[index].href" class="ml-auto size-4 shrink-0" />
						</component>
					</template>
				</div>
			</nav>

			<div
				class="relative min-h-[min(52vh,480px)]"
				:class="{ 'emberslot-content rounded-lg': isEmberslot }"
			>
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-4"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-4"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-4 bg-gradient-to-b to-transparent"
						:class="isEmberslot ? 'from-surface-1' : 'from-bg-raised'"
					/>
				</Transition>

				<div
					ref="scrollContainer"
					class="absolute inset-0 overflow-y-auto"
					:class="[
						isEmberslot ? 'px-4 pt-3' : '',
						floatingActionBarShown ? 'pb-24' : isEmberslot ? 'pb-3' : 'pb-2',
					]"
					@scroll="checkScrollState"
				>
					<Suspense>
						<component
							:is="visibleTabs[selectedTab]?.content"
							v-if="visibleTabs[selectedTab]?.content"
						/>
						<template #fallback>
							<div class="flex h-full min-h-[12rem] items-center justify-center">
								<SpinnerIcon class="size-8 animate-spin text-secondary" />
							</div>
						</template>
					</Suspense>
				</div>

				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-16"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-16"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-16 bg-gradient-to-t to-transparent"
						:class="isEmberslot ? 'from-surface-1' : 'from-bg-raised'"
					/>
				</Transition>

				<div class="pointer-events-none absolute bottom-3 left-0 right-0 z-20">
					<div class="pointer-events-auto">
						<slot name="floating-action-bar" />
					</div>
				</div>
			</div>

			<slot name="footer" />
		</div>

		<!-- Side sidebar layout (default) -->
		<div
			v-else
			class="grid grid-cols-[minmax(12.5rem,18rem)_minmax(0,1fr)] p-6 pb-3 pr-0"
			:class="{ 'emberslot-panel': isEmberslot }"
		>
			<div
				class="flex min-w-0 max-h-[min(65vh,600px)] flex-col border-0 border-r-[1px] border-solid border-divider pr-4"
				:class="{ 'emberslot-sidebar -ml-6 -mt-6 mb-0 rounded-l-xl pl-6 pt-6': isEmberslot }"
			>
				<div class="relative min-h-0 flex-1">
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-4"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-4"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showSidebarTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-4 bg-gradient-to-b to-transparent"
							:class="isEmberslot ? 'from-surface-2' : 'from-bg-raised'"
						/>
					</Transition>

					<div
						ref="sidebarScrollContainer"
						class="flex h-full flex-col gap-1 overflow-y-auto"
						@scroll="checkSidebarScrollState"
					>
						<template v-for="(tab, index) in visibleTabs" :key="index">
							<div
								v-if="startsCategory(index) && tab.category"
								class="shrink-0 truncate px-4 pb-1 pt-2 text-xs font-bold uppercase tracking-wide text-secondary"
							>
								{{ formatMessage(tab.category) }}
							</div>
							<component
								:is="tab.href ? 'a' : 'button'"
								:href="tab.href ?? undefined"
								:target="tab.href ? '_blank' : undefined"
								:rel="tab.href ? 'noopener noreferrer' : undefined"
								:class="`relative flex min-w-0 shrink-0 gap-2 items-center text-left rounded-xl px-4 py-2 border-none font-semibold cursor-pointer active:scale-[0.97] transition-all no-underline ${!tab.href && selectedTab === index ? selectedTabClasses : idleTabClasses}`"
								@click="!tab.href && setTab(index)"
							>
								<component :is="tab.icon" class="h-4 w-4 flex-shrink-0" />
								<span
									:ref="(element) => setTabLabelRef(index, element)"
									v-tooltip="tabLabelTooltip(index, formatMessage(tab.name))"
									class="min-w-0 flex-1 truncate"
								>
									{{ formatMessage(tab.name) }}
								</span>
								<span
									v-if="tab.badge"
									class="shrink-0 rounded-full bg-brand-highlight px-1.5 py-0.5 text-xs font-bold text-brand-green"
								>
									{{ formatMessage(tab.badge) }}
								</span>
								<RightArrowIcon v-if="tab.href" class="ml-auto size-4 shrink-0" />
							</component>
						</template>
					</div>

					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-16"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-16"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showSidebarBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-16 bg-gradient-to-t to-transparent"
							:class="isEmberslot ? 'from-surface-2' : 'from-bg-raised'"
						/>
					</Transition>
				</div>

				<slot name="footer" />
			</div>
			<div
				class="relative min-h-[min(65vh,600px)]"
				:class="{ 'emberslot-content -mr-0 rounded-r-xl': isEmberslot }"
			>
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-4"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-4"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-4 bg-gradient-to-b to-transparent"
						:class="isEmberslot ? 'from-surface-1' : 'from-bg-raised'"
					/>
				</Transition>

				<div
					ref="scrollContainer"
					class="absolute inset-0 overflow-y-auto px-6"
					:class="floatingActionBarShown ? 'pb-24' : 'pb-6'"
					@scroll="checkScrollState"
				>
					<Suspense>
						<component
							:is="visibleTabs[selectedTab]?.content"
							v-if="visibleTabs[selectedTab]?.content"
						/>
						<template #fallback>
							<div class="flex h-full min-h-[12rem] items-center justify-center">
								<SpinnerIcon class="size-8 animate-spin text-secondary" />
							</div>
						</template>
					</Suspense>
				</div>

				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-16"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-16"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-16 bg-gradient-to-t to-transparent"
						:class="isEmberslot ? 'from-surface-1' : 'from-bg-raised'"
					/>
				</Transition>

				<div class="pointer-events-none absolute bottom-3 left-6 right-6 z-20">
					<div class="pointer-events-auto">
						<slot name="floating-action-bar" />
					</div>
				</div>
			</div>
		</div>
	</NewModal>
</template>

<style lang="scss" scoped>
.emberslot-sidebar {
	background: var(--surface-2);
}

.emberslot-content {
	background: var(--surface-1);
}

.emberslot-tab {
	transition:
		background-color var(--shell-motion, 0.28s cubic-bezier(0.32, 0.72, 0, 1)),
		color var(--shell-motion, 0.28s cubic-bezier(0.32, 0.72, 0, 1)),
		box-shadow var(--shell-motion, 0.28s cubic-bezier(0.32, 0.72, 0, 1)),
		transform var(--shell-motion, 0.28s cubic-bezier(0.32, 0.72, 0, 1));

	@media (prefers-reduced-motion: reduce) {
		transition: none;
	}
}

.emberslot-tab--active {
	background: var(--surface-3);
	box-shadow: inset 0 0 0 1px var(--surface-5);
	transform: translateX(2px);

	&::before {
		content: '';
		position: absolute;
		left: 0.35rem;
		top: 50%;
		transform: translateY(-50%);
		height: 1rem;
		width: 0.2rem;
		border-radius: 999px;
		background: var(--color-brand);
	}

	@media (prefers-reduced-motion: reduce) {
		transform: none;
	}
}

.emberslot-tab--top.emberslot-tab--active {
	transform: none;
	box-shadow: none;

	&::before {
		left: 50%;
		top: auto;
		bottom: 0;
		transform: translateX(-50%);
		height: 2px;
		width: 1.25rem;
		border-radius: 1px;
	}
}
</style>
