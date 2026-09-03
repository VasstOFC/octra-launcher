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
 * @property {string | null} [last_seen]
 */

/**
 * @typedef {Object} OctraCommunitySnapshot
 * @property {boolean} connected
 * @property {OctraCommunityMember[]} members
 */

/** @returns {Promise<OctraAccountSession | null>} */
export async function octraAccountSession() {
	return await invoke('plugin:octra|octra_account_session')
}

/**
 * Register Octra account linked to the launcher's default Minecraft account.
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
