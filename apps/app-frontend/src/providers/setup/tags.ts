import type { AbstractWebNotificationManager } from '@modrinth/ui'
import { provideTags } from '@modrinth/ui'
import type { Ref } from 'vue'
import { ref, watch } from 'vue'

import { get_game_versions, get_loaders } from '@/helpers/tags'

export function setupTagsProvider(
	notificationManager: AbstractWebNotificationManager,
	stateInitialized?: Ref<boolean>,
) {
	const { handleError } = notificationManager

	const gameVersions = ref([])
	const loaders = ref([])

	async function refreshTags() {
		get_game_versions()
			.then((v) => {
				gameVersions.value = v
			})
			.catch(handleError)
		get_loaders()
			.then((v) => {
				loaders.value = v
			})
			.catch(handleError)
	}

	if (stateInitialized) {
		watch(
			stateInitialized,
			(value) => {
				if (value) {
					void refreshTags()
				}
			},
			{ immediate: true },
		)
	} else {
		void refreshTags()
	}

	provideTags({ gameVersions, loaders })
}
