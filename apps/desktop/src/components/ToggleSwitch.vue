<script setup lang="ts">
withDefaults(defineProps<{ modelValue: boolean; disabled?: boolean }>(), { disabled: false })
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()
</script>

<template>
  <button
    type="button"
    role="switch"
    :aria-checked="modelValue"
    :disabled="disabled"
    class="relative h-5.5 w-9.5 shrink-0 rounded-xl transition-colors duration-150"
    :class="disabled ? 'cursor-default opacity-40' : 'cursor-pointer'"
    :style="{
      background: modelValue ? 'var(--vx-accent-grad)' : 'var(--vx-track-off)',
      boxShadow: modelValue
        ? 'inset 0 1px 0 rgba(255,255,255,.25)'
        : 'inset 0 1px 2px rgba(0,0,0,.3)',
    }"
    @click="emit('update:modelValue', !modelValue)"
  >
    <span
      class="absolute top-0.5 h-4.5 w-4.5 rounded-full bg-white shadow-[0_1px_3px_rgba(0,0,0,.4)] transition-[left] duration-150"
      :class="modelValue ? 'left-4.5' : 'left-0.5'"
    />
  </button>
</template>
