import {
	type AuthConfig,
	AuthFeature,
	type FeatureConfig,
	type NuxtClientConfig,
	NuxtLumenClient,
} from '@lumen/api-client'
import { getRequestHeader, type H3Event } from 'h3'

import { readEnv } from '~/helpers/env'
import { getFrontendUserAgent, VISITOR_USER_AGENT_HEADER } from '~/helpers/user-agent'

export interface ServerLumenClientOptions {
	event?: H3Event
	authToken?: string
}

export function useServerLumenClient(options?: ServerLumenClientOptions): NuxtLumenClient {
	const config = useRuntimeConfig(options?.event)
	const apiBaseUrl = (config.apiBaseUrl || config.public.apiBaseUrl).replace('/v2/', '/')
	const sharedInstancesBaseUrl =
		config.sharedInstancesBaseUrl || config.public.sharedInstancesBaseUrl

	const visitorUserAgent = options?.event
		? getRequestHeader(options.event, 'user-agent')
		: undefined

	const features = []

	if (options?.authToken) {
		features.push(
			new AuthFeature({
				token: options.authToken,
				tokenPrefix: '',
			} as AuthConfig as FeatureConfig),
		)
	}

	const clientConfig: NuxtClientConfig = {
		labrinthBaseUrl: apiBaseUrl,
		sharedInstancesBaseUrl,
		userAgent: getFrontendUserAgent(config.public.hash),
		headers: visitorUserAgent ? { [VISITOR_USER_AGENT_HEADER]: visitorUserAgent } : undefined,
		rateLimitKey: config.rateLimitKey || (() => readEnv('RATE_LIMIT_IGNORE_KEY')),
		features,
	}

	return new NuxtLumenClient(clientConfig)
}
