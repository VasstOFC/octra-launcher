import { computed, ref } from 'vue'

import { getPlayerHeadUrl } from '@/helpers/rendering/batch-skin-renderer'
import { get_available_skins, type Skin } from '@/helpers/skins'

const STEVE_HEAD = 'https://launcher-files.modrinth.com/assets/steve_head.png'

const equippedSkin = ref<Skin | null>(null)
const headUrlCache = ref(new Map<string, string>())

export function useMinecraftAccountAvatar() {
	const equippedSkinAvatarUrl = computed(() => {
		if (!equippedSkin.value?.texture_key) {
			return null
		}

		return headUrlCache.value.get(equippedSkin.value.texture_key) ?? null
	})

	async function refreshEquippedSkinAvatar() {
		try {
			const skins = await get_available_skins()
			equippedSkin.value = skins.find((skin) => skin.is_equipped) ?? null

			if (!equippedSkin.value) {
				return
			}

			try {
				const headUrl = await getPlayerHeadUrl(equippedSkin.value)
				headUrlCache.value = new Map(headUrlCache.value).set(
					equippedSkin.value.texture_key,
					headUrl,
				)
			} catch (error) {
				console.warn('Failed to get head render for equipped skin:', error)
			}
		} catch {
			equippedSkin.value = null
		}
	}

	async function setEquippedSkinAvatar(skin: Skin) {
		equippedSkin.value = skin

		try {
			const headUrl = await getPlayerHeadUrl(skin)
			headUrlCache.value = new Map(headUrlCache.value).set(skin.texture_key, headUrl)
		} catch (error) {
			console.warn('Failed to get head render for equipped skin:', error)
		}
	}

	function getAccountAvatarUrl(profileId: string | undefined, isSelectedAccount: boolean) {
		if (isSelectedAccount) {
			const equippedUrl = equippedSkinAvatarUrl.value
			if (equippedUrl) {
				return equippedUrl
			}
		}

		if (profileId) {
			return `https://mc-heads.net/avatar/${profileId}/128`
		}

		return STEVE_HEAD
	}

	return {
		equippedSkin,
		equippedSkinAvatarUrl,
		refreshEquippedSkinAvatar,
		setEquippedSkinAvatar,
		getAccountAvatarUrl,
		STEVE_HEAD,
	}
}
