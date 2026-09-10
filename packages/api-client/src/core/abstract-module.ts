import type { AbstractLumenClient } from './abstract-client'

export abstract class AbstractModule {
	protected client: AbstractLumenClient

	public constructor(client: AbstractLumenClient) {
		this.client = client
	}

	/**
	 * Get the module's name, used for error reporting & for module field generation.
	 * @returns Module name
	 */
	public abstract getModuleID(): string
}
