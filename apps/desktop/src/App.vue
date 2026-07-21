<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'

import AccountsView from './views/AccountsView.vue'
import AddAccountView from './views/AddAccountView.vue'
import FirstRunView from './views/FirstRunView.vue'
import MainView from './views/MainView.vue'
import SettingsView from './views/SettingsView.vue'
import { useProfilesStore } from './stores/profiles'
import { useSettingsStore } from './stores/settings'
import { useUiStore } from './stores/ui'

const profiles = useProfilesStore()
const settings = useSettingsStore()
const ui = useUiStore()

const error = ref<string | null>(null)
const unsubscribes: (() => void)[] = []

/**
 * The welcome screen takes over until the first account exists, and only while
 * the user has not navigated away from it — otherwise the add-account wizard
 * would be yanked back to it on every render.
 */
const view = computed(() => {
  if (!profiles.profiles.length && !ui.firstRunDone && ui.view === 'main') return 'first-run'
  return ui.view
})

onMounted(async () => {
  try {
    await Promise.all([
      profiles.load(),
      settings.load().then((off) => unsubscribes.push(off)),
      profiles.watchUnread().then((off) => unsubscribes.push(off)),
      ui.watchWindow().then((off) => unsubscribes.push(off)),
      ui.watchNavigate().then((off) => unsubscribes.push(off)),
    ])
  } catch (e) {
    error.value = String(e)
  }
})

onUnmounted(() => unsubscribes.forEach((off) => off()))
</script>

<template>
  <div class="h-full overflow-hidden" style="background: var(--vx-window)">
    <FirstRunView v-if="view === 'first-run'" />
    <SettingsView v-else-if="view === 'settings'" @error="error = $event" />
    <AccountsView v-else-if="view === 'accounts'" @error="error = $event" />
    <AddAccountView v-else-if="view === 'add-account'" @error="error = $event" />
    <MainView v-else @error="error = $event" />

    <!-- Errors surface as a dismissible toast: most come from IPC, where the
         only alternative is a silently dropped promise rejection. -->
    <div
      v-if="error"
      class="vx-fade absolute right-4.5 bottom-4.5 z-20 w-78 rounded-[13px] px-4 py-3.75"
      :style="{
        background: 'var(--vx-panel)',
        border: '1px solid rgba(242,85,90,.3)',
        boxShadow: 'var(--vx-shadow-toast)',
      }"
    >
      <p class="mb-2 text-[13px] font-bold" :style="{ color: 'var(--vx-danger)' }">Có lỗi xảy ra</p>
      <p
        class="mb-3 text-[12px] leading-relaxed break-words"
        :style="{ color: 'var(--vx-text-2)' }"
      >
        {{ error }}
      </p>
      <button
        type="button"
        class="cursor-pointer rounded-lg px-3.5 py-1.75 text-[12px] font-semibold"
        :style="{
          background: 'var(--vx-ghost)',
          border: '1px solid var(--vx-input-border)',
          color: 'var(--vx-text-2)',
        }"
        @click="error = null"
      >
        Đóng
      </button>
    </div>
  </div>
</template>
