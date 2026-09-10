import type { AbstractLumenClient } from '@lumen/api-client'
import { provideUserCountry } from '@lumen/ui'
import { ref } from 'vue'

export function setupUserCountryProvider(client: AbstractLumenClient) {
	const country = ref('US')

	void client.labrinth.geoip
		.getCountry()
		.then((detectedCountry) => {
			country.value = detectedCountry ?? country.value
		})
		.catch(() => {})

	return provideUserCountry(country)
}
