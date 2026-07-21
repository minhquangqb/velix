<script setup lang="ts">
import { computed, nextTick, ref, type ComponentPublicInstance } from 'vue'
import type { Profile } from '@velix/core'

import GlyphBadge from '../components/GlyphBadge.vue'
import TitleBar from '../components/TitleBar.vue'
import { Bell, BellOff, Pencil, Plus, Trash2 } from '@lucide/vue'

import { PLUGINS, pluginMark, pluginTint } from '../registry'
import { useProfilesStore } from '../stores/profiles'
import { useTabsStore } from '../stores/tabs'
import { useUiStore } from '../stores/ui'

const profiles = useProfilesStore()
const tabs = useTabsStore()
const ui = useUiStore()

const emit = defineEmits<{ error: [message: string] }>()

const editingId = ref<string | null>(null)
const draftName = ref('')

/**
 * A plain `ref="..."` inside `v-for` collects into an array, so the editing
 * input is captured with a function ref instead — only one row edits at a time.
 */
const nameInput = ref<HTMLInputElement | null>(null)
function captureInput(el: Element | ComponentPublicInstance | null) {
  nameInput.value = el instanceof HTMLInputElement ? el : null
}

/** Account pending deletion; the dialog is a type-to-confirm gate. */
const pendingDelete = ref<Profile | null>(null)
const confirmText = ref('')

const groups = computed(() =>
  PLUGINS.map((plugin) => ({ plugin, accounts: profiles.byPlugin(plugin.id) })),
)

const deleteReady = computed(
  () => !!pendingDelete.value && confirmText.value.trim() === pendingDelete.value.name,
)

async function run(action: () => Promise<unknown>) {
  try {
    await action()
  } catch (e) {
    emit('error', String(e))
  }
}

async function startRename(profile: Profile) {
  editingId.value = profile.id
  draftName.value = profile.name
  await nextTick()
  nameInput.value?.select()
}

function commitRename() {
  const id = editingId.value
  const name = draftName.value.trim()
  editingId.value = null
  if (!id || !name) return
  const current = profiles.find(id)
  if (!current || current.name === name) return
  run(() => profiles.rename(id, name))
}

function askDelete(profile: Profile) {
  pendingDelete.value = profile
  confirmText.value = ''
}

function confirmDelete() {
  const profile = pendingDelete.value
  if (!profile || !deleteReady.value) return
  pendingDelete.value = null
  run(async () => {
    await profiles.remove(profile.id)
    // Rust closes the webview as part of deletion; drop our bookkeeping too.
    tabs.forget(profile.id)
  })
}
</script>

