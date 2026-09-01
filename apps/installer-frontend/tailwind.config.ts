import preset from '@modrinth/tooling-config/tailwind/tailwind-preset.ts'
import type { Config } from 'tailwindcss'

export default {
	content: ['./index.html', './src/**/*.{vue,ts}'],
	presets: [preset],
} satisfies Config
