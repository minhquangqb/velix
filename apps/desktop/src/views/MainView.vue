<script setup lang="ts">
import { computed } from 'vue'
import type { Profile, VelixPlugin } from '@velix/core'

import { Plus } from '@lucide/vue'

import AccountList from '../components/AccountList.vue'
import PlatformRail from '../components/PlatformRail.vue'
import TitleBar from '../components/TitleBar.vue'
import { usePlatformsStore } from '../stores/platforms'
import { useProfilesStore } from '../stores/profiles'
import { useTabsStore } from '../stores/tabs'
import { useUiStore } from '../stores/ui'

const platforms = usePlatformsStore()
const profiles = useProfilesStore()
const tabs = useTabsStore()
const ui = useUiStore()

const emit = defineEmits<{ error: [message: string] }>()

const platformName = computed(() => platforms.active?.name ?? '')
const accounts = computed(() => profiles.byPlugin(platforms.activeId))
const activeProfile = computed(() =>
  tabs.activeProfileId ? profiles.find(tabs.activeProfileId) : undefined,
)

const crumbs = computed(() =>
  activeProfile.value ? [platformName.value, activeProfile.value.name] : [platformName.value],
)

async function run(action: () => Promise<unknown>) {
  try {
    await action()
  } catch (e) {
    emit('error', String(e))
  }
}

/** Switching platform resumes that platform's last account, per the design. */
function selectPlatform(plugin: VelixPlugin) {
  platforms.select(plugin.id)
  const remembered = tabs.lastByPlugin[plugin.id]
  const next =
    (remembered ? profiles.find(remembered) : undefined) ?? profiles.byPlugin(plugin.id)[0]
  if (next) run(() => tabs.open(next))
}

function selectAccount(profile: Profile) {
  run(() => tabs.open(profile))
}
</script>

<template>
  <div class="flex h-full">
    <PlatformRail @select="selectPlatform" />
    <AccountList @select="selectAccount" />

    <!-- Workspace column. Its content area is the rectangle platform webviews
         are positioned into by webviews::relayout, so everything drawn here is
         a placeholder that a live webview covers. -->
    <div class="flex min-w-0 flex-1 flex-col" style="background: var(--vx-workspace)">
      <TitleBar :crumbs="crumbs">
        <template #status>
          <span
            v-if="activeProfile"
            class="ml-2 inline-flex items-center gap-1.25 rounded-full px-2 py-[2.5px] text-[10.5px] font-bold tracking-[0.05em]"
            :style="{
              color: tabs.opening ? 'var(--vx-warning)' : 'var(--vx-success)',
              background: tabs.opening ? 'rgba(245,192,68,.1)' : 'rgba(61,214,140,.09)',
            }"
          >
            <span
              class="h-1.25 w-1.25 rounded-full"
              :style="{ background: tabs.opening ? 'var(--vx-warning)' : 'var(--vx-success)' }"
            />
            {{ tabs.opening ? 'đang tải' : 'trực tiếp' }}
          </span>
        </template>
      </TitleBar>

      <div class="relative min-h-0 flex-1">
        <div v-if="tabs.opening" class="absolute inset-0 grid place-items-center">
          <div class="vx-fade text-center">
            <div
              class="vx-spin mx-auto mb-4 h-7.5 w-7.5 rounded-full"
              style="border: 2.5px solid var(--vx-card-border); border-top-color: var(--vx-accent)"
            />
            <p class="text-[13px]" :style="{ color: 'var(--vx-text-3)' }">Đang mở phiên…</p>
          </div>
        </div>

        <div v-else-if="!accounts.length" class="absolute inset-0 grid place-items-center">
          <div class="vx-fade max-w-75 text-center">
            <div
              class="mx-auto mb-3.5 grid h-12 w-12 place-items-center rounded-[14px]"
              style="
                background: rgba(124, 140, 248, 0.1);
                border: 1px dashed rgba(124, 140, 248, 0.4);
                color: #6474ee;
              "
            >
              <Plus :size="17" :stroke-width="1.9" aria-hidden="true" />
            </div>
            <p class="mb-1.25 text-[14px] font-bold">Chưa có tài khoản {{ platformName }}</p>
            <p class="mb-4 text-[12.5px] leading-relaxed" :style="{ color: 'var(--vx-text-3)' }">
              Thêm tài khoản đầu tiên để bắt đầu — vùng này sẽ là trang web của
              {{ platformName }}.
            </p>
            <button
              type="button"
              class="cursor-pointer rounded-[9px] px-4 py-2 text-[12.5px] font-bold text-white"
              style="
                background: var(--vx-accent-grad);
                box-shadow:
                  0 3px 10px rgba(100, 116, 238, 0.3),
                  inset 0 1px 0 rgba(255, 255, 255, 0.25);
              "
              @click="ui.goto('add-account', platforms.activeId)"
            >
              Thêm tài khoản
            </button>
          </div>
        </div>

        <div
          v-else-if="!activeProfile"
          class="absolute inset-0 grid place-items-center"
          :style="{
            backgroundImage: 'radial-gradient(var(--vx-dots) 1px, transparent 1px)',
            backgroundSize: '22px 22px',
          }"
        >
          <p class="vx-fade font-mono text-[13px]" :style="{ color: 'var(--vx-text-3)' }">
            Chọn một tài khoản {{ platformName }} để mở
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
