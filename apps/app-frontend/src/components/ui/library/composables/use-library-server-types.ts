import type { Labrinth } from '@lumen/api-client'
import { ref } from 'vue'

import { get_project_v3_many } from '@/helpers/cache.js'
import type { GameInstance } from '@/helpers/types'

export type LibraryInstanceType = 'custom' | 'modpack' | 'server'

const SERVER_TYPE_CACHE_TTL_MS = 60_000

const serverTypeCache = new Map<string, { isServer: boolean; cachedAt: number }>()
let serverTypeRequest: Promise<void> | null = null

export function useLibraryServerTypes() {
	const serverProjectIds = ref(new Set<string>())

	const refreshServerTypes = async (linkedInstances: GameInstance[]) => {
		const projectIds = [
			...new Set(
				linkedInstances.flatMap((instance) =>
					instance.link?.project_id ? [instance.link.project_id] : [],
				),
			),
		]

		if (projectIds.length === 0) {
			serverProjectIds.value = new Set()
			return
		}

		while (true) {
			const now = Date.now()
			const staleIds = projectIds.filter((projectId) => {
				const cached = serverTypeCache.get(projectId)
				return !cached || now - cached.cachedAt >= SERVER_TYPE_CACHE_TTL_MS
			})

			if (staleIds.length === 0) break

			serverTypeRequest ??= (async () => {
				try {
					const projects = (await get_project_v3_many(
						staleIds,
						'must_revalidate',
					)) as Array<Labrinth.Projects.v3.Project | null>
					const cachedAt = Date.now()
					for (const project of projects) {
						if (!project) continue
						serverTypeCache.set(project.id, {
							isServer: project.minecraft_server != null,
							cachedAt,
						})
					}
				} catch {
					// Keep previously cached values when the fetch fails.
				} finally {
					serverTypeRequest = null
				}
			})()

			await serverTypeRequest
		}

		serverProjectIds.value = new Set(
			projectIds.filter((projectId) => serverTypeCache.get(projectId)?.isServer),
		)
	}

	const getInstanceType = (instance: GameInstance): LibraryInstanceType => {
		if (!instance.link) return 'custom'
		if (serverProjectIds.value.has(instance.link.project_id ?? '')) return 'server'
		return 'modpack'
	}

	return { serverProjectIds, refreshServerTypes, getInstanceType }
}
