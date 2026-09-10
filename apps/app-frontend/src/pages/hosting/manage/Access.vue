<script setup lang="ts">
import { injectLumenClient, injectLumenServerContext, ServersManageAccessPage } from '@lumen/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectLumenClient()
const { serverId } = injectLumenServerContext()
const queryClient = useQueryClient()

try {
	await Promise.all([
		queryClient.ensureQueryData({
			queryKey: ['servers', 'users', 'v1', serverId],
			queryFn: () => client.archon.server_users_v1.list(serverId),
			staleTime: 30_000,
		}),
		queryClient.ensureQueryData({
			queryKey: ['servers', 'v1', 'detail', serverId],
			queryFn: () => client.archon.servers_v1.get(serverId),
			staleTime: 30_000,
		}),
	])
} catch {
	// Let mounted layouts' useQuery surface errors; do not fail route setup.
}

function userProfileLink(username: string) {
	return `/user/${encodeURIComponent(username)}`
}
</script>

<template>
	<ServersManageAccessPage :user-profile-link="userProfileLink" />
</template>
