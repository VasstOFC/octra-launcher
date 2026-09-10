import type { AbstractLumenClient } from '@lumen/api-client'
import type { AbstractPopupNotificationManager, AbstractWebNotificationManager } from '@lumen/ui'
import type { Ref } from 'vue'

import type { InstanceIconConfig } from '@/helpers/types'

import type { AppEvents } from './app-events'
import { setupOnboardingChecklistProvider } from './onboarding-checklist'
import { setupCreationModal } from './setup/creation-modal'
import { setupFileDropProvider } from './setup/file-drop'
import { setupFilePickerProvider } from './setup/file-picker'
import { setupImageViewerEditorProvider } from './setup/image-viewer-editor'
import { setupInstanceImportProvider } from './setup/instance-import'
import { setupTagsProvider } from './setup/tags'
import { setupUserCountryProvider } from './setup/user-country'

export function setupProviders(
	client: AbstractLumenClient,
	notificationManager: AbstractWebNotificationManager,
	_popupNotificationManager: AbstractPopupNotificationManager,
	appEvents: AppEvents,
	stateInitialized: Ref<boolean>,
	getGeneratedIconConfig?: (iconPath: string) => InstanceIconConfig | null,
) {
	setupUserCountryProvider(client)
	setupTagsProvider(notificationManager, stateInitialized)
	setupFileDropProvider()
	setupFilePickerProvider()
	setupImageViewerEditorProvider()
	setupInstanceImportProvider(notificationManager)
	const onboardingChecklist = setupOnboardingChecklistProvider(appEvents)

	return {
		...setupCreationModal(notificationManager, getGeneratedIconConfig),
		onboardingChecklist,
	}
}
