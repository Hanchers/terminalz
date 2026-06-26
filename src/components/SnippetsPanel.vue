<template>
  <div class="sp-panel">
    <div class="sp-toolbar">
      <input v-model="search" class="sp-search" :placeholder="$t('snippets.searchPlaceholder')" />
      <button class="sp-btn" @click="openNew">{{ $t('snippets.newSnippet') }}</button>
    </div>
    <div class="sp-scroll">
      <div v-if="filtered.length > 0" class="sp-cards">
        <div
          v-for="s in filtered"
          :key="s.id"
          class="sp-card"
          @dblclick="sendToTerminal(s)"
          @contextmenu.prevent="showCtx(s, $event)"
        >
          <div class="sp-icon">{{ s.is_favorite ? '⭐' : '📝' }}</div>
          <div class="sp-name">{{ s.name }}</div>
          <div class="sp-preview">{{ s.content.slice(0, 80) }}{{ s.content.length > 80 ? '…' : '' }}</div>
          <div class="sp-meta">
            <span class="sp-lang">{{ s.language }}</span>
          </div>
        </div>
      </div>
      <div v-else class="sp-empty">{{ $t('snippets.empty') }}</div>
    </div>
    <div v-if="toast" class="toast-bar" @click="toast = ''">{{ toast }}</div>

    <!-- Context Menu -->
    <div v-if="ctx.visible" class="context-menu" :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }">
      <div class="ctx-item" @click="sendToTerminal(ctx.snippet!)">{{ $t('snippets.sendToTerminal') }}</div>
      <div class="ctx-item" @click="copyToClipboard(ctx.snippet!)">{{ $t('snippets.copyToClipboard') }}</div>
      <div class="ctx-item" @click="toggleFavorite(ctx.snippet!)">
        {{ ctx.snippet!.is_favorite ? 'Unfavorite' : $t('snippets.favorite') }}
      </div>
      <div class="ctx-sep"></div>
      <div class="ctx-item" @click="editSnippet(ctx.snippet!)">{{ $t('snippets.editSnippet') }}</div>
      <div class="ctx-item ctx-danger" @click="deleteSnippet(ctx.snippet!)">{{ $t('snippets.delete') }}</div>
    </div>

    <!-- Dialog -->
    <div v-if="dialog.visible" class="modal-overlay" @click.self="closeDialog">
      <div class="modal-box" style="max-width:560px">
        <div class="modal-title">{{ dialog.editingId ? $t('snippets.editSnippet') : $t('snippets.newSnippet') }}</div>
        <div class="modal-field">
          <label>{{ $t('snippets.name') }}</label>
          <input v-model="dialog.name" :placeholder="$t('snippets.namePlaceholder')" />
        </div>
        <div class="modal-field">
          <label>{{ $t('snippets.content') }}</label>
          <textarea v-model="dialog.content" :placeholder="$t('snippets.contentPlaceholder')" rows="5" style="font-family:monospace;font-size:12px"></textarea>
        </div>
        <div class="modal-field">
          <label>{{ $t('snippets.language') }}</label>
          <select v-model="dialog.language">
            <option value="shell">Shell</option>
            <option value="bash">Bash</option>
            <option value="powershell">PowerShell</option>
            <option value="python">Python</option>
            <option value="sql">SQL</option>
            <option value="docker">Docker</option>
            <option value="other">Other</option>
          </select>
        </div>
        <div class="modal-field">
          <label>{{ $t('snippets.remark') }}</label>
          <textarea v-model="dialog.remark" :placeholder="$t('snippets.remarkPlaceholder')" rows="2"></textarea>
        </div>
        <div class="modal-actions">
          <button class="modal-btn cancel" @click="closeDialog">{{ $t('snippets.cancel') }}</button>
          <button class="modal-btn primary" @click="saveDialog">{{ $t('snippets.save') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import type { Snippet } from '../types'

const { t } = useI18n({ useScope: 'global' })

const toast = ref('')
let toastTimer: ReturnType<typeof setTimeout> | null = null
function showToast(msg: string) {
  toast.value = msg
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toast.value = '' }, 3000)
}
interface Ctx { visible: boolean; x: number; y: number; snippet: Snippet | null }

const emit = defineEmits<{ 'send-command': [command: string] }>()

const snippets = ref<Snippet[]>([])
const search = ref('')
const ctx = reactive<Ctx>({ visible: false, x: 0, y: 0, snippet: null })
const dialog = reactive({ visible: false, editingId: 0, name: '', content: '', language: 'shell', remark: '' })

