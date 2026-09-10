import { I18N_INJECTION_KEY, type I18nContext } from '@lumen/ui'
import type { App } from 'vue'

import i18n, { DEFAULT_APP_LOCALE } from '@/i18n.config'

export default {
	install(app: App) {
		i18n.global.locale.value = DEFAULT_APP_LOCALE
		document.documentElement.lang = 'pl'
		app.use(i18n)

		const context: I18nContext = {
			locale: i18n.global.locale,
			t: (key, values) => i18n.global.t(key, values ?? {}) as string,
			// Language switching is intentionally disabled — Polish only.
			setLocale: () => {
				i18n.global.locale.value = DEFAULT_APP_LOCALE
				document.documentElement.lang = 'pl'
			},
		}

		app.provide(I18N_INJECTION_KEY, context)
	},
}
