import { invoke } from '@tauri-apps/api/core'

export type LumenServer = {
	name: string
	address: string
}

export async function listLumenServers(): Promise<LumenServer[]> {
	return await invoke('plugin:Lumen|list_servers')
}

export async function removeLumenServer(address: string): Promise<boolean> {
	return await invoke('plugin:Lumen|remove_server', { address })
}
