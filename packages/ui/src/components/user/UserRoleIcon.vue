<template>
	<LumenIcon
		v-if="role === 'admin'"
		v-tooltip="formatMessage(messages.LumenTeamLabel)"
		role="img"
		:aria-label="formatMessage(messages.LumenTeamLabel)"
		class="size-4 shrink-0 !text-green"
	/>
	<ScaleIcon
		v-else-if="role === 'moderator'"
		v-tooltip="formatMessage(messages.moderatorLabel)"
		role="img"
		:aria-label="formatMessage(messages.moderatorLabel)"
		class="size-4 shrink-0 !text-orange"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@lumen/api-client'
import { LumenIcon, ScaleIcon } from '@lumen/assets'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

defineProps<{
	role?: Labrinth.Users.v2.Role | null
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	LumenTeamLabel: {
		id: 'user.role.Lumen-team',
		defaultMessage: 'Lumen Team',
	},
	moderatorLabel: {
		id: 'user.role.moderator',
		defaultMessage: 'Content Moderator',
	},
})
</script>
