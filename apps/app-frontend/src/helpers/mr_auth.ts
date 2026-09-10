/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank instance object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'

export type LumenCredentials = {
	session: string
	expires: string
	user_id: string
	active: boolean
}

export type LumenAuthFlow = 'sign-in' | 'sign-up'

export async function login(
	flow: LumenAuthFlow = 'sign-in',
	addAccount = false,
): Promise<LumenCredentials> {
	return await invoke('plugin:mr-auth|lumen_login', { flow, addAccount })
}

export async function logout(): Promise<void> {
	return await invoke('plugin:mr-auth|logout')
}

export async function get(): Promise<LumenCredentials | null> {
	return await invoke('plugin:mr-auth|get')
}

export async function getAll(): Promise<LumenCredentials[]> {
	return await invoke('plugin:mr-auth|get_all')
}

export async function setActive(userId: string): Promise<void> {
	return await invoke('plugin:mr-auth|set_active', { userId })
}

export async function removeUser(userId: string): Promise<void> {
	return await invoke('plugin:mr-auth|remove_account', { userId })
}

export async function cancelLogin(): Promise<void> {
	return await invoke('plugin:mr-auth|cancel_lumen_login')
}
