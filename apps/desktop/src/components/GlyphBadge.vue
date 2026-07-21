<script setup lang="ts">
import { computed } from 'vue'

/**
 * The tinted square used for platform icons and account avatars — by far the
 * most repeated primitive in the design. The alpha suffixes (`1C` fill, `30`
 * ring) are the design's own recipe and are identical in both themes.
 */
const props = withDefaults(
  defineProps<{
    glyph: string
    tint: string
    size?: number
    radius?: number
    /** Solid gradient treatment used for the selected rail icon. */
    selected?: boolean
    dimmed?: boolean
  }>(),
  { size: 34, radius: 10, selected: false, dimmed: false },
)

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
    {{ glyph }}
    <slot />
  </div>
</template>
