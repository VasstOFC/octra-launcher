import { invoke } from '@tauri-apps/api/core'

/**
 * @typedef {Object} LumenAccountSession
 * @property {string} token
 * @property {string} username
 * @property {string} minecraft_nick
 * @property {string} profile_uuid
 * @property {string} [account_type]
 */

/**
 * @typedef {Object} LumenCommunityMember
 * @property {number} id
 * @property {string} minecraft_nick
 * @property {string} profile_uuid
 * @property {string} account_type
 * @property {string} created_at
 * @property {string} avatar_url
 * @property {string} [presence]
 * @property {string | null} [instance_name]
 * @property {string | null} [join_address]
 * @property {string | null} [pack_project_id]
 * @property {string | null} [pack_version_id]
 * @property {string | null} [last_seen]
 */

/**
 * @typedef {Object} LumenCommunitySnapshot
 * @property {boolean} connected
 * @property {LumenCommunityMember[]} members
 */

/**
 * @typedef {Object} LumenChatMember
 * @property {number} id
 * @property {string} minecraft_nick
 * @property {string} profile_uuid
 */

/**
 * @typedef {Object} LumenChatChannel
 * @property {number} id
 * @property {'dm'|'group'|string} kind
 * @property {string|null} [name]
 * @property {string} created_at
 * @property {string|null} [last_body]
 * @property {string|null} [last_at]
 * @property {number|null} [last_id]
 * @property {number} [last_read_id]
 * @property {number} [unread_count]
 * @property {LumenChatMember[]} [members]
 */

/**
 * @typedef {Object} LumenChatReaction
 * @property {string} emoji
 * @property {number} count
 * @property {number[]} [user_ids]
 */

/**
 * @typedef {Object} LumenChatMessage
 * @property {number} id
 * @property {number} [channel_id]
 * @property {number} user_id
 * @property {string} minecraft_nick
 * @property {string} body
 * @property {string} created_at
 * @property {boolean} [pinned]
 * @property {boolean} [deleted]
 * @property {string|null} [attachment_url]
 * @property {LumenChatReaction[]} [reactions]
 */

/**
 * @typedef {Object} LumenChatAttachment
 * @property {string} url
 * @property {string} path
 */

/**
 * @typedef {Object} LumenSharedServer
 * @property {number} id
 * @property {string} name
 * @property {string} address
 * @property {number} created_by
 * @property {string|null} [created_by_nick]
 * @property {string} created_at
 */

/** @returns {Promise<LumenAccountSession | null>} */
export async function LumenAccountSession() {
	return await invoke('plugin:octra|octra_account_session')
}

/**
 * @param {string} password
 * @returns {Promise<LumenAccountSession>}
 */
export async function LumenAccountRegister(password) {
	return await invoke('plugin:octra|octra_account_register', { password })
}

/** @returns {Promise<LumenAccountSession>} */
export async function LumenAccountLogin(username, password) {
	return await invoke('plugin:octra|octra_account_login', { username, password })
}

export async function LumenAccountLogout() {
	return await invoke('plugin:octra|octra_account_logout')
}

/** @returns {Promise<LumenCommunitySnapshot>} */
export async function LumenCommunity() {
	return await invoke('plugin:octra|octra_community')
}

/** @returns {Promise<LumenChatChannel[]>} */
export async function LumenChatChannels() {
	return await invoke('plugin:octra|octra_chat_channels')
}

/**
 * @param {number} userId
 * @returns {Promise<LumenChatChannel>}
 */
export async function LumenChatOpenDm(userId) {
	return await invoke('plugin:octra|octra_chat_open_dm', { userId })
}

/**
 * @param {string} name
 * @param {number[]} memberIds
 * @returns {Promise<LumenChatChannel>}
 */
export async function LumenChatCreateGroup(name, memberIds) {
	return await invoke('plugin:octra|octra_chat_create_group', { name, memberIds })
}