<template>
  <div class="relative flex h-full flex-col">
    <TitleBar :crumbs="['Quản lý tài khoản']" />

    <div class="min-h-0 flex-1 overflow-y-auto px-6 py-7.5">
      <div class="vx-fade mx-auto w-155 max-w-full">
        <h1 class="mb-1.5 font-display text-[21px] font-bold">Tài khoản</h1>
        <p class="mb-6 text-[13px]" :style="{ color: 'var(--vx-text-3)' }">
          Nhấp đúp tên để đổi · chuông để tắt thông báo từng tài khoản.
        </p>

        <section v-for="group in groups" :key="group.plugin.id" class="mb-5.5">
          <div class="mb-2.25 flex items-center gap-2.25">
            <GlyphBadge v-bind="pluginMark(group.plugin)" :size="24" :radius="7" />
            <span
              class="text-[12px] font-bold tracking-[0.06em] uppercase"
              :style="{ color: 'var(--vx-text-2)' }"
            >
              {{ group.plugin.name }}
            </span>
            <span class="text-[11px]" :style="{ color: 'var(--vx-text-5)' }">
              {{ group.accounts.length ? `${group.accounts.length} tài khoản` : 'trống' }}
            </span>
          </div>

          <div
            class="overflow-hidden rounded-[14px]"
            style="background: var(--vx-card); border: 1px solid var(--vx-card-border)"
          >
            <div
              v-for="(profile, index) in group.accounts"
              :key="profile.id"
              class="flex items-center gap-3 px-3.5 py-2.75 transition-colors hover:bg-(--vx-hover)"
              :style="
                index < group.accounts.length - 1
                  ? { borderBottom: '1px solid var(--vx-divider)' }
                  : {}
              "
            >
              <GlyphBadge
                :glyph="profile.name.trim().charAt(0).toUpperCase()"
                :tint="pluginTint(group.plugin)"
                :size="34"
                :radius="10"
              />

              <div class="min-w-0 flex-1">
                <input
                  v-if="editingId === profile.id"
                  :ref="captureInput"
                  v-model="draftName"
                  class="w-55 rounded-[7px] px-2.25 py-1.25 text-[13px] font-semibold outline-none"
                  :style="{
                    background: 'var(--vx-well)',
                    border: '1px solid rgba(124,140,248,.5)',
                    color: 'var(--vx-text)',
                  }"
                  @keydown.enter="commitRename"
                  @keydown.esc="editingId = null"
                  @blur="commitRename"
                />
                <p
                  v-else
                  class="cursor-text truncate text-[13.5px] font-semibold"
                  title="Nhấp đúp để đổi tên"
                  @dblclick="startRename(profile)"
                >
                  {{ profile.name }}
                </p>
                <p class="mt-px text-[11.5px]" :style="{ color: 'var(--vx-text-4)' }">
                  {{ tabs.isOpen(profile.id) ? 'Phiên hoạt động' : 'Chưa mở' }} ·
                  {{ profile.muted ? 'thông báo đang tắt' : 'thông báo bật' }}
                </p>
              </div>

              <button
                type="button"
                class="grid h-7.5 w-7.5 cursor-pointer place-items-center rounded-lg"
                :style="
                  profile.muted
                    ? { color: 'var(--vx-warning)', background: 'rgba(245,192,68,.09)' }
                    : { color: 'var(--vx-text-3)' }
                "
                :title="profile.muted ? 'Bật lại thông báo' : 'Tắt thông báo tài khoản này'"
                @click="run(() => profiles.setMuted(profile.id, !profile.muted))"
              >
                <component
                  :is="profile.muted ? BellOff : Bell"
                  :size="16"
                  :stroke-width="1.7"
                  aria-hidden="true"
                />
              </button>

              <button
                type="button"
                class="grid h-7.5 w-7.5 cursor-pointer place-items-center rounded-lg hover:bg-(--vx-ghost)! hover:text-(--vx-text)!"
                :style="{ color: 'var(--vx-text-3)' }"
                title="Đổi tên"
                @click="startRename(profile)"
              >
                <Pencil :size="15" :stroke-width="1.7" aria-hidden="true" />
              </button>

              <button
                type="button"
                class="grid h-7.5 w-7.5 cursor-pointer place-items-center rounded-lg hover:!bg-[rgba(242,85,90,.12)] hover:text-(--vx-danger)!"
                :style="{ color: 'var(--vx-text-3)' }"
                title="Xóa tài khoản"
                @click="askDelete(profile)"
              >
                <Trash2 :size="15" :stroke-width="1.7" aria-hidden="true" />
              </button>
            </div>

            <div v-if="!group.accounts.length" class="flex items-center gap-2.75 px-4 py-3.5">
              <p
                class="flex-1 text-[12.5px] leading-relaxed"
                :style="{ color: 'var(--vx-text-4)' }"
              >
                Chưa có tài khoản nào — màn hình chính vẫn hiển thị nền tảng này kèm lời mời thêm
                tài khoản.
              </p>
              <button
                type="button"
                class="cursor-pointer text-[12px] font-bold whitespace-nowrap"
                :style="{ color: 'var(--vx-accent-text)' }"
                @click="ui.goto('add-account', group.plugin.id)"
              >
                ＋ Thêm tài khoản
              </button>
            </div>
          </div>
        </section>

        <button
          type="button"
          class="inline-flex cursor-pointer items-center gap-2 rounded-[10px] px-4 py-2.25 text-[13px] font-semibold hover:text-(--vx-text)!"
          :style="{ color: 'var(--vx-text-2)', border: '1px dashed var(--vx-input-border)' }"
          @click="ui.goto('add-account')"
        >
          <Plus :size="13" :stroke-width="2" aria-hidden="true" />
          Thêm tài khoản
        </button>
      </div>
    </div>

    <!-- Delete confirmation. Typing the exact name is the gate: deletion wipes
         the profile's whole data directory, which is not recoverable. -->
    <div
      v-if="pendingDelete"
      class="absolute inset-0 z-10 grid place-items-center backdrop-blur-[3px]"
      :style="{ background: 'var(--vx-scrim)' }"
      @click="pendingDelete = null"
      @keydown.esc="pendingDelete = null"
    >
      <div
        class="vx-pop w-107.5 max-w-[calc(100%-48px)] rounded-[15px] p-6"
        :style="{
          background: 'var(--vx-panel)',
          border: '1px solid rgba(242,85,90,.25)',
          boxShadow: 'var(--vx-shadow-panel)',
        }"
        @click.stop
      >
        <div class="mb-3.5 flex items-center gap-3">
          <div
            class="grid h-10 w-10 shrink-0 place-items-center rounded-xl"
            style="
              background: rgba(242, 85, 90, 0.1);
              border: 1px solid rgba(242, 85, 90, 0.25);
              color: #f2555a;
            "
          >
            <Trash2 :size="19" :stroke-width="1.6" aria-hidden="true" />
          </div>
          <div class="min-w-0">
            <p class="truncate font-display text-[16px] font-bold">
              Xóa "{{ pendingDelete.name }}"?
            </p>
            <p class="text-[12px]" :style="{ color: 'var(--vx-text-3)' }">
              {{ PLUGINS.find((p) => p.id === pendingDelete?.pluginId)?.name }} · hành động không
              thể hoàn tác
            </p>
          </div>
        </div>

        <p
          class="mb-4.5 rounded-[10px] px-3.5 py-3 text-[13px] leading-relaxed"
          :style="{
            color: 'var(--vx-text-2)',
            background: 'rgba(242,85,90,.06)',
            border: '1px solid rgba(242,85,90,.15)',
          }"
        >
          Phiên đăng nhập sẽ bị đăng xuất và
          <strong :style="{ color: 'var(--vx-text)' }">toàn bộ dữ liệu cục bộ</strong>
          của tài khoản này (cookie, cache, cấu hình) sẽ bị xóa vĩnh viễn khỏi máy. Tin nhắn trên
          máy chủ của nền tảng không bị ảnh hưởng.
        </p>

        <p class="mb-1.75 text-[12px]" :style="{ color: 'var(--vx-text-3)' }">
          Gõ
          <strong class="font-mono text-[11.5px]" :style="{ color: 'var(--vx-text)' }">
            {{ pendingDelete.name }}
          </strong>
          để xác nhận:
        </p>
        <input
          v-model="confirmText"
          autofocus
          :placeholder="pendingDelete.name"
          class="mb-4 w-full rounded-[9px] px-3.25 py-2.5 text-[13px] outline-none"
          :style="{
            background: 'var(--vx-well)',
            border: '1px solid var(--vx-input-border)',
            color: 'var(--vx-text)',
          }"
          @keydown.enter="confirmDelete"
        />

        <div class="flex justify-end gap-2.25">
          <button
            type="button"
            class="cursor-pointer rounded-[9px] px-4.5 py-2.25 text-[13px] font-semibold"
            :style="{
              background: 'var(--vx-ghost)',
              border: '1px solid var(--vx-input-border)',
              color: 'var(--vx-text)',
            }"
            @click="pendingDelete = null"
          >
            Hủy
          </button>
          <button
            type="button"
            :disabled="!deleteReady"
            class="rounded-[9px] px-4.5 py-2.25 text-[13px] font-bold"
            :style="
              deleteReady
                ? {
                    background: 'var(--vx-danger-grad)',
                    color: '#fff',
                    cursor: 'pointer',
                    boxShadow: '0 3px 10px rgba(217,58,63,.35), inset 0 1px 0 rgba(255,255,255,.2)',
                  }
                : { background: 'var(--vx-ghost)', color: 'var(--vx-text-4)', cursor: 'default' }
            "
            @click="confirmDelete"
          >
            Xóa vĩnh viễn
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
