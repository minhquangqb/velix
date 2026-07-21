<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import {
  getSettings,
  listUnread,
  onSettings,
  onUnread,
  openAccount,
  openSettings,
  quitApp,
  setQuiet,
  showMainWindow,
  type UnreadEntry,
} from '@velix/core'
import { PLUGINS, pluginTint } from '../registry'

/**
 * Row heights are load-bearing: the popup window is sized in Rust
 * (src-tauri/src/tray.rs) from the unread count, so any change here must match
 * the constants there.
 */
const MAX_VISIBLE_ACCOUNTS = 6

const entries = ref<UnreadEntry[]>([])
const quiet = ref(false)
let unlisten: (() => void) | null = null

const visible = computed(() => entries.value.slice(0, MAX_VISIBLE_ACCOUNTS))

function pluginName(pluginId: string) {
  return PLUGINS.find((p) => p.id === pluginId)?.name ?? pluginId
}

/** Brand tint comes from the plugin manifest, same as the shell's avatars. */
function tint(pluginId: string) {
  return pluginTint(PLUGINS.find((p) => p.id === pluginId))
}

let unlistenSettings: (() => void) | null = null

onMounted(async () => {
  const [initial, settings] = await Promise.all([listUnread(), getSettings()])
  entries.value = initial
  quiet.value = settings.quiet
  unlisten = await onUnread((next) => (entries.value = next))
  // Quiet mode can also be toggled from the settings page.
  unlistenSettings = await onSettings((next) => (quiet.value = next.quiet))
})

onUnmounted(() => {
  unlisten?.()
  unlistenSettings?.()
})

async function toggleQuiet() {
  quiet.value = (await setQuiet(!quiet.value)).quiet
}

async function showAccount(entry: UnreadEntry) {
  // Rust reveals the window and hands the switch to the shell, so its sidebar
  // selection follows along instead of going stale.
  await openAccount(entry.profileId)
}

async function showSettings() {
  // Settings lives in the main window; Rust reveals it and pushes the view.
  await openSettings()
}
</script>

