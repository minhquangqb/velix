<script setup lang="ts">
import type { VelixPlugin } from '@velix/core'

import GlyphBadge from './GlyphBadge.vue'
import { pluginGlyph, pluginTint } from '../registry'
import { usePlatformsStore } from '../stores/platforms'
import { useProfilesStore } from '../stores/profiles'
import { useUiStore } from '../stores/ui'

const platforms = usePlatformsStore()
const profiles = useProfilesStore()
const ui = useUiStore()

const emit = defineEmits<{ select: [plugin: VelixPlugin] }>()

function badge(pluginId: string) {
  const total = profiles.unreadByPlugin[pluginId] ?? 0
  return total > 9 ? '9+' : String(total)
}
</script>

<template>
  <!-- w-17 = 68px, must match RAIL_WIDTH in src-tauri/src/webviews.rs -->
  <nav
    class="flex w-17 shrink-0 flex-col items-center gap-1.5 pt-3.5 pb-3"
    style="background: var(--vx-rail); border-right: 1px solid var(--vx-hairline)"
  >
    <div
      class="mb-3.5 grid h-9.5 w-9.5 place-items-center rounded-xl font-display text-[19px] font-bold text-white"
      style="
        background: var(--vx-brand-grad);
        box-shadow:
          0 4px 14px rgba(93, 108, 232, 0.35),
          inset 0 1px 0 rgba(255, 255, 255, 0.25);
      "
      title="Velix"
    >
      V
    </div>
    <div class="mb-2 h-px w-7" style="background: var(--vx-hairline)" />

    <button
      v-for="plugin in platforms.plugins"
      :key="plugin.id"
      type="button"
      class="relative grid h-13 w-17 cursor-pointer place-items-center"
      :title="plugin.name"
      @click="emit('select', plugin)"
    >
      <span
        class="absolute top-1/2 left-0 w-[3px] -translate-y-1/2 rounded-r-[3px] bg-white transition-all duration-200"
        :class="platforms.activeId === plugin.id ? 'h-6 opacity-100' : 'h-0 opacity-0'"
      />
      <GlyphBadge
        :glyph="pluginGlyph(plugin)"
        :tint="pluginTint(plugin)"
        :size="42"
        :radius="13"
        :selected="platforms.activeId === plugin.id"
      />
      <span
        v-if="profiles.unreadByPlugin[plugin.id]"
        class="absolute top-px right-[7px] grid h-[17px] min-w-[17px] place-items-center rounded-[9px] px-1 font-display text-[10px] font-bold text-white"
        :style="{
          background: 'var(--vx-danger)',
          border: '2.5px solid var(--vx-rail)',
        }"
      >
        {{ badge(plugin.id) }}
      </span>
    </button>

    <div class="flex-1" />

    <button
      type="button"
      class="grid h-10 w-10 cursor-pointer place-items-center rounded-[11px] hover:bg-(--vx-ghost) hover:text-(--vx-text)!"
      :style="{
        color: ui.view === 'settings' ? 'var(--vx-text)' : 'var(--vx-text-3)',
        background: ui.view === 'settings' ? 'var(--vx-ghost)' : undefined,
      }"
      title="Cài đặt"
      @click="ui.goto('settings')"
    >
      <svg width="17" height="17" viewBox="0 0 17 17" aria-hidden="true">
        <path
          d="M2.5 5h12M2.5 12h12"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
        />
        <circle
          cx="11"
          cy="5"
          r="1.9"
          :fill="'var(--vx-rail)'"
          stroke="currentColor"
          stroke-width="1.5"
        />
        <circle
          cx="6"
          cy="12"
          r="1.9"
          :fill="'var(--vx-rail)'"
          stroke="currentColor"
          stroke-width="1.5"
        />
      </svg>
    </button>
  </nav>
</template>
