<script setup lang="ts">
import { PlusIcon, SearchIcon, SquarePlusIcon } from '@lumen/assets'
import { Button, defineMessages, Input, useVIntl } from '@lumen/ui'
import { computed, inject } from 'vue'

import FilterMenu from '@/components/ui/library/library-toolbar/filter-menu.vue'
import NewGroupModal from '@/components/ui/library/library-toolbar/new-group-modal.vue'
import SortMenu from '@/components/ui/library/library-toolbar/sort-menu.vue'
import { useLibrary } from '@/components/ui/library/use-library'

const { searchInput, setSearchInput, selectedLibraryInstances, openNewGroupModal } =
	useLibrary()
const showCreationModal = inject<() => void>('showCreationModal')
const { formatMessage } = useVIntl()
const messages = defineMessages({
	search: { id: 'app.library.search.placeholder', defaultMessage: 'Search' },
	newGroup: { id: 'app.library.group.new', defaultMessage: 'New group' },
	newInstance: { id: 'app.library.instance.new', defaultMessage: 'New instance' },
})
const selectedInstanceIds = computed(
	() =>
		new Set([...selectedLibraryInstances.value.values()].map((selection) => selection.instanceId)),
)

function openNewGroup() {
	openNewGroupModal(selectedInstanceIds.value)
}
</script>

<template>
	<div class="flex flex-wrap items-center gap-2">
		<Input
			:model-value="searchInput"
			:icon="SearchIcon"
			type="text"
			:placeholder="formatMessage(messages.search)"
			clearable
			wrapper-class="min-w-[12rem] flex-1"
			@update:model-value="(value: string) => setSearchInput(value)"
		/>
		<SortMenu />
		<div class="mx-1 h-6 w-px bg-surface-5 max-sm:hidden" />
		<FilterMenu />
		<div class="mx-1 h-6 w-px bg-surface-5 max-sm:hidden" />
		<Button @click="openNewGroup">
			<SquarePlusIcon />
			{{ formatMessage(messages.newGroup) }}
		</Button>
		<Button type="colored" color="brand" @click="showCreationModal?.()">
			<PlusIcon />
			{{ formatMessage(messages.newInstance) }}
		</Button>
	</div>
	<NewGroupModal />
</template>