<template>
  <!-- p-4 is the transparent gutter for the panel shadow (SHADOW_MARGIN in tray.rs) -->
  <div class="h-screen w-screen p-4 select-none">
    <div
      class="flex h-full flex-col rounded-xl p-1.5"
      :style="{
        background: 'var(--vx-panel)',
        border: '1px solid var(--vx-input-border)',
        color: 'var(--vx-text)',
        boxShadow: 'var(--vx-shadow-panel)',
      }"
    >
      <button
        class="flex h-10 shrink-0 cursor-pointer items-center gap-2.5 rounded-lg px-2.5 hover:bg-[rgba(124,140,248,0.1)]"
        @click="showMainWindow()"
      >
        <span
          class="grid h-5.5 w-5.5 place-items-center rounded-[7px] text-[11px] font-bold text-white"
          :style="{ background: 'var(--vx-brand-grad)' }"
        >
          V
        </span>
        <span class="flex-1 text-left text-[13px] font-bold">Mở Velix</span>
        <kbd
          class="rounded-[5px] border border-b-2 px-1.25 py-px font-mono text-[10px]"
          :style="{
            color: 'var(--vx-text-4)',
            borderColor: 'var(--vx-input-border)',
            background: 'var(--vx-ghost)',
          }"
        >
          Ctrl Shift V
        </kbd>
      </button>

      <div class="my-1.25 h-px shrink-0" :style="{ background: 'var(--vx-hairline)' }" />

      <template v-if="visible.length">
        <div
          class="h-5.75 shrink-0 px-2.5 pt-1.25 text-[10px] font-bold tracking-[0.09em] uppercase"
          :style="{ color: 'var(--vx-text-4)' }"
        >
          Có tin mới
        </div>
        <div class="min-h-0 overflow-y-auto">
          <button
            v-for="entry in visible"
            :key="entry.label"
            class="flex h-9.5 w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 hover:bg-(--vx-ghost)"
            @click="showAccount(entry)"
          >
            <span
              class="grid h-6.5 w-6.5 shrink-0 place-items-center rounded-lg text-[11px] font-bold"
              :style="{
                color: tint(entry.pluginId),
                background: `color-mix(in srgb, ${tint(entry.pluginId)} 11%, transparent)`,
                boxShadow: `inset 0 0 0 1px color-mix(in srgb, ${tint(entry.pluginId)} 19%, transparent)`,
              }"
            >
              {{ pluginName(entry.pluginId).charAt(0).toUpperCase() }}
            </span>
            <span class="min-w-0 flex-1 text-left">
              <span class="block truncate text-[12.5px] leading-tight font-semibold">
                {{ entry.name }}
              </span>
              <span
                class="block text-[10.5px] leading-tight"
                :style="{ color: 'var(--vx-text-4)' }"
              >
                {{ pluginName(entry.pluginId) }}
              </span>
            </span>
            <span
              class="grid h-4.5 min-w-4.5 shrink-0 place-items-center rounded-full bg-[rgba(124,140,248,0.16)] px-1.25 text-[10.5px] font-bold"
              :style="{ color: 'var(--vx-accent-text)' }"
            >
              {{ entry.count }}
            </span>
          </button>
        </div>
        <div class="my-1.25 h-px shrink-0" :style="{ background: 'var(--vx-hairline)' }" />
      </template>

      <button
        class="flex h-8.5 shrink-0 cursor-pointer items-center gap-2.5 rounded-lg px-2.5 hover:bg-(--vx-ghost)"
        @click="toggleQuiet()"
      >
        <span class="grid w-5.5 place-items-center" :style="{ color: 'var(--vx-text-2)' }">
          <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
            <path
              d="M4 5.5a3 3 0 016 0c0 2.4.9 3 .9 3H3.1s.9-.6.9-3zM5.9 10.8a1.2 1.2 0 002.2 0"
              stroke="currentColor"
              stroke-width="1.2"
              fill="none"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </span>
        <span class="flex-1 text-left text-[12.5px] font-medium">Tạm tắt thông báo</span>
        <span
          class="relative h-4.5 w-8 shrink-0 rounded-full transition-colors"
          :style="{ background: quiet ? '#6474EE' : 'var(--vx-track-off)' }"
        >
          <span
            class="absolute top-0.5 h-3.5 w-3.5 rounded-full bg-white transition-[left]"
            :class="quiet ? 'left-4' : 'left-0.5'"
          />
        </span>
      </button>

      <button
        class="flex h-8.5 shrink-0 cursor-pointer items-center gap-2.5 rounded-lg px-2.5 hover:bg-(--vx-ghost)"
        @click="showSettings()"
      >
        <span class="grid w-5.5 place-items-center" :style="{ color: 'var(--vx-text-2)' }">
          <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
            <path
              d="M2 4h10M2 10h10"
              stroke="currentColor"
              stroke-width="1.2"
              stroke-linecap="round"
            />
            <circle
              cx="9"
              cy="4"
              r="1.6"
              fill="var(--vx-panel)"
              stroke="currentColor"
              stroke-width="1.2"
            />
            <circle
              cx="5"
              cy="10"
              r="1.6"
              fill="var(--vx-panel)"
              stroke="currentColor"
              stroke-width="1.2"
            />
          </svg>
        </span>
        <span class="text-[12.5px] font-medium">Cài đặt…</span>
      </button>

      <div class="my-1.25 h-px shrink-0" :style="{ background: 'var(--vx-hairline)' }" />

      <button
        class="group flex h-8.5 shrink-0 cursor-pointer items-center gap-2.5 rounded-lg px-2.5 hover:bg-[rgba(242,85,90,0.1)] hover:text-(--vx-danger)!"
        :style="{ color: 'var(--vx-text-2)' }"
        @click="quitApp()"
      >
        <span class="grid w-5.5 place-items-center">
          <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
            <path
              d="M5.5 2H3a1 1 0 00-1 1v8a1 1 0 001 1h2.5M9 4.5L11.5 7 9 9.5M11.5 7H5.5"
              stroke="currentColor"
              stroke-width="1.2"
              fill="none"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </span>
        <span class="text-[12.5px] font-medium">Thoát hẳn Velix</span>
      </button>
    </div>
  </div>
</template>
