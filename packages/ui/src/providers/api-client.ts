import type { AbstractLumenClient } from '@lumen/api-client'

import { createContext } from './create-context'

export const [injectLumenClient, provideLumenClient] = createContext<AbstractLumenClient>(
	'root',
	'LumenClient',
)
