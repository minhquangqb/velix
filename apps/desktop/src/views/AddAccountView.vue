<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import type { VelixPlugin } from '@velix/core'

import GlyphBadge from '../components/GlyphBadge.vue'
import TitleBar from '../components/TitleBar.vue'
import { PLUGINS, pluginDomain, pluginGlyph, pluginTint } from '../registry'
import { usePlatformsStore } from '../stores/platforms'
import { useProfilesStore } from '../stores/profiles'
import { useTabsStore } from '../stores/tabs'
import { useUiStore } from '../stores/ui'

const platforms = usePlatformsStore()
const profiles = useProfilesStore()
const tabs = useTabsStore()
const ui = useUiStore()

const emit = defineEmits<{ error: [message: string] }>()

const NAME_SUGGESTIONS = ['Cá nhân', 'Công việc', 'Shop', 'CSKH']

const picked = ref<VelixPlugin | null>(PLUGINS.find((p) => p.id === ui.addAccountPluginId) ?? null)
const name = ref('')
const busy = ref(false)
const nameInput = ref<HTMLInputElement | null>(null)

const step = computed(() => (picked.value ? 2 : 1))
const canContinue = computed(() => !!picked.value && name.value.trim().length > 0)

watch(picked, async (plugin) => {
  if (!plugin) return
  await nextTick()
  nameInput.value?.focus()
})

function pick(plugin: VelixPlugin) {
  picked.value = plugin
}

function back() {
  if (picked.value) {
    picked.value = null
    name.value = ''
  } else {
    ui.goto('main')
  }
}

// Escape steps back. Bound on window rather than the root element: step 1 has
// nothing focused, so a keydown handler on the template would never fire.
function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') back()
}
onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))

/**
 * Creates the profile and opens its webview, then hands off to the main view —
 * the login page loads in the normal workspace rather than inside the wizard,
 * because platform webviews are positioned into the workspace rectangle and
 * cannot be embedded in an arbitrary panel.
 */
