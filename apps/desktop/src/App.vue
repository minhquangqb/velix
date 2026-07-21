<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import type { Profile } from '@velix/core'
import { PLUGINS } from './registry'
import { useProfilesStore } from './stores/profiles'

const store = useProfilesStore()
const newNames = ref<Record<string, string>>({})
const error = ref<string | null>(null)

let unwatchUnread: (() => void) | null = null

onMounted(() => {
  store.load().catch((e) => (error.value = String(e)))
  store
    .watchUnread()
    .then((unwatch) => (unwatchUnread = unwatch))
    .catch((e) => (error.value = String(e)))
})

onUnmounted(() => unwatchUnread?.())

async function run(action: () => Promise<unknown>) {
  error.value = null
  try {
    await action()
  } catch (e) {
    error.value = String(e)
  }
}

function addProfile(pluginId: string) {
  const name = (newNames.value[pluginId] ?? '').trim()
  if (!name) return
  run(async () => {
    await store.create(pluginId, name)
    newNames.value[pluginId] = ''
  })
}

function openProfile(profile: Profile) {
  const plugin = PLUGINS.find((p) => p.id === profile.pluginId)
  if (!plugin) return
  run(() => store.open(profile, plugin.url))
}
</script>

<template>
  <div class="flex h-screen bg-zinc-900 text-zinc-100">
    <!-- w-65 = 260px, must match SIDEBAR_WIDTH in src-tauri/src/webviews.rs -->
    <aside class="flex w-65 shrink-0 flex-col border-r border-zinc-800">
      <header class="px-4 py-3">
        <h1 class="text-lg font-bold tracking-tight">Velix</h1>
      </header>

      <div class="flex-1 space-y-6 overflow-y-auto px-4 pb-4">
        <section v-for="plugin in PLUGINS" :key="plugin.id">
          <h2 class="mb-2 text-xs font-semibold tracking-wide text-zinc-400 uppercase">
            {{ plugin.name }}
          </h2>

          <ul class="space-y-1">
            <li
              v-for="profile in store.byPlugin(plugin.id)"
              :key="profile.id"
              class="group flex items-center gap-1 rounded-md px-2 py-1.5 text-sm hover:bg-zinc-800"
              :class="{ 'bg-zinc-800': store.activeProfileId === profile.id }"
            >
              <button class="flex-1 cursor-pointer text-left" @click="openProfile(profile)">
                {{ profile.name }}
                <span
                  v-if="store.openLabels[profile.id]"
                  class="ml-1 inline-block h-1.5 w-1.5 rounded-full bg-emerald-400"
                  title="Open"
                />
              </button>
              <span
                v-if="store.unread[profile.id]"
                class="grid h-4.5 min-w-4.5 place-items-center rounded-full bg-indigo-500/20 px-1.25 text-[10.5px] font-bold text-indigo-300"
                :title="`${store.unread[profile.id]} tin chưa đọc`"
              >
                {{ store.unread[profile.id] }}
              </span>
              <button
                v-if="store.openLabels[profile.id]"
                class="hidden cursor-pointer text-zinc-500 hover:text-zinc-200 group-hover:block"
                title="Close webview"
                @click="run(() => store.close(profile.id))"
              >
                &times;
              </button>
              <button
                class="hidden cursor-pointer text-zinc-500 hover:text-red-400 group-hover:block"
                title="Delete profile"
                @click="run(() => store.remove(profile.id))"
              >
                &#128465;
              </button>
            </li>
          </ul>

          <form class="mt-2 flex gap-1" @submit.prevent="addProfile(plugin.id)">
            <input
              v-model="newNames[plugin.id]"
              type="text"
              placeholder="New profile name"
              class="w-full rounded-md border border-zinc-700 bg-zinc-800 px-2 py-1 text-sm placeholder-zinc-500 focus:border-zinc-500 focus:outline-none"
            />
            <button
              type="submit"
              class="cursor-pointer rounded-md border border-zinc-700 px-2 text-sm hover:bg-zinc-800"
            >
              +
            </button>
          </form>
        </section>
      </div>

      <p v-if="error" class="border-t border-zinc-800 px-4 py-2 text-xs wrap-break-word text-red-400">
        {{ error }}
      </p>
    </aside>

    <!-- Placeholder only: open webviews are native children overlaid on this area -->
    <main class="flex flex-1 items-center justify-center">
      <p class="text-sm text-zinc-500">Select a profile to open its workspace</p>
    </main>
  </div>
</template>
