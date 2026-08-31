<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="544px" no-padding>
		<div class="grid grid-cols-[1fr_auto] gap-2.5 h-[154px] px-7 pt-4 pb-1 pr-9">
			<div class="flex flex-col gap-2.5 items-start justify-center h-min mt-5">
				<div class="font-semibold text-xl text-contrast">
					{{ formatMessage(messages.descriptionHeader) }}
				</div>
				<div class="text-secondary leading-6">
					{{ formatMessage(messages.description) }}
				</div>
			</div>
			<div class="relative h-full w-[96px] overflow-hidden mx-3">
				<div class="absolute top-0 left-0 z-0 w-full flex grow-0 flex-col items-end p-0">
					<img :src="steveImage" alt="" class="self-stretch" />
				</div>
				<div
					class="absolute left-0 bottom-0 z-10 order-1 h-6 w-[120px] shrink-0 grow-0 bg-[linear-gradient(180deg,rgba(39,41,46,0)_0%,#27292E_80%,#27292E_100%)]"
				></div>
			</div>
		</div>

		<div class="flex flex-col gap-4 px-6 pb-6">
			<Button type="colored" color="brand" :disabled="loading" @click="signIn">
				<SpinnerIcon v-if="loadingSignIn" class="animate-spin" />
				<svg
					v-else
					width="20"
					height="20"
					viewBox="0 0 20 20"
					fill="none"
					xmlns="http://www.w3.org/2000/svg"
				>
					<rect width="9.25" height="9.25" fill="black" fill-opacity="0.9" />
					<rect x="10.75" width="9.25" height="9.25" fill="black" fill-opacity="0.9" />
					<rect y="10.75" width="9.25" height="9.25" fill="black" fill-opacity="0.9" />
					<rect x="10.75" y="10.75" width="9.25" height="9.25" fill="black" fill-opacity="0.9" />
				</svg>
				{{ formatMessage(messages.signIn) }}
			</Button>
			<div class="flex flex-col gap-2">
				<div class="text-sm font-medium text-contrast">
					{{ formatMessage(messages.orPlayOffline) }}
				</div>
				<Input
					v-model="offlineName"
					maxlength="16"
					:placeholder="formatMessage(messages.offlineNickPlaceholder)"
					:disabled="loading"
					@keyup.enter="signInOffline"
				/>
				<Button :disabled="loading || !canLoginOffline" @click="signInOffline">
					<SpinnerIcon v-if="loadingOffline" class="animate-spin" />
					{{ formatMessage(messages.playOffline) }}
				</Button>
			</div>
			<p class="m-0 text-center text-sm text-secondary">
				{{ formatMessage(messages.dontHaveAccount) }}
				<a
					class="text-blue font-medium hover:underline"
					href="https://www.minecraft.net/en-us/store/minecraft-java-bedrock-edition-pc"
				>
					{{ formatMessage(messages.getMinecraft) }}
				</a>
			</p>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { Button, defineMessages, Input, NewModal, useVIntl } from '@modrinth/ui'
import { computed, inject, type Ref, ref } from 'vue'

import steveImage from '@/assets/steve-look-up-left.webp'
import type AccountsCard from '@/components/ui/AccountsCard.vue'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import {
	login as loginFlow,
	login_offline as loginOfflineFlow,
	set_default_user,
} from '@/helpers/auth.js'

const { formatMessage } = useVIntl()
const accountsCard = inject('accountsCard') as Ref<InstanceType<typeof AccountsCard> | null>

const messages = defineMessages({
	header: {
		id: 'minecraft-required.header',
		defaultMessage: 'Log in to Octra',
	},
	descriptionHeader: {
		id: 'minecraft-required.description-header',
		defaultMessage: 'Log in to Octra',
	},
	description: {
		id: 'minecraft-required.description',
		defaultMessage:
			'Sign in with Microsoft, or play offline with a nickname. Your skin is uploaded so it shows in singleplayer and on SkinsRestorer servers.',
	},
	signIn: {
		id: 'minecraft-required.sign-in',
		defaultMessage: 'Microsoft account',
	},
	orPlayOffline: {
		id: 'minecraft-required.or-play-offline',
		defaultMessage: 'Or play offline',
	},
	offlineNickPlaceholder: {
		id: 'minecraft-required.offline-nick',
		defaultMessage: 'Offline nickname',
	},
	playOffline: {
		id: 'minecraft-required.play-offline',
		defaultMessage: 'Play offline',
	},
	dontHaveAccount: {
		id: 'minecraft-required.dont-have-account',
		defaultMessage: 'Don’t have a Microsoft account?',
	},
	getMinecraft: {
		id: 'minecraft-required.get-minecraft',
		defaultMessage: 'Get Minecraft',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const loadingSignIn = ref(false)
const loadingOffline = ref(false)
const offlineName = ref('')

const loading = computed(() => loadingSignIn.value || loadingOffline.value)
const canLoginOffline = computed(() => {
	const name = offlineName.value.trim()
	return name.length >= 1 && name.length <= 16 && !/\s/.test(name) && /^[A-Za-z0-9_]+$/.test(name)
})

function show() {
	modal.value?.show()
}

async function signIn() {
	loadingSignIn.value = true

	try {
		const loggedIn = await loginFlow()
		if (!loggedIn) return

		await set_default_user(loggedIn.profile.id)
		await accountsCard.value?.refreshValues()
		await trackEvent('AccountLogIn', { source: 'MinecraftRequiredModal' })
		modal.value?.hide()
	} catch (error) {
		handleSevereError(error)
	} finally {
		loadingSignIn.value = false
	}
}

async function signInOffline() {
	if (!canLoginOffline.value) return
	loadingOffline.value = true

	try {
		const loggedIn = await loginOfflineFlow(offlineName.value.trim())
		if (!loggedIn) return

		await set_default_user(loggedIn.profile.id)
		await accountsCard.value?.refreshValues()
		await trackEvent('AccountLogIn', { source: 'MinecraftRequiredModalOffline' })
		modal.value?.hide()
	} catch (error) {
		handleSevereError(error)
	} finally {
		loadingOffline.value = false
	}
}

defineExpose({
	show,
})
</script>
