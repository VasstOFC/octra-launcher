import { invoke } from '@tauri-apps/api/core'

/**
 * @typedef {Object} OctraAccountSession
 * @property {string} token
 * @property {string} username
 * @property {string} minecraft_nick
 * @property {string} profile_uuid
 * @property {string} [account_type]
 */

/**
 * @typedef {Object} OctraCommunityMember
 * @property {number} id
 * @property {string} minecraft_nick
 * @property {string} profile_uuid
 * @property {string} account_type
 * @property {string} created_at
 * @property {string} avatar_url
 * @property {string} [presence]
 * @property {string | null} [instance_name]
 * @property {string | null} [join_address]
 * @property {string | null} [last_seen]
 */

/**
 * @typedef {Object} OctraCommunitySnapshot
 * @property {boolean} connected
 * @property {OctraCommunityMember[]} members
 */

/**
 * @typedef {Object} OctraChatMember
 * @property {number} id
 * @property {string} minecraft_nick
 * @property {string} profile_uuid
 */

/**
 * @typedef {Object} OctraChatChannel
 * @property {number} id
 * @property {'dm'|'group'|string} kind
 * @property {string|null} [name]
 * @property {string} created_at
 * @property {string|null} [last_body]
 * @property {string|null} [last_at]
 * @property {OctraChatMember[]} [members]
 */

/**
 * @typedef {Object} OctraChatMessage
 * @property {number} id
 * @property {number} [channel_id]
 * @property {number} user_id
 * @property {string} minecraft_nick
 * @property {string} body
 * @property {string} created_at
 */

/** @returns {Promise<OctraAccountSession | null>} */
export async function octraAccountSession() {
	return await invoke('plugin:octra|octra_account_session')
}

/**
 * @param {string} password
 * @returns {Promise<OctraAccountSession>}
 */
export async function octraAccountRegister(password) {
	return await invoke('plugin:octra|octra_account_register', { password })
}

/** @returns {Promise<OctraAccountSession>} */
export async function octraAccountLogin(username, password) {
	return await invoke('plugin:octra|octra_account_login', { username, password })
}

export async function octraAccountLogout() {
	return await invoke('plugin:octra|octra_account_logout')
}

/** @returns {Promise<OctraCommunitySnapshot>} */
export async function octraCommunity() {
	return await invoke('plugin:octra|octra_community')
}

/** @returns {Promise<OctraChatChannel[]>} */
export async function octraChatChannels() {
	return await invoke('plugin:octra|octra_chat_channels')
}

/**
 * @param {number} userId
 * @returns {Promise<OctraChatChannel>}
 */
export async function octraChatOpenDm(userId) {
	return await invoke('plugin:octra|octra_chat_open_dm', { userId })
}

/**
 * @param {string} name
 * @param {number[]} memberIds
 * @returns {Promise<OctraChatChannel>}
 */
export async function octraChatCreateGroup(name, memberIds) {
	return await invoke('plugin:octra|octra_chat_create_group', { name, memberIds })
}

/**
 * @param {number} channelId
 * @param {number} [afterId]
 * @returns {Promise<OctraChatMessage[]>}
 */
export async function octraChatList(channelId, afterId = 0) {
	return await invoke('plugin:octra|octra_chat_list', { channelId, afterId })
}

/**
 * @param {number} channelId
 * @param {string} text
 * @returns {Promise<OctraChatMessage>}
 */
export async function octraChatPost(channelId, text) {
	return await invoke('plugin:octra|octra_chat_post', { channelId, text })
}

/**
 * @param {string} url
 * @returns {Promise<string>}
 */
export async function octraCacheMrpackUrl(url) {
	return await invoke('plugin:octra|octra_cache_mrpack_url', { url })
}

/** @param {string} text */
export function extractMrpackUrls(text) {
	const matches = text.match(/https?:\/\/[^\s<>"']+\.mrpack\b/gi)
	if (!matches) return []
	return [...new Set(matches)]
}

/**
 * @param {OctraChatChannel} channel
 * @param {string} [selfNick]
 */
export function channelTitle(channel, selfNick) {
	if (channel.kind === 'group') {
		return channel.name || 'Group'
	}
	const other = (channel.members || []).find(
		(m) => m.minecraft_nick.toLowerCase() !== (selfNick || '').toLowerCase(),
	)
	return other?.minecraft_nick || channel.name || 'Direct message'
}
