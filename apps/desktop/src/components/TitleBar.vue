<script setup lang="ts">
import { windowClose, windowMinimize, windowStartDrag, windowToggleMaximize } from '@velix/core'

import { useUiStore } from '../stores/ui'

defineProps<{
  /** Breadcrumb trail after the "Velix" root, e.g. ["Messenger", "Cá nhân"]. */
  crumbs: string[]
}>()

const ui = useUiStore()

/**
 * The window is frameless, so the bar itself is the drag handle. A double click
 * must still maximise rather than start a drag, and `detail` is the only signal
 * available before `start_dragging` hands the gesture to the OS.
 */
function onDragMouseDown(event: MouseEvent) {
  if (event.button !== 0) return
  if (event.detail === 2) {
    windowToggleMaximize()
    return
  }
  windowStartDrag()
}
</script>

<template>
  <header
    class="flex h-11 shrink-0 items-center pl-4.5"
    style="border-bottom: 1px solid var(--vx-hairline); background: var(--vx-card)"
  >
    <div class="flex h-full flex-1 items-center gap-2 text-[12.5px]" @mousedown="onDragMouseDown">
      <span :style="{ color: 'var(--vx-text-3)' }">Velix</span>
      <template v-for="(crumb, index) in crumbs" :key="index">
        <svg width="5" height="8" viewBox="0 0 5 8" class="opacity-30" aria-hidden="true">
          <path
            d="M1 1l3 3-3 3"
            stroke="currentColor"
            stroke-width="1.3"
            fill="none"
            stroke-linecap="round"
          />
        </svg>
        <span :class="index === crumbs.length - 1 ? 'font-bold' : ''">{{ crumb }}</span>
      </template>
      <slot name="status" />
    </div>

    <div class="flex h-11 shrink-0">
      <button
        type="button"
        title="Thu nhỏ"
        class="grid w-11.5 cursor-pointer place-items-center hover:bg-(--vx-ghost)"
        :style="{ color: 'var(--vx-text-3)' }"
        @click="windowMinimize()"
      >
        <svg width="11" height="11" viewBox="0 0 11 11" aria-hidden="true">
          <path d="M1.5 5.5h8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
        </svg>
      </button>
      <button
        type="button"
        :title="ui.maximized ? 'Khôi phục' : 'Phóng to'"
        class="grid w-11.5 cursor-pointer place-items-center hover:bg-(--vx-ghost)"
        :style="{ color: 'var(--vx-text-3)' }"
        @click="windowToggleMaximize()"
      >
        <svg v-if="!ui.maximized" width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <rect
            x="1.2"
            y="1.2"
            width="7.6"
            height="7.6"
            rx="1"
            stroke="currentColor"
            stroke-width="1.3"
            fill="none"
          />
        </svg>
        <svg v-else width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <rect
            x="1.2"
            y="3"
            width="5.8"
            height="5.8"
            rx="1"
            stroke="currentColor"
            stroke-width="1.3"
            fill="none"
          />
          <path
            d="M3.4 3V1.2h5.4V6.6H7"
            stroke="currentColor"
            stroke-width="1.3"
            fill="none"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
      <button
        type="button"
        title="Đóng"
        class="grid w-11.5 cursor-pointer place-items-center hover:bg-(--vx-close-hover)! hover:!text-white"
        :style="{ color: 'var(--vx-text-3)' }"
        @click="windowClose()"
      >
        <svg width="11" height="11" viewBox="0 0 11 11" aria-hidden="true">
          <path
            d="M1.5 1.5l8 8M9.5 1.5l-8 8"
            stroke="currentColor"
            stroke-width="1.3"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </div>
  </header>
</template>
