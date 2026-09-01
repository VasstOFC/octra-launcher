import { provideAppBackup } from '@modrinth/ui'
import { type MaybeRefOrGetter, toValue } from 'vue'

import i18n from '@/i18n.config'
import { install_duplicate_instance, installJobInstanceId } from '@/helpers/install'
import { edit, list } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

export function provideInstanceBackup(instance: MaybeRefOrGetter<GameInstance>) {
	provideAppBackup({
		async createBackup() {
			const source = toValue(instance)
			const prefix = i18n.global.t('instance.backup.name-prefix', { name: source.name })
			const legacyPrefix = `${source.name} - Backup #`
			const existingNumbers = (await list())
				.filter(
					(candidate) =>
						candidate.name.startsWith(prefix) || candidate.name.startsWith(legacyPrefix),
				)
				.map((candidate) => {
					if (candidate.name.startsWith(prefix)) {
						return Number.parseInt(candidate.name.slice(prefix.length), 10)
					}

					return Number.parseInt(candidate.name.slice(legacyPrefix.length), 10)
				})
				.filter(Number.isFinite)
			const nextNumber = existingNumbers.length ? Math.max(...existingNumbers) + 1 : 1
			const job = await install_duplicate_instance(source.id)
			const backupInstanceId = installJobInstanceId(job)
			if (backupInstanceId) {
				await edit(backupInstanceId, { name: `${prefix}${nextNumber}` })
			}
		},
	})
}
