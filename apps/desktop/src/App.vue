<script setup lang="ts">
import { useAppStore } from './stores/app'

const app = useAppStore()
</script>

<template>
  <main class="flex h-screen flex-col items-center justify-center bg-zinc-900 text-zinc-100">
    <h1 class="text-4xl font-bold tracking-tight">{{ app.name }}</h1>
    <p class="mt-2 text-sm text-zinc-400">v{{ app.version }} — scaffold ready</p>
  </main>
</template>
