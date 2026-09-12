import { useEventListener } from '@vueuse/core'
import { computed, ref } from 'vue'

import {
	getLibraryInstanceSelectionKey,
	type LibraryInstanceSelection,
} from '../library-types'

export function useLibrarySelection() {
	const selectedLibraryInstances = ref(new Map<string, LibraryInstanceSelection>())
	const isLibraryInstanceSelectionActive = computed(
		() => selectedLibraryInstances.value.size > 0,
	)

	const clearLibraryInstanceSelection = () => {
		selectedLibraryInstances.value = new Map()
	}

	const setSelectedLibraryInstances = (selections: Iterable<LibraryInstanceSelection>) => {
		selectedLibraryInstances.value = new Map(
			[...selections].map((selection) => [getLibraryInstanceSelectionKey(selection), selection]),
		)
	}

	const toggleLibraryInstanceSelection = (selection: LibraryInstanceSelection) => {
		const selectedInstances = new Map(selectedLibraryInstances.value)
		const selectionKey = getLibraryInstanceSelectionKey(selection)

		if (selectedInstances.has(selectionKey)) {
			selectedInstances.delete(selectionKey)
		} else {
			selectedInstances.set(selectionKey, selection)
		}

		selectedLibraryInstances.value = selectedInstances
	}

	useEventListener(window, 'keydown', (event) => {
		if (
			event.key === 'Escape' &&
			!event.defaultPrevented &&
			isLibraryInstanceSelectionActive.value
		) {
			clearLibraryInstanceSelection()
		}
	})

	return {
		selectedLibraryInstances,
		isLibraryInstanceSelectionActive,
		clearLibraryInstanceSelection,
		setSelectedLibraryInstances,
		toggleLibraryInstanceSelection,
	}
}
