<script setup lang="ts">
import { computed, type Component } from 'vue'

/**
 * The tinted square used for platform marks and account avatars — by far the
 * most repeated primitive in the design. The alpha suffixes (`1C` fill, `30`
 * ring) are the design's own recipe and are identical in both themes.
 *
 * Contents, in precedence order: a filled brand path (`path`), a stroked icon
 * component (`icon`), or the `glyph` letter the design falls back to.
 */
const props = withDefaults(
  defineProps<{
    glyph?: string
    /** Filled 24x24 brand path, see brands.ts. */
    path?: string
    /** Stroked icon component, for platforms with no usable brand mark. */
    icon?: Component
    tint: string
    size?: number
    radius?: number
    /** Solid gradient treatment used for the selected rail icon. */
    selected?: boolean
    dimmed?: boolean
  }>(),
  {
    glyph: '',
    path: undefined,
    icon: undefined,
    size: 34,
    radius: 10,
    selected: false,
    dimmed: false,
  },
)

/** Marks read best a little smaller than the letter they replace. */
const markSize = computed(() => Math.round(props.size * 0.5))

const style = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
  borderRadius: `${props.radius}px`,
  fontSize: `${Math.round(props.size * 0.41)}px`,
  color: props.selected ? '#fff' : props.tint,
  background: props.selected
    ? `linear-gradient(150deg, ${props.tint}, ${props.tint}BB)`
    : `${props.tint}1C`,
  boxShadow: props.selected
    ? `0 4px 14px ${props.tint}4D, inset 0 1px 0 rgba(255,255,255,.3)`
    : `inset 0 0 0 1px ${props.tint}30`,
  filter: props.dimmed ? 'grayscale(1) opacity(.6)' : undefined,
}))
</script>

<template>
  <div
    class="relative grid shrink-0 place-items-center font-display font-bold transition-all duration-150"
    :style="style"
  >
    <svg
      v-if="path"
      :width="markSize"
      :height="markSize"
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
    >
      <path :d="path" />
    </svg>
    <component :is="icon" v-else-if="icon" :size="markSize" :stroke-width="2" aria-hidden="true" />
    <template v-else>{{ glyph }}</template>
    <slot />
  </div>
</template>
