import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'
import { defineConfig } from 'vite'
import svgLoader from 'vite-svg-loader'

export default defineConfig({
	clearScreen: false,
	server: {
		port: 1421,
		strictPort: true,
	},
	resolve: {
		alias: {
			'@': resolve(__dirname, 'src'),
		},
	},
	plugins: [
		vue(),
		svgLoader({
			svgoConfig: {
				plugins: [{ name: 'preset-default', params: { overrides: { removeViewBox: false } } }],
			},
		}),
	],
	build: {
		target: 'es2021',
		minify: !process.env.TAURI_DEBUG,
		sourcemap: !!process.env.TAURI_DEBUG,
	},
})