const filtered = computed(() => {
  if (!search.value) return snippets.value
  const q = search.value.toLowerCase()
  return snippets.value.filter(s =>
    s.name.toLowerCase().includes(q) || s.content.toLowerCase().includes(q)
  )
})

window.addEventListener('click', () => { ctx.visible = false })

function showCtx(s: Snippet, e: MouseEvent) {
  ctx.visible = true; ctx.x = e.clientX; ctx.y = e.clientY; ctx.snippet = s
}

function openNew() {
  Object.assign(dialog, { visible: true, editingId: 0, name: '', content: '', language: 'shell', remark: '' })
}

function editSnippet(s: Snippet) {
  ctx.visible = false
  Object.assign(dialog, { visible: true, editingId: s.id, name: s.name, content: s.content, language: s.language, remark: s.remark })
}

function closeDialog() { dialog.visible = false }

async function saveDialog() {
  if (!dialog.name || !dialog.content) return
  try {
    await invoke('save_snippet', { snippet: { id: dialog.editingId, name: dialog.name, content: dialog.content, language: dialog.language, is_favorite: false, remark: dialog.remark } })
    closeDialog()
    await loadSnippets()
  } catch (e) { showToast(t('sidebar.error.failed') + e) }
}

async function toggleFavorite(s: Snippet) {
  ctx.visible = false
  try {
    await invoke('save_snippet', { snippet: { ...s, is_favorite: !s.is_favorite } })
    await loadSnippets()
  } catch (e) { showToast(t('sidebar.error.failed') + e) }
}

async function sendToTerminal(s: Snippet) {
  ctx.visible = false
  if (s.content) { emit('send-command', s.content) }
}

function copyToClipboard(s: Snippet) {
  ctx.visible = false
  navigator.clipboard.writeText(s.content).then(() => {})
}

async function deleteSnippet(s: Snippet) {
  ctx.visible = false
  if (!confirm(t('snippets.deleteConfirm'))) return
  try { await invoke('delete_snippet', { id: s.id }); await loadSnippets() } catch (e) { showToast(t('sidebar.error.failed') + e) }
}

async function loadSnippets() {
  try { snippets.value = await invoke<Snippet[]>('list_snippets') } catch (_) { snippets.value = [] }
}

onMounted(() => { loadSnippets() })
</script>

<style scoped>
.sp-panel { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.sp-toolbar { display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; }
.sp-search { flex: 1; padding: 5px 10px; font-size: 12px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-primary); outline: none; }
.sp-search:focus { border-color: var(--color-accent); }
.sp-btn { padding: 5px 12px; font-size: 11px; background: transparent; border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; }
.sp-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }
.sp-scroll { flex: 1; overflow-y: auto; padding: 12px; }
.sp-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 8px; }
.sp-card { padding: 12px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 6px; cursor: pointer; transition: all 0.12s; }
.sp-card:hover { border-color: var(--color-accent); }
.sp-icon { font-size: 18px; margin-bottom: 4px; }
.sp-name { font-size: 12px; font-weight: 500; color: var(--color-text-primary); }
.sp-preview { font-size: 10px; color: var(--color-text-tertiary); margin-top: 4px; font-family: monospace; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.sp-meta { display: flex; gap: 6px; margin-top: 4px; }
.sp-lang { font-size: 9px; padding: 1px 5px; background: var(--color-accent-bg); border-radius: 3px; color: var(--color-accent); }
.sp-empty { padding: 24px; text-align: center; color: var(--color-text-tertiary); font-size: 12px; line-height: 1.6; }
.toast-bar { padding: 6px 12px; font-size: 11px; color: var(--color-text-primary); background: var(--color-accent-bg); border-top: 1px solid var(--color-accent); cursor: pointer; flex-shrink: 0; }
.context-menu { position: fixed; z-index: 200; min-width: 150px; background: var(--color-bg-panel); border: 1px solid var(--color-border-primary); border-radius: 6px; box-shadow: var(--shadow-panel); padding: 4px; }
.ctx-item { padding: 6px 10px; font-size: 12px; color: var(--color-text-primary); border-radius: 4px; cursor: pointer; }
.ctx-item:hover { background: var(--color-bg-hover); }
.ctx-danger { color: var(--color-danger); }
.ctx-sep { height: 1px; background: var(--color-border-secondary); margin: 3px 6px; }
</style>