async function submit() {
  const plugin = picked.value
  if (!plugin || !canContinue.value || busy.value) return

  busy.value = true
  try {
    const profile = await profiles.create(plugin.id, name.value.trim())
    platforms.select(plugin.id)
    await ui.goto('main')
    await tabs.open(profile)
  } catch (e) {
    emit('error', String(e))
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="flex h-full flex-col">
    <TitleBar :crumbs="['Thêm tài khoản']" />

    <div class="flex shrink-0 justify-center pt-6.5 pb-1.5">
      <div class="flex items-center gap-2.5">
        <template v-for="(label, index) in ['Chọn nền tảng', 'Đặt tên']" :key="label">
          <span
            v-if="index > 0"
            class="h-px w-11"
            :style="{ background: 'var(--vx-input-border)' }"
          />
          <span class="flex items-center gap-2.5">
            <span
              class="grid h-5.5 w-5.5 place-items-center rounded-full font-display text-[11px] font-bold"
              :style="
                step === index + 1
                  ? {
                      color: '#fff',
                      background: 'var(--vx-accent-grad)',
                      boxShadow: '0 2px 8px rgba(100,116,238,.35)',
                    }
                  : step > index + 1
                    ? {
                        color: 'var(--vx-success)',
                        background: 'rgba(61,214,140,.12)',
                        boxShadow: 'inset 0 0 0 1px rgba(61,214,140,.3)',
                      }
                    : {
                        color: 'var(--vx-text-4)',
                        background: 'var(--vx-ghost)',
                        boxShadow: 'inset 0 0 0 1px var(--vx-input-border)',
                      }
              "
            >
              {{ step > index + 1 ? '✓' : index + 1 }}
            </span>
            <span
              class="text-[12.5px]"
              :style="
                step === index + 1
                  ? { fontWeight: 700, color: 'var(--vx-text)' }
                  : { fontWeight: 500, color: 'var(--vx-text-4)' }
              "
            >
              {{ label }}
            </span>
          </span>
        </template>
      </div>
    </div>

    <div class="grid min-h-0 flex-1 place-items-center p-6">
      <!-- Step 1 -->
      <div v-if="!picked" class="vx-fade w-160 max-w-full text-center">
        <h1 class="mb-2 font-display text-[22px] font-bold">Chọn nền tảng</h1>
        <p class="mb-7 text-[13px]" :style="{ color: 'var(--vx-text-3)' }">
          Danh sách sẽ mở rộng qua plugin — chọn nơi tài khoản của bạn đang ở.
        </p>

        <div class="mb-6 grid grid-cols-4 gap-3">
          <button
            v-for="plugin in PLUGINS"
            :key="plugin.id"
            type="button"
            class="cursor-pointer rounded-[14px] px-3 pt-5.5 pb-4.5 transition-all duration-150 hover:-translate-y-0.5"
            :style="{ background: 'var(--vx-card)', border: '1px solid var(--vx-card-border)' }"
            @click="pick(plugin)"
          >
            <GlyphBadge
              class="mx-auto"
              :glyph="pluginGlyph(plugin)"
              :tint="pluginTint(plugin)"
              :size="46"
              :radius="14"
            />
            <p class="mt-3 text-[13px] font-semibold">{{ plugin.name }}</p>
            <p class="mt-0.5 text-[11px]" :style="{ color: 'var(--vx-text-4)' }">
              {{ pluginDomain(plugin) }}
            </p>
          </button>
        </div>

        <button
          type="button"
          class="cursor-pointer text-[12px]"
          :style="{ color: 'var(--vx-text-5)' }"
          @click="ui.goto('main')"
        >
          Quay lại workspace
        </button>
      </div>

      <!-- Step 2 -->
      <div v-else class="vx-fade w-115 max-w-full text-center">
        <GlyphBadge
          class="mx-auto"
          :glyph="pluginGlyph(picked)"
          :tint="pluginTint(picked)"
          :size="52"
          :radius="16"
        />
        <h1 class="mt-4.5 mb-2 font-display text-[22px] font-bold">
          Đặt tên cho tài khoản {{ picked.name }}
        </h1>
        <p class="mb-6.5 text-[13px]" :style="{ color: 'var(--vx-text-3)' }">
          Tên chỉ hiển thị trong Velix, giúp bạn phân biệt các tài khoản cùng nền tảng.
        </p>

        <input
          ref="nameInput"
          v-model="name"
          placeholder="Ví dụ: Cá nhân, Shop Táo Xanh…"
          class="w-full rounded-[11px] px-4 py-3.25 text-center text-[15px] outline-none"
          :style="{
            background: 'var(--vx-well)',
            border: '1px solid var(--vx-input-border)',
            color: 'var(--vx-text)',
          }"
          @keydown.enter="submit"
        />

        <div class="mt-3.5 flex flex-wrap justify-center gap-2">
          <button
            v-for="suggestion in NAME_SUGGESTIONS"
            :key="suggestion"
            type="button"
            class="cursor-pointer rounded-full px-3.25 py-1.25 text-[12px] font-semibold hover:text-(--vx-text)!"
            :style="{ color: 'var(--vx-text-2)', border: '1px solid var(--vx-input-border)' }"
            @click="name = suggestion"
          >
            {{ suggestion }}
          </button>
        </div>

        <div class="mt-7.5 flex justify-center gap-2.25">
          <button
            type="button"
            class="cursor-pointer rounded-[9px] px-5 py-2.5 text-[13px] font-semibold"
            :style="{
              background: 'var(--vx-ghost)',
              border: '1px solid var(--vx-input-border)',
              color: 'var(--vx-text)',
            }"
            @click="back"
          >
            ← Quay lại
          </button>
          <button
            type="button"
            :disabled="!canContinue || busy"
            class="rounded-[9px] px-5 py-2.5 text-[13px] font-bold"
            :style="
              canContinue && !busy
                ? {
                    background: 'var(--vx-accent-grad)',
                    color: '#fff',
                    cursor: 'pointer',
                    boxShadow:
                      '0 3px 10px rgba(100,116,238,.3), inset 0 1px 0 rgba(255,255,255,.25)',
                  }
                : { background: 'var(--vx-ghost)', color: 'var(--vx-text-4)', cursor: 'default' }
            "
            @click="submit"
          >
            {{ busy ? 'Đang mở phiên…' : 'Tiếp tục — mở trang đăng nhập' }}
          </button>
        </div>

        <p class="mt-4 text-[11.5px] leading-relaxed" :style="{ color: 'var(--vx-text-5)' }">
          Bạn đăng nhập trực tiếp trên trang của nền tảng.<br />
          Velix không bao giờ hỏi mật khẩu của bạn.
        </p>
      </div>
    </div>
  </div>
</template>
