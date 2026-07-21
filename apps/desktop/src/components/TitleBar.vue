<script setup lang="ts">
import { ChevronRight, Copy, Minus, Square, X } from '@lucide/vue'
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
        <ChevronRight :size="13" :stroke-width="1.6" class="opacity-30" aria-hidden="true" />
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
        <Minus :size="15" :stroke-width="1.6" aria-hidden="true" />
      </button>
      <button
        type="button"
        :title="ui.maximized ? 'Khôi phục' : 'Phóng to'"
        class="grid w-11.5 cursor-pointer place-items-center hover:bg-(--vx-ghost)"
        :style="{ color: 'var(--vx-text-3)' }"
        @click="windowToggleMaximize()"
      >
        <Square v-if="!ui.maximized" :size="12" :stroke-width="1.8" aria-hidden="true" />
        <Copy v-else :size="13" :stroke-width="1.7" aria-hidden="true" />
      </button>
      <button
        type="button"
        title="Đóng"
        class="grid w-11.5 cursor-pointer place-items-center hover:bg-(--vx-close-hover)! hover:!text-white"
        :style="{ color: 'var(--vx-text-3)' }"
        @click="windowClose()"
      >
        <X :size="15" :stroke-width="1.6" aria-hidden="true" />
      </button>
    </div>
  </header>
</template>
