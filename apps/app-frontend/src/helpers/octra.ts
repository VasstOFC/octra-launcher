import { invoke } from '@tauri-apps/api/core'

export type OctraServer = {
	name: string
	address: string
}

export async function listOctraServers(): Promise<OctraServer[]> {
	return await invoke('plugin:octra|list_servers')
}

export async function removeOctraServer(address: string): Promise<boolean> {
	return await invoke('plugin:octra|remove_server', { address })
}
