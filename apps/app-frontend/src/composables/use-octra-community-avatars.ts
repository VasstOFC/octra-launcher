import { ref, watch, type Ref } from 'vue'

import { getPlayerHeadUrl } from '@/helpers/rendering/batch-skin-renderer'
import type { Skin } from '@/helpers/skins'

/** Same Steve head used by the account switcher. */
export const OCTRA_COMMUNITY_STEVE_HEAD =
	'https://launcher-files.modrinth.com/assets/steve_head.png'

const MISS_RETRY_MS = 60_000

type MemberLike = {
	profile_uuid: string
	minecraft_nick: string
	avatar_url: string
}

function memberKey(member: MemberLike): string {
	return (member.profile_uuid || member.minecraft_nick).trim().toLowerCase()
}

function candidateSkinUrls(member: MemberLike): string[] {
	const urls: string[] = []
	const push = (url: string) => {
		const trimmed = url.trim()
		if (trimmed && !urls.includes(trimmed)) {
			urls.push(trimmed)
		}
	}

	push(member.avatar_url)

	try {
		const origin = new URL(member.avatar_url).origin
		const nick = member.minecraft_nick.trim()
		const uuid = member.profile_uuid.trim()
		if (nick) {
			push(`${origin}/skins/MinecraftSkins/${encodeURIComponent(nick)}.png`)
		}
		if (uuid) {
			push(`${origin}/skins/${uuid}`)
		}
	} catch {
		// avatar_url may be empty before the first community payload
	}

	return urls
}

export function useOctraCommunityAvatars(members: Ref<MemberLike[]>) {
	const headByKey = ref(new Map<string, string>())
	const missUntilByKey = new Map<string, number>()
	const inFlight = new Set<string>()
	const revision = ref(0)

	function avatarFor(member: MemberLike): string {
		void revision.value
		return headByKey.value.get(memberKey(member)) ?? OCTRA_COMMUNITY_STEVE_HEAD
	}

	async function resolveMember(member: MemberLike): Promise<void> {
		const key = memberKey(member)
		if (!key || inFlight.has(key)) {
			return
		}

		const cached = headByKey.value.get(key)
		if (cached && cached !== OCTRA_COMMUNITY_STEVE_HEAD) {
			return
		}
		if (cached === OCTRA_COMMUNITY_STEVE_HEAD) {
			const missUntil = missUntilByKey.get(key) ?? 0
			if (missUntil > Date.now()) {
				return
			}
		}

		inFlight.add(key)
		try {
			for (const texture of candidateSkinUrls(member)) {
				try {
					const skin: Skin = {
						texture_key: `octra-community-${key}`,
						variant: 'CLASSIC',
						texture,
						source: 'custom_external',
						is_equipped: false,
					}
					const headUrl = await getPlayerHeadUrl(skin)
					headByKey.value = new Map(headByKey.value).set(key, headUrl)
					missUntilByKey.delete(key)
					revision.value++
					return
				} catch {
					// try next candidate (uuid 404, nick path, etc.)
				}
			}

			headByKey.value = new Map(headByKey.value).set(key, OCTRA_COMMUNITY_STEVE_HEAD)
			missUntilByKey.set(key, Date.now() + MISS_RETRY_MS)
			revision.value++
		} finally {
			inFlight.delete(key)
		}
	}

	watch(
		members,
		(list) => {
			void Promise.all(list.map((member) => resolveMember(member)))
		},
		{ immediate: true, deep: true },
	)

	return {
		avatarFor,
		steveHead: OCTRA_COMMUNITY_STEVE_HEAD,
	}
}
