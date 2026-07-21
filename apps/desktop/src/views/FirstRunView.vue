<script setup lang="ts">
import GlyphBadge from '../components/GlyphBadge.vue'
import TitleBar from '../components/TitleBar.vue'
import { PLUGINS, pluginMark } from '../registry'
import { useUiStore } from '../stores/ui'

const ui = useUiStore()
</script>

<template>
  <div class="relative flex h-full flex-col">
    <!-- Ambient glow, purely decorative. -->
    <div
      class="pointer-events-none absolute inset-0"
      style="
        background: radial-gradient(
          ellipse 60% 45% at 50% 8%,
          rgba(124, 140, 248, 0.08),
          transparent 65%
        );
      "
    />

    <div class="relative z-1 flex h-full flex-col">
      <TitleBar :crumbs="[]" />

      <div class="grid flex-1 place-items-center p-6">
        <div class="w-140 max-w-full text-center">
          <div
            class="vx-fade mx-auto mb-6.5 grid h-14 w-14 place-items-center rounded-[17px] font-display text-[28px] font-bold text-white"
            style="
              background: var(--vx-brand-grad);
              box-shadow:
                0 8px 28px rgba(93, 108, 232, 0.4),
                inset 0 1px 0 rgba(255, 255, 255, 0.25);
            "
          >
            V
          </div>

          <h1 class="vx-fade mb-3 font-display text-[30px] font-bold tracking-[-0.01em]">
            Mọi cuộc trò chuyện, một cửa sổ
          </h1>
          <p
            class="vx-fade mx-auto mb-8.5 max-w-110 text-[14.5px] leading-relaxed"
            :style="{ color: 'var(--vx-text-2)' }"
          >
            Messenger, Zalo, Telegram, ChatGPT… gom về một nơi. Mỗi tài khoản là một phiên đăng nhập
            độc lập — dùng 3 tài khoản Messenger cùng lúc, không đè phiên nhau.
          </p>

          <div class="vx-fade mb-9.5 flex justify-center gap-3">
            <div
              class="max-w-62.5 flex-1 rounded-[14px] px-4 py-4.5"
              :style="{ background: 'var(--vx-card)', border: '1px solid var(--vx-card-border)' }"
            >
              <div class="mb-3.25 flex justify-center gap-1.75">
                <GlyphBadge
                  v-for="plugin in PLUGINS"
                  :key="plugin.id"
                  v-bind="pluginMark(plugin)"
                  :size="32"
                  :radius="10"
                />
              </div>
              <p class="mb-0.75 text-[12.5px] font-semibold">Một cửa sổ thay chục tab</p>
              <p class="text-[11.5px] leading-relaxed" :style="{ color: 'var(--vx-text-3)' }">
                Nhẹ hơn trình duyệt, chạy nền yên tĩnh trong khay hệ thống.
              </p>
            </div>

            <div
              class="max-w-62.5 flex-1 rounded-[14px] px-4 py-4.5"
              :style="{ background: 'var(--vx-card)', border: '1px solid var(--vx-card-border)' }"
            >
              <div class="mb-3.25 flex justify-center gap-1.75">
                <GlyphBadge
                  v-for="glyph in ['C', 'S', 'K']"
                  :key="glyph"
                  :glyph="glyph"
                  tint="#7B8CFF"
                  :size="32"
                  :radius="10"
                >
                  <span
                    class="absolute -right-0.5 -bottom-0.5 h-2.25 w-2.25 rounded-full"
                    :style="{
                      border: '2px solid var(--vx-pip-ring)',
                      background: 'var(--vx-success)',
                    }"
                  />
                </GlyphBadge>
              </div>
              <p class="mb-0.75 text-[12.5px] font-semibold">Nhiều tài khoản, phiên độc lập</p>
              <p class="text-[11.5px] leading-relaxed" :style="{ color: 'var(--vx-text-3)' }">
                Cá nhân, Shop, CSKH — cùng nền tảng, đăng nhập song song.
              </p>
            </div>
          </div>

          <div class="vx-fade">
            <button
              type="button"
              class="cursor-pointer rounded-[11px] px-6.5 py-3 text-[14px] font-bold text-white"
              style="
                background: var(--vx-accent-grad);
                box-shadow:
                  0 6px 20px rgba(100, 116, 238, 0.35),
                  inset 0 1px 0 rgba(255, 255, 255, 0.25);
              "
              @click="ui.goto('add-account')"
            >
              Thêm tài khoản đầu tiên
            </button>
            <p class="mt-4 text-[12px] leading-relaxed" :style="{ color: 'var(--vx-text-4)' }">
              Bạn đăng nhập trực tiếp trên trang của nền tảng.<br />
              Velix không bao giờ hỏi mật khẩu của bạn.
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
