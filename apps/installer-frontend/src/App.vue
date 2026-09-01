<script setup lang="ts">
import { computed } from 'vue'

import OctraMark from '@/components/OctraMark.vue'
import ProgressBar from '@/components/ProgressBar.vue'
import { useInstaller } from '@/composables/use-installer'

const {
	step,
	installDir,
	desktopShortcut,
	launchAfter,
	progress,
	statusMessage,
	error,
	installing,
	browseInstallDir,
	startInstall,
	closeInstaller,
} = useInstaller()

const canGoBack = computed(
	() => step.value === 'destination' || step.value === 'options',
)

function goBack() {
	if (step.value === 'options') {
		step.value = 'destination'
		return
	}
	if (step.value === 'destination') {
		step.value = 'welcome'
	}
}

function goNext() {
	if (step.value === 'welcome') {
		step.value = 'destination'
		return
	}
	if (step.value === 'destination') {
		step.value = 'options'
		return
	}
	if (step.value === 'options') {
		startInstall()
	}
}
</script>

<template>
	<div class="installer-shell dark-mode">
		<div class="installer-shell__bg" />
		<div class="installer-shell__cube" />

		<header class="installer-shell__header" data-tauri-drag-region>
			<div class="installer-shell__brand">
				<OctraMark class="installer-shell__logo" />
				<div>
					<p class="installer-shell__title">Octra Setup</p>
					<p class="installer-shell__subtitle">Instalator Octra App</p>
				</div>
			</div>
		</header>

		<main class="installer-shell__content">
			<section v-if="step === 'welcome'">
				<h1 class="installer-step-title">Witaj w Octra App</h1>
				<p class="installer-step-desc">
					Ten kreator zainstaluje Octra App na Twoim komputerze. Możesz wybrać folder
					instalacji, utworzyć skrót na pulpicie i od razu uruchomić aplikację po zakończeniu.
				</p>
			</section>

			<section v-else-if="step === 'destination'">
				<h1 class="installer-step-title">Folder instalacji</h1>
				<p class="installer-step-desc">Wybierz, gdzie ma zostać zainstalowana Octra App.</p>
				<div class="installer-field">
					<label for="install-dir">Lokalizacja</label>
					<div class="installer-field__row">
						<input id="install-dir" v-model="installDir" type="text" spellcheck="false" />
						<button type="button" class="installer-btn installer-btn--ghost" @click="browseInstallDir">
							Przeglądaj…
						</button>
					</div>
				</div>
			</section>

			<section v-else-if="step === 'options'">
				<h1 class="installer-step-title">Opcje instalacji</h1>
				<p class="installer-step-desc">Dostosuj instalację do swoich preferencji.</p>
				<label class="installer-option">
					<input v-model="desktopShortcut" type="checkbox" />
					Utwórz skrót na pulpicie
				</label>
				<label class="installer-option">
					<input v-model="launchAfter" type="checkbox" />
					Uruchom Octra App po instalacji
				</label>
				<p v-if="error" class="installer-error">{{ error }}</p>
			</section>

			<section v-else-if="step === 'progress'">
				<h1 class="installer-step-title">Instalowanie…</h1>
				<p class="installer-step-desc">Proszę czekać — kopiujemy pliki i konfigurujemy skróty.</p>
				<ProgressBar class="mt-6" :progress="progress" />
				<p class="installer-progress-copy">{{ statusMessage }}</p>
			</section>

			<section v-else>
				<h1 class="installer-step-title">Gotowe!</h1>
				<p class="installer-step-desc">
					Octra App została zainstalowana w
					<strong class="text-brand">{{ installDir }}</strong
					>.
				</p>
			</section>
		</main>

		<footer class="installer-shell__footer">
			<div>
				<button
					v-if="canGoBack"
					type="button"
					class="installer-btn installer-btn--ghost"
					:disabled="installing"
					@click="goBack"
				>
					Wstecz
				</button>
			</div>
			<div class="flex gap-2">
				<button
					v-if="step !== 'done' && step !== 'progress'"
					type="button"
					class="installer-btn installer-btn--ghost"
					@click="closeInstaller"
				>
					Anuluj
				</button>
				<button
					v-if="step === 'done'"
					type="button"
					class="installer-btn installer-btn--primary"
					@click="closeInstaller"
				>
					Zamknij
				</button>
				<button
					v-else-if="step !== 'progress'"
					type="button"
					class="installer-btn installer-btn--primary"
					:disabled="installing || !installDir"
					@click="goNext"
				>
					{{ step === 'options' ? 'Instaluj' : 'Dalej' }}
				</button>
			</div>
		</footer>
	</div>
</template>

<style scoped>
.mt-6 {
	margin-top: 1.5rem;
}

.flex {
	display: flex;
}

.gap-2 {
	gap: 0.5rem;
}

.text-brand {
	color: var(--color-brand);
}
</style>
