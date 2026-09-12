import type { GameInstance } from '@/helpers/types'

export type LibraryInstanceSelection = {
	instanceId: string
	groupId: string
}

export const getLibraryInstanceSelectionKey = ({
	instanceId,
	groupId,
}: LibraryInstanceSelection) => JSON.stringify([groupId, instanceId])

export type ActiveInstanceGroupDrag = {
	instances: LibraryInstanceSelection[]
	primaryInstanceId: string
	fromGroup: string | null
}

export type InstanceGroup = {
	id: string
	key: string
	instances: GameInstance[]
}

export type InstanceCard = {
	instance: GameInstance
	playing: boolean
	play: (event: MouseEvent | null, context: string) => Promise<void>
	stop: (event: MouseEvent | null, context: string) => Promise<void>
	addContent: () => Promise<void>
	seeInstance: () => Promise<void>
	openFolder: () => Promise<void>
}
