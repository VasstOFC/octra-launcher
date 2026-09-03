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
 * @property {number|null} [last_id]
 * @property {number} [last_read_id]
 * @property {number} [unread_count]
 * @property {OctraChatMember[]} [members]
 */

/**
 * @typedef {Object} OctraChatReaction
 * @property {string} emoji
 * @property {number} count
 * @property {number[]} [user_ids]
 */

/**
 * @typedef {Object} OctraChatMessage
 * @property {number} id
 * @property {number} [channel_id]
 * @property {number} user_id
 * @property {string} minecraft_nick
 * @property {string} body
 * @property {string} created_at
 * @property {boolean} [pinned]
 * @property {boolean} [deleted]
 * @property {OctraChatReaction[]} [reactions]
 */

/**
 * @typedef {Object} OctraSharedServer
 * @property {number} id
 * @property {string} name
 * @property {string} address
 * @property {number} created_by
 * @property {string|null} [created_by_nick]
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
 * @param {number} channelId
 * @param {number[]} memberIds
 */
export async function octraChatAddMembers(channelId, memberIds) {
	return await invoke('plugin:octra|octra_chat_add_members', { channelId, memberIds })
}

/**
 * @param {number} channelId
 * @param {number} lastReadId
 */
export async function octraChatMarkRead(channelId, lastReadId) {
	return await invoke('plugin:octra|octra_chat_mark_read', { channelId, lastReadId })
}

/** @param {number} messageId */
export async function octraChatDeleteMessage(messageId) {
	return await invoke('plugin:octra|octra_chat_delete_message', { messageId })
}

/**
 * @param {number} messageId
 * @param {boolean} pinned
 */
export async function octraChatPinMessage(messageId, pinned) {
	return await invoke('plugin:octra|octra_chat_pin_message', { messageId, pinned })
}

/**
 * @param {number} messageId
 * @param {string} emoji
 */
export async function octraChatReactMessage(messageId, emoji) {
	return await invoke('plugin:octra|octra_chat_react_message', { messageId, emoji })
}

/** @param {string} address */
export async function octraShareJoinAddress(address) {
	return await invoke('plugin:octra|octra_share_join_address', { address })
}

/** @returns {Promise<OctraSharedServer[]>} */
export async function octraSharedServersList() {
	return await invoke('plugin:octra|octra_shared_servers_list')
}

/**
 * @param {string} name
 * @param {string} address
 */
export async function octraSharedServersAdd(name, address) {
	return await invoke('plugin:octra|octra_shared_servers_add', { name, address })
}

/** @param {number} serverId */
export async function octraSharedServersDelete(serverId) {
	return await invoke('plugin:octra|octra_shared_servers_delete', { serverId })
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
