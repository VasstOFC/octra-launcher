import { I18N_INJECTION_KEY, type I18nContext } from '@modrinth/ui'
import type { App } from 'vue'

import i18n, { DEFAULT_APP_LOCALE } from '@/i18n.config'

export default {
	install(app: App) {
		i18n.global.locale.value = DEFAULT_APP_LOCALE
		app.use(i18n)

		const context: I18nContext = {
			locale: i18n.global.locale,
			t: (key, values) => i18n.global.t(key, values ?? {}) as string,
			setLocale: () => {
				i18n.global.locale.value = DEFAULT_APP_LOCALE
			},
		}

		app.provide(I18N_INJECTION_KEY, context)
	},
}
