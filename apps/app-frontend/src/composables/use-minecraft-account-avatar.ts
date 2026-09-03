import { computed, ref } from 'vue'

import { get_default_user, isOfflineAccount, users } from '@/helpers/auth'
import { getPlayerHeadUrl } from '@/helpers/rendering/batch-skin-renderer'
import {
	get_available_skins,
	get_profile_equipped_skin_texture,
	skin_from_equipped_texture,
	type Skin,
} from '@/helpers/skins'

const STEVE_HEAD = 'https://launcher-files.modrinth.com/assets/steve_head.png'
const MC_HEADS_AVATAR = (profileId: string) => `https://mc-heads.net/avatar/${profileId}/128`

const equippedSkin = ref<Skin | null>(null)
const headUrlCache = ref(new Map<string, string>())
const profileHeadUrlCache = ref(new Map<string, string>())
const offlineProfileIds = ref(new Set<string>())

type AccountLike = {
	profile?: { id?: string }
	is_offline?: boolean
	refresh_token?: string
}

function cacheProfileHead(profileId: string, headUrl: string) {
	profileHeadUrlCache.value = new Map(profileHeadUrlCache.value).set(profileId, headUrl)
}

function cacheTextureHead(textureKey: string, headUrl: string) {
	headUrlCache.value = new Map(headUrlCache.value).set(textureKey, headUrl)
}

function markOfflineProfiles(accounts: AccountLike[]) {
	const next = new Set<string>()
	for (const account of accounts) {
		const profileId = account.profile?.id
		if (profileId && isOfflineAccount(account)) {
			next.add(profileId)
		}
	}
	offlineProfileIds.value = next
}

async function resolveProfileHead(profileId: string, isSelected: boolean) {
	if (isSelected && equippedSkin.value) {
		const equippedUrl =
			headUrlCache.value.get(equippedSkin.value.texture_key) ??
			(await getPlayerHeadUrl(equippedSkin.value).catch(() => null))
		if (equippedUrl) {
			cacheTextureHead(equippedSkin.value.texture_key, equippedUrl)
			cacheProfileHead(profileId, equippedUrl)
			return
		}
	}

	try {
		const equipped = await get_profile_equipped_skin_texture(profileId)
		if (!equipped) {
			return
		}

		const skin = skin_from_equipped_texture(equipped)
		const cachedTextureUrl = headUrlCache.value.get(skin.texture_key)
		const headUrl = cachedTextureUrl ?? (await getPlayerHeadUrl(skin))
		cacheTextureHead(skin.texture_key, headUrl)
		cacheProfileHead(profileId, headUrl)
	} catch (error) {
		console.warn(`Failed to load equipped skin avatar for ${profileId}:`, error)
	}
}

export function useMinecraftAccountAvatar() {
	const equippedSkinAvatarUrl = computed(() => {
		if (!equippedSkin.value?.texture_key) {
			return null
		}

		return headUrlCache.value.get(equippedSkin.value.texture_key) ?? null
	})

	async function refreshEquippedSkinAvatar(accountList?: AccountLike[]) {
		try {
			const skins = await get_available_skins()
			equippedSkin.value = skins.find((skin) => skin.is_equipped) ?? null

			if (equippedSkin.value) {
				try {
					const headUrl = await getPlayerHeadUrl(equippedSkin.value)
					cacheTextureHead(equippedSkin.value.texture_key, headUrl)
				} catch (error) {
					console.warn('Failed to get head render for equipped skin:', error)
				}
			}
		} catch {
			equippedSkin.value = null
		}

		const accounts = Array.isArray(accountList)
			? accountList
			: await users().catch(() => [] as AccountLike[])
		const selectedId = await get_default_user().catch(() => undefined)
		markOfflineProfiles(accounts)

		await Promise.all(
			accounts.map(async (account) => {
				const profileId = account.profile?.id
				if (!profileId) {
					return
				}

				await resolveProfileHead(profileId, profileId === selectedId)
			}),
		)
	}

	async function setEquippedSkinAvatar(skin: Skin) {
		equippedSkin.value = skin

		try {
			const headUrl = await getPlayerHeadUrl(skin)
			cacheTextureHead(skin.texture_key, headUrl)
			const selectedId = await get_default_user().catch(() => undefined)
			if (selectedId) {
				cacheProfileHead(selectedId, headUrl)
			}
		} catch (error) {
			console.warn('Failed to get head render for equipped skin:', error)
		}
	}

	function getAccountAvatarUrl(
		profileId: string | undefined,
		isSelectedAccount: boolean,
		isOffline = false,
	) {
		const offline = isOffline || (!!profileId && offlineProfileIds.value.has(profileId))

		if (isSelectedAccount) {
			const equippedUrl = equippedSkinAvatarUrl.value
			if (equippedUrl) {
				return equippedUrl
			}
		}

		if (profileId) {
			const cached = profileHeadUrlCache.value.get(profileId)
			if (cached) {
				return cached
			}
		}

		if (profileId && !offline) {
			return MC_HEADS_AVATAR(profileId)
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
