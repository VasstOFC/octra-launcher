<script setup lang="ts">
import { injectLumenServerContext, ServersManageOverviewPage } from '@lumen/ui'

const { server } = injectLumenServerContext()
const flags = useFeatureFlags()

useHead({
	title: computed(() => `Overview - ${server.value?.name ?? 'Server'} - Lumen`),
})
</script>

<template>
	<ServersManageOverviewPage :show-advanced-debug-info="flags.advancedDebugInfo" />
</template>
