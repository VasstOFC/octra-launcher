<script setup lang="ts">
import { ServerStackIcon } from '@lumen/assets'
import { ServersManagePageIndex } from '@lumen/ui'

import { useRootBreadcrumb } from '@/providers/breadcrumbs'

useRootBreadcrumb({
	slot: 'root',
	id: 'servers',
	label: 'Hosting',
	to: '/hosting/manage/',
	visual: { type: 'icon', component: ServerStackIcon },
})
</script>

<template>
	<ServersManagePageIndex :stripe-publishable-key="''" :products="[]" />
</template>
