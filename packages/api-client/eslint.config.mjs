import config from '@lumen/tooling-config/eslint/nuxt.mjs'

export default config.append([
	{
		ignores: ['dist/'],
	},
])
