import { useDebounceFn } from '@vueuse/core'
import { computed, ref } from 'vue'

const LIBRARY_SEARCH_DEBOUNCE_MS = 200

export function useLibrarySearch() {
	const searchInput = ref('')
	const search = ref('')

	const applySearchDebounced = useDebounceFn((value: string) => {
		search.value = value
	}, LIBRARY_SEARCH_DEBOUNCE_MS)

	const setSearchInput = (value: string) => {
		searchInput.value = value
		if (value === '') {
			applySearchDebounced.cancel()
			search.value = ''
			return
		}
		applySearchDebounced(value)
	}

	const isSearching = computed(() => search.value.length > 0)

	return {
		searchInput,
		search,
		setSearchInput,
		isSearching,
	}
}
