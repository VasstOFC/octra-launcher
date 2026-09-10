import { ref, watch } from 'vue'

const BACKGROUND_KEY = 'Lumen.background'
const DIMMING_KEY = 'Lumen.backgroundDimming'

export type BackgroundMode = 'none' | 'color' | 'image'

interface BackgroundState {
	mode: BackgroundMode
	color: string
	imageUrl: string
	dimming: number
}

const DEFAULT_STATE: BackgroundState = {
	mode: 'none',
	color: '#0a0a0f',
	imageUrl: '',
	dimming: 85,
}

function loadState(): BackgroundState {
	try {
		const stored = localStorage.getItem(BACKGROUND_KEY)
		if (stored) {
			return { ...DEFAULT_STATE, ...JSON.parse(stored) }
		}
	} catch {}
	return { ...DEFAULT_STATE }
}

function loadDimming(): number {
	const stored = localStorage.getItem(DIMMING_KEY)
	if (stored) {
		const val = parseInt(stored, 10)
		if (!isNaN(val) && val >= 0 && val <= 100) return val
	}
	return 85
}

const state = ref<BackgroundState>(loadState())
const dimming = ref(loadDimming())

watch(state, (val) => {
	localStorage.setItem(BACKGROUND_KEY, JSON.stringify(val))
	applyBackground()
}, { deep: true })

watch(dimming, (val) => {
	localStorage.setItem(DIMMING_KEY, String(val))
	applyBackground()
})

function applyBackground() {
	const root = document.documentElement
	const s = state.value
	const d = dimming.value

	if (s.mode === 'image' && s.imageUrl) {
		root.style.setProperty('--custom-bg-image', `url("${s.imageUrl}")`)
		root.style.setProperty('--custom-bg-dimming', `${d / 100}`)
		root.classList.add('has-custom-bg')
	} else {
		root.style.removeProperty('--custom-bg-image')
		root.style.removeProperty('--custom-bg-dimming')
		root.classList.remove('has-custom-bg')
	}
}

export function useBackground() {
	function setMode(mode: BackgroundMode) {
		state.value.mode = mode
	}

	function setImage(url: string) {
		state.value.imageUrl = url
		state.value.mode = 'image'
	}

	function setColor(color: string) {
		state.value.color = color
		state.value.mode = 'color'
	}

	function setDimming(value: number) {
		dimming.value = Math.max(0, Math.min(100, value))
	}

	function removeBackground() {
		state.value = { ...DEFAULT_STATE }
		localStorage.removeItem(BACKGROUND_KEY)
		localStorage.removeItem(DIMMING_KEY)
		applyBackground()
	}

	function init() {
		applyBackground()
	}

	return {
		state,
		dimming,
		setMode,
		setImage,
		setColor,
		setDimming,
		removeBackground,
		init,
	}
}
