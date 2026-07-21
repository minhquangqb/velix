<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Theme } from '@velix/core'

import GlyphBadge from '../components/GlyphBadge.vue'
import TitleBar from '../components/TitleBar.vue'
import ToggleSwitch from '../components/ToggleSwitch.vue'
import { findPlugin, pluginTint } from '../registry'
import { useProfilesStore } from '../stores/profiles'
import { useSettingsStore } from '../stores/settings'

type Section = 'appearance' | 'behaviour' | 'notifications'

const settings = useSettingsStore()
const profiles = useProfilesStore()

const emit = defineEmits<{ error: [message: string] }>()

const section = ref<Section>('appearance')

const NAV: { id: Section; label: string }[] = [
  { id: 'appearance', label: 'Giao diện' },
  { id: 'behaviour', label: 'Hành vi' },
  { id: 'notifications', label: 'Thông báo' },
]

const THEMES: { id: Theme; label: string; swatch: string; bar: string }[] = [
  { id: 'dark', label: 'Tối', swatch: '#14151B', bar: '#2A2C38' },
  { id: 'light', label: 'Sáng', swatch: '#F2F3F7', bar: '#D9DBE4' },
  {
    id: 'system',
    label: 'Theo hệ thống',
    swatch: 'linear-gradient(100deg, #14151B 50%, #F2F3F7 50%)',
    bar: '#8A8C98',
  },
]

const notifiable = computed(() => profiles.profiles)

async function run(action: () => Promise<unknown>) {
  try {
    await action()
  } catch (e) {
    emit('error', String(e))
  }
}

function platformName(pluginId: string) {
  return findPlugin(pluginId)?.name ?? pluginId
}
</script>

