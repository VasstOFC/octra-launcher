import { buildLocaleMessages, createMessageCompiler, type CrowdinMessages } from '@lumen/ui'
import { uiLocaleModulesEager } from '@lumen/ui/src/locales.eager.ts'
import { createI18n } from 'vue-i18n'

export const DEFAULT_APP_LOCALE = 'pl-PL'
export const FALLBACK_APP_LOCALE = 'en-US'

const ALLOWED_LOCALES = new Set([DEFAULT_APP_LOCALE, FALLBACK_APP_LOCALE])

function filterLocaleModules(
	modules: Record<string, { default: CrowdinMessages }>,
): Record<string, { default: CrowdinMessages }> {
	return Object.fromEntries(
		Object.entries(modules).filter(([path]) =>
			[...ALLOWED_LOCALES].some(
				(locale) => path.includes(`/${locale}/`) || path.includes(`\\${locale}\\`),
			),
		),
	)
}

const localeModules = filterLocaleModules(
	import.meta.glob<{ default: CrowdinMessages }>('./locales/*/index.json', {
		eager: true,
	}),
)

const i18n = createI18n({
	legacy: false,
	locale: DEFAULT_APP_LOCALE,
	fallbackLocale: FALLBACK_APP_LOCALE,
	messageCompiler: createMessageCompiler(),
	missingWarn: false,
	fallbackWarn: false,
	messages: buildLocaleMessages(localeModules, filterLocaleModules(uiLocaleModulesEager)),
})

export default i18n
