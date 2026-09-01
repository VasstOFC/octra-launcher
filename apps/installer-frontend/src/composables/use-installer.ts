import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { onMounted, onUnmounted, ref } from 'vue'

export type InstallerStep = 'welcome' | 'destination' | 'options' | 'progress' | 'done'

export type InstallProgress = {
	step: string
	progress: number
	message: string
}

export function useInstaller() {
	const step = ref<InstallerStep>('welcome')
	const installDir = ref('')
	const desktopShortcut = ref(true)
	const launchAfter = ref(true)
	const progress = ref(0)
	const statusMessage = ref('')
	const error = ref<string | null>(null)
	const installing = ref(false)

	let unlisten: (() => void) | null = null

	onMounted(async () => {
		installDir.value = await invoke<string>('default_install_dir')
		unlisten = await listen<InstallProgress>('install-progress', (event) => {
			progress.value = Math.round(event.payload.progress * 100)
			statusMessage.value = event.payload.message
		})
	})

	onUnmounted(() => {
		unlisten?.()
	})

	async function browseInstallDir() {
		const selected = await open({
			directory: true,
			multiple: false,
			defaultPath: installDir.value,
			title: 'Wybierz folder instalacji',
		})
		if (typeof selected === 'string') {
			installDir.value = selected
		}
	}

	async function startInstall() {
		error.value = null
		installing.value = true
		step.value = 'progress'
		progress.value = 0
		statusMessage.value = 'Rozpoczynanie instalacji…'

		try {
			await invoke('run_install', {
				request: {
					installDir: installDir.value,
					desktopShortcut: desktopShortcut.value,
					launchAfter: false,
				},
			})
			if (launchAfter.value) {
				await invoke('launch_installed_app', { installDir: installDir.value })
			}
			step.value = 'done'
		} catch (cause) {
			error.value = cause instanceof Error ? cause.message : String(cause)
			step.value = 'options'
		} finally {
			installing.value = false
		}
	}

	function closeInstaller() {
		getCurrentWindow().close()
	}

	return {
		step,
		installDir,
		desktopShortcut,
		launchAfter,
		progress,
		statusMessage,
		error,
		installing,
		browseInstallDir,
		startInstall,
		closeInstaller,
	}
}