<template>
  <div class="flex h-full flex-col">
    <TitleBar :crumbs="['Cài đặt']" />

    <div class="flex min-h-0 flex-1">
      <nav
        class="flex w-52.5 shrink-0 flex-col gap-0.5 px-2.5 py-4.5"
        style="border-right: 1px solid var(--vx-hairline)"
      >
        <button
          v-for="item in NAV"
          :key="item.id"
          type="button"
          class="cursor-pointer rounded-[9px] px-3.25 py-2.25 text-left text-[13px]"
          :style="
            section === item.id
              ? {
                  fontWeight: 700,
                  color: 'var(--vx-text)',
                  background: 'rgba(124,140,248,.09)',
                  boxShadow: 'inset 0 0 0 1px rgba(124,140,248,.22)',
                }
              : { fontWeight: 500, color: 'var(--vx-text-2)' }
          "
          @click="section = item.id"
        >
          {{ item.label }}
        </button>
      </nav>

      <div class="min-h-0 flex-1 overflow-y-auto px-8.5 py-6.5">
        <div class="vx-fade w-140 max-w-full">
          <!-- Giao diện -->
          <section v-if="section === 'appearance'">
            <h2 class="mb-5 font-display text-[19px] font-bold">Giao diện</h2>
            <p class="mb-2.5 text-[13px] font-semibold">Chủ đề</p>
            <div class="mb-2 grid grid-cols-3 gap-2.5">
              <button
                v-for="option in THEMES"
                :key="option.id"
                type="button"
                class="cursor-pointer overflow-hidden rounded-xl text-left transition-all duration-100"
                :style="{
                  background: 'var(--vx-card)',
                  border:
                    settings.settings.theme === option.id
                      ? '1px solid rgba(124,140,248,.5)'
                      : '1px solid var(--vx-input-border)',
                  boxShadow:
                    settings.settings.theme === option.id
                      ? '0 0 0 3px rgba(124,140,248,.12)'
                      : 'none',
                }"
                @click="run(() => settings.setTheme(option.id))"
              >
                <span class="relative block h-14" :style="{ background: option.swatch }">
                  <span
                    class="absolute top-2.5 left-2.5 h-1.75 w-8.5 rounded"
                    :style="{ background: option.bar }"
                  />
                </span>
                <span class="flex items-center gap-1.5 px-2.75 py-2.25">
                  <span
                    class="h-4 w-4 shrink-0 rounded-full transition-all duration-100"
                    :style="
                      settings.settings.theme === option.id
                        ? { border: '5px solid var(--vx-accent)', background: '#fff' }
                        : { border: '1.5px solid var(--vx-input-border)' }
                    "
                  />
                  <span class="text-[12.5px] font-semibold">{{ option.label }}</span>
                </span>
              </button>
            </div>
            <p class="text-[12px] leading-relaxed" :style="{ color: 'var(--vx-text-4)' }">
              "Theo hệ thống" đổi theo cài đặt sáng/tối của Windows.
            </p>
          </section>

          <!-- Hành vi -->
          <section v-else-if="section === 'behaviour'">
            <h2 class="mb-5 font-display text-[19px] font-bold">Hành vi</h2>
            <div
              class="overflow-hidden rounded-[14px]"
              style="background: var(--vx-card); border: 1px solid var(--vx-card-border)"
            >
              <div
                class="flex items-center gap-3.5 px-4 py-3.75"
                style="border-bottom: 1px solid var(--vx-divider)"
              >
                <div class="flex-1">
                  <p class="text-[13.5px] font-semibold">Khởi động cùng Windows</p>
                  <p class="mt-0.5 text-[12px]" :style="{ color: 'var(--vx-text-3)' }">
                    Mở Velix ẩn trong khay hệ thống khi bật máy.
                  </p>
                </div>
                <ToggleSwitch
                  :model-value="settings.settings.autostart"
                  @update:model-value="(v) => run(() => settings.setAutostart(v))"
                />
              </div>

              <div class="px-4 py-3.75">
                <p class="mb-0.5 text-[13.5px] font-semibold">Khi đóng cửa sổ</p>
                <p class="mb-3 text-[12px]" :style="{ color: 'var(--vx-text-3)' }">
                  Velix vẫn nhận thông báo khi thu về khay hệ thống.
                </p>
                <div class="flex flex-col gap-2">
                  <button
                    v-for="option in [
                      { value: true, label: 'Thu về khay hệ thống', hint: '(khuyến nghị)' },
                      { value: false, label: 'Thoát hẳn ứng dụng', hint: '' },
                    ]"
                    :key="String(option.value)"
                    type="button"
                    class="flex cursor-pointer items-center gap-2.25 text-left text-[13px]"
                    @click="run(() => settings.setCloseToTray(option.value))"
                  >
                    <span
                      class="h-4 w-4 shrink-0 rounded-full transition-all duration-100"
                      :style="
                        settings.settings.closeToTray === option.value
                          ? { border: '5px solid var(--vx-accent)', background: '#fff' }
                          : { border: '1.5px solid var(--vx-input-border)' }
                      "
                    />
                    <span>
                      {{ option.label }}
                      <span v-if="option.hint" :style="{ color: 'var(--vx-text-4)' }">
                        {{ option.hint }}
                      </span>
                    </span>
                  </button>
                </div>
              </div>
            </div>
          </section>

          <!-- Thông báo -->
          <section v-else>
            <h2 class="mb-5 font-display text-[19px] font-bold">Thông báo</h2>

            <div
              class="mb-3.5 flex items-center gap-3.5 rounded-[14px] px-4 py-3.75"
              style="background: var(--vx-card); border: 1px solid var(--vx-card-border)"
            >
              <div class="flex-1">
                <p class="text-[13.5px] font-semibold">Thông báo hệ thống</p>
                <p class="mt-0.5 text-[12px]" :style="{ color: 'var(--vx-text-3)' }">
                  Tắt là tắt tất cả — kể cả khi app chạy nền.
                </p>
              </div>
              <!-- Stored inverted: Rust calls the suppressed state `quiet`. -->
              <ToggleSwitch
                :model-value="!settings.settings.quiet"
                @update:model-value="(v) => run(() => settings.setQuiet(!v))"
              />
            </div>

            <p
              class="mb-2.25 ml-0.5 text-[12px] font-bold tracking-[0.06em] uppercase"
              :style="{ color: 'var(--vx-text-4)' }"
            >
              Theo tài khoản
            </p>

            <div
              class="overflow-hidden rounded-[14px] transition-opacity"
              :class="settings.settings.quiet ? 'opacity-55' : ''"
              style="background: var(--vx-card); border: 1px solid var(--vx-card-border)"
            >
              <div
                v-for="(profile, index) in notifiable"
                :key="profile.id"
                class="flex items-center gap-3 px-3.5 py-2.75"
                :style="
                  index < notifiable.length - 1
                    ? { borderBottom: '1px solid var(--vx-divider)' }
                    : {}
                "
              >
                <GlyphBadge
                  :glyph="profile.name.trim().charAt(0).toUpperCase()"
                  :tint="pluginTint(findPlugin(profile.pluginId))"
                  :size="30"
                  :radius="9"
                />
                <div class="min-w-0 flex-1">
                  <p class="truncate text-[13px] font-semibold">{{ profile.name }}</p>
                  <p class="text-[11.5px]" :style="{ color: 'var(--vx-text-4)' }">
                    {{ platformName(profile.pluginId) }}
                  </p>
                </div>
                <ToggleSwitch
                  :model-value="!profile.muted"
                  :disabled="settings.settings.quiet"
                  @update:model-value="(v) => run(() => profiles.setMuted(profile.id, !v))"
                />
              </div>

              <p
                v-if="!notifiable.length"
                class="px-4 py-3.5 text-[12.5px]"
                :style="{ color: 'var(--vx-text-4)' }"
              >
                Chưa có tài khoản nào.
              </p>
            </div>
          </section>
        </div>
      </div>
    </div>
  </div>
</template>
