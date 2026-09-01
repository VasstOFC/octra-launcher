import { invoke } from '@tauri-apps/api/core'

/**
 * @typedef {Object} OctraAccountSession
 * @property {string} token
 * @property {string} username
 * @property {string} minecraft_nick
 * @property {string} profile_uuid
 */

/** @returns {Promise<OctraAccountSession | null>} */
export async function octraAccountSession() {
	return await invoke('plugin:octra|octra_account_session')
}

/** @returns {Promise<OctraAccountSession>} */
export async function octraAccountRegister(username, password, minecraftNick) {
	return await invoke('plugin:octra|octra_account_register', {
		username,
		password,
		minecraftNick,
	})
}

/** @returns {Promise<OctraAccountSession>} */
export async function octraAccountLogin(username, password) {
	return await invoke('plugin:octra|octra_account_login', { username, password })
}

export async function octraAccountLogout() {
	return await invoke('plugin:octra|octra_account_logout')
}
