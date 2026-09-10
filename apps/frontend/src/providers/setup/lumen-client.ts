import { provideLumenClient } from '@lumen/ui'

import { createLumenClient } from '~/helpers/api.ts'

export function setupLumenClientProvider(auth: Awaited<ReturnType<typeof useAuth>>) {
	const config = useRuntimeConfig()
	const client = createLumenClient(auth, {
		apiBaseUrl: config.public.apiBaseUrl.replace('/v2/', '/'),
		archonBaseUrl: config.public.pyroBaseUrl.replace('/v2/', '/'),
		sharedInstancesBaseUrl: config.public.sharedInstancesBaseUrl,
		commitHash: config.public.hash,
		rateLimitKey: config.rateLimitKey,
	})
	provideLumenClient(client)
	return client
}
