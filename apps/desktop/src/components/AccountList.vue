<script setup lang="ts">
import { computed } from 'vue'
import type { Profile } from '@velix/core'

import GlyphBadge from './GlyphBadge.vue'
import { pluginTint } from '../registry'
import { usePlatformsStore } from '../stores/platforms'
import { useProfilesStore } from '../stores/profiles'
import { useTabsStore } from '../stores/tabs'
import { useUiStore } from '../stores/ui'

const platforms = usePlatformsStore()
const profiles = useProfilesStore()
const tabs = useTabsStore()
const ui = useUiStore()

const emit = defineEmits<{ select: [profile: Profile] }>()

const accounts = computed(() => profiles.byPlugin(platforms.activeId))
const tint = computed(() => pluginTint(platforms.active))
const platformName = computed(() => platforms.active?.name ?? '')

const countLabel = computed(() =>
  accounts.value.length > 0 ? `${accounts.value.length} tài khoản` : 'Trống',
)

function initial(name: string) {
  return name.trim().charAt(0).toUpperCase()
}

function subtitle(profile: Profile) {
  if (tabs.activeProfileId === profile.id) return 'Đang xem'
  return tabs.isOpen(profile.id) ? 'Phiên hoạt động' : 'Chưa mở'
}
</script>

<template>
  <!-- w-62 = 248px, must match ACCOUNTS_WIDTH in src-tauri/src/webviews.rs -->
  <aside
    class="flex w-62 shrink-0 flex-col"
    style="background: var(--vx-accounts); border-right: 1px solid var(--vx-hairline)"
  >
    <div class="flex items-center justify-between px-4 pt-4.5 pb-3">
      <h2 class="font-display text-[15px] font-semibold tracking-[0.01em]">{{ platformName }}</h2>
      <button
        type="button"
        class="grid h-6.5 w-6.5 cursor-pointer place-items-center rounded-lg hover:bg-(--vx-ghost-hover)! hover:text-(--vx-text)!"
        :style="{ color: 'var(--vx-text-3)', background: 'var(--vx-nav-hover)' }"
        title="Thêm tài khoản"
        @click="ui.goto('add-account', platforms.activeId)"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
          <path
            d="M6 1.5v9M1.5 6h9"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </div>

    <div
      class="px-4 pb-2 text-[10.5px] font-semibold tracking-[0.1em] uppercase"
      :style="{ color: 'var(--vx-text-4)' }"
    >
      {{ countLabel }}
    </div>

    <div class="flex flex-1 flex-col gap-[3px] overflow-y-auto px-2">
      <button
        v-for="profile in accounts"
        :key="profile.id"
        type="button"
        class="flex cursor-pointer items-center gap-2.75 rounded-[10px] px-2.5 py-2.25 text-left transition-colors duration-100"
        :style="{
          background: tabs.activeProfileId === profile.id ? 'rgba(124,140,248,.09)' : 'transparent',
          boxShadow:
            tabs.activeProfileId === profile.id ? 'inset 0 0 0 1px rgba(124,140,248,.22)' : 'none',
        }"
        @click="emit('select', profile)"
      >
        <GlyphBadge
          :glyph="initial(profile.name)"
          :tint="tint"
          :size="34"
          :radius="10"
          :dimmed="!tabs.isOpen(profile.id)"
        >
          <span
            class="absolute -right-0.5 -bottom-0.5 h-2.25 w-2.25 rounded-full"
            :style="{
              border: '2px solid var(--vx-pip-ring)',
              background: tabs.isOpen(profile.id) ? 'var(--vx-success)' : 'var(--vx-text-5)',
            }"
          />
        </GlyphBadge>

        <span class="min-w-0 flex-1">
          <span class="block truncate text-[13px] font-semibold tracking-[0.01em]">
            {{ profile.name }}
          </span>
          <span
            class="block truncate text-[11px]"
            :style="{
              color:
                tabs.activeProfileId === profile.id ? 'var(--vx-accent-text)' : 'var(--vx-text-4)',
              marginTop: '1px',
            }"
          >
            {{ subtitle(profile) }}
          </span>
        </span>

        <span
          v-if="profiles.unread[profile.id]"
          class="grid h-5 min-w-5 shrink-0 place-items-center rounded-[10px] px-1.5 font-display text-[11px] font-bold"
          :style="{ background: 'rgba(124,140,248,.16)', color: 'var(--vx-accent-text)' }"
        >
          {{ profiles.unread[profile.id] }}
        </span>

        <span
          v-if="profile.muted"
          class="shrink-0 rounded-[5px] px-1.25 py-0.5 text-[9.5px] font-semibold tracking-[0.06em]"
          :style="{ color: 'var(--vx-text-4)', border: '1px solid var(--vx-card-border)' }"
          title="Đã tắt thông báo"
        >
          IM LẶNG
        </span>
      </button>

      <div v-if="!accounts.length" class="vx-fade px-4.5 py-9 text-center">
        <div
          class="mx-auto mb-3.5 grid h-12 w-12 place-items-center rounded-[14px]"
          style="
            background: rgba(124, 140, 248, 0.1);
            border: 1px dashed rgba(124, 140, 248, 0.35);
            color: #8a97fa;
          "
        >
          <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
            <path
              d="M8 2.5v11M2.5 8h11"
              stroke="currentColor"
              stroke-width="1.6"
              stroke-linecap="round"
            />
          </svg>
        </div>
        <p class="mb-1.25 text-[13.5px] font-bold">Chưa có tài khoản</p>
        <p class="mb-4 text-[12px] leading-relaxed" :style="{ color: 'var(--vx-text-3)' }">
          Thêm tài khoản {{ platformName }} đầu tiên — bạn đăng nhập trực tiếp trên trang của họ,
          Velix không giữ mật khẩu.
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

    <button
      type="button"
      class="m-2.5 flex cursor-pointer items-center gap-2.25 rounded-[10px] px-3 py-2.25 text-[12.5px] hover:text-(--vx-text-2)!"
      :style="{
        color: 'var(--vx-text-4)',
        background: 'var(--vx-well)',
        border: '1px solid var(--vx-card-border)',
      }"
      @click="ui.goto('accounts')"
    >
      <svg width="13" height="13" viewBox="0 0 13 13" aria-hidden="true">
        <path
          d="M2 3.5h9M2 6.5h9M2 9.5h5"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
        />
      </svg>
      <span>Quản lý tài khoản</span>
    </button>
  </aside>
</template>
