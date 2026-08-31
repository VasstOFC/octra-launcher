<template>
	<NewModal ref="modal" :header="formatMessage(messages.addOffline)" max-width="400px">
		<div class="flex flex-col gap-3">
			<Input
				v-model="offlineName"
				maxlength="16"
				:placeholder="formatMessage(messages.offlineNickPlaceholder)"
				:disabled="loading"
				@keyup.enter="submit"
			/>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" :disabled="loading" @click="hide">
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="loading || !canLoginOffline"
					@click="submit"
				>
					<SpinnerIcon v-if="loading" class="animate-spin" />
					<PlusIcon v-else />
					{{ formatMessage(messages.addOffline) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { PlusIcon, SpinnerIcon } from '@modrinth/assets'
import { Button, commonMessages, defineMessages, Input, NewModal, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { handleSevereError } from '@/composables/use-error.js'
import { login_offline as loginOfflineFlow } from '@/helpers/auth.js'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	added: [account: unknown]
}>()

const messages = defineMessages({
	addOffline: {
		id: 'minecraft-account.add-offline',
		defaultMessage: 'Add offline account',
	},
	offlineNickPlaceholder: {
		id: 'minecraft-account.offline-nick',
		defaultMessage: 'Offline nickname',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const offlineName = ref('')
const loading = ref(false)

const canLoginOffline = computed(() => {
	const name = offlineName.value.trim()
	return name.length >= 1 && name.length <= 16 && !/\s/.test(name) && /^[A-Za-z0-9_]+$/.test(name)
})

function show() {
	offlineName.value = ''
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

async function submit() {
	if (!canLoginOffline.value || loading.value) return
	loading.value = true

	try {
		const loggedIn = await loginOfflineFlow(offlineName.value.trim())
		if (!loggedIn) return

		emit('added', loggedIn)
		hide()
	} catch (error) {
		handleSevereError(error)
	} finally {
		loading.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>