/**
 * @param {number} channelId
 * @param {number} [afterId]
 * @returns {Promise<LumenChatMessage[]>}
 */
export async function LumenChatList(channelId, afterId = 0) {
	return await invoke('plugin:octra|octra_chat_list', { channelId, afterId })
}

/**
 * @param {number} channelId
 * @param {string} text
 * @param {string|null} [attachmentUrl]
 * @returns {Promise<LumenChatMessage>}
 */
export async function LumenChatPost(channelId, text, attachmentUrl = null) {
	return await invoke('plugin:octra|octra_chat_post', {
		channelId,
		text,
		attachmentUrl,
	})
}

/**
 * @param {string} path
 * @returns {Promise<LumenChatAttachment>}
 */
export async function LumenChatUploadImage(path) {
	return await invoke('plugin:octra|octra_chat_upload_image', { path })
}

/**
 * @param {number} channelId
 * @param {number[]} memberIds
 */
export async function LumenChatAddMembers(channelId, memberIds) {
	return await invoke('plugin:octra|octra_chat_add_members', { channelId, memberIds })
}

/**
 * @param {number} channelId
 * @param {number} lastReadId
 */
export async function LumenChatMarkRead(channelId, lastReadId) {
	return await invoke('plugin:octra|octra_chat_mark_read', { channelId, lastReadId })
}

/** @param {number} messageId */
export async function LumenChatDeleteMessage(messageId) {
	return await invoke('plugin:octra|octra_chat_delete_message', { messageId })
}

/**
 * @param {number} messageId
 * @param {boolean} pinned
 */
export async function LumenChatPinMessage(messageId, pinned) {
	return await invoke('plugin:octra|octra_chat_pin_message', { messageId, pinned })
}

/**
 * @param {number} messageId
 * @param {string} emoji
 */
export async function LumenChatReactMessage(messageId, emoji) {
	return await invoke('plugin:octra|octra_chat_react_message', { messageId, emoji })
}

/**
 * @typedef {Object} LumenChatDeleteVote
 * @property {boolean} active
 * @property {number} channel_id
 * @property {number} member_count
 * @property {number} yes_count
 * @property {number} no_count
 * @property {number} needed
 * @property {boolean|null} [my_vote]
 * @property {boolean} [deleted]
 */

/**
 * @param {number} channelId
 * @returns {Promise<LumenChatDeleteVote>}
 */
export async function LumenChatGetDeleteVote(channelId) {
	return await invoke('plugin:octra|octra_chat_get_delete_vote', { channelId })
}

/**
 * @param {number} channelId
 * @param {boolean} yes
 * @returns {Promise<LumenChatDeleteVote>}
 */
export async function LumenChatCastDeleteVote(channelId, yes) {
	return await invoke('plugin:octra|octra_chat_cast_delete_vote', { channelId, yes })
}

/** @param {string} address */
export async function LumenShareJoinAddress(address) {
	return await invoke('plugin:octra|octra_share_join_address', { address })
}

/** @returns {Promise<LumenSharedServer[]>} */
export async function LumenSharedServersList() {
	return await invoke('plugin:octra|octra_shared_servers_list')
}

/**
 * @param {string} name
 * @param {string} address
 */
export async function LumenSharedServersAdd(name, address) {
	return await invoke('plugin:octra|octra_shared_servers_add', { name, address })
}

/** @param {number} serverId */
export async function LumenSharedServersDelete(serverId) {
	return await invoke('plugin:octra|octra_shared_servers_delete', { serverId })
}

/**
 * @param {string} url
 * @returns {Promise<string>}
 */
export async function LumenCacheMrpackUrl(url) {
	return await invoke('plugin:octra|octra_cache_mrpack_url', { url })
}

/** @param {string} text */
export function extractMrpackUrls(text) {
	const matches = text.match(/https?:\/\/[^\s<>"']+\.mrpack\b/gi)
	if (!matches) return []
	return [...new Set(matches)]
}

/**
 * @param {LumenChatChannel} channel
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

