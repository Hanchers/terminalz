<template>
  <div class="kp-panel">
    <div class="kp-toolbar">
      <span class="kp-title">{{ $t('keychain.title') }}</span>
      <button class="kp-btn" @click="openNew">{{ $t('keychain.newEntry') }}</button>
    </div>
    <div class="kp-scroll">
      <div v-if="keys.length > 0" class="kp-cards">
        <div
          v-for="k in keys"
          :key="k.id"
          class="kp-card"
          @dblclick="editKey(k)"
          @contextmenu.prevent="showCtx(k, $event)"
        >
          <div class="kp-icon">{{ k.key_type === 'password' ? '🔑' : '🗝️' }}</div>
          <div class="kp-name">{{ k.name }}</div>
          <div class="kp-meta">
            <span class="kp-type">{{ k.key_type === 'password' ? $t('keychain.passwordEntry') : $t('keychain.sshKey') }}</span>
            <span v-if="k.username" class="kp-user">{{ k.username }}</span>
          </div>
          <div class="kp-note" v-if="k.host">{{ k.host }}</div>
        </div>
      </div>
      <div v-else class="kp-empty">{{ $t('keychain.empty') }}</div>
    </div>
    <div v-if="toast" class="toast-bar" @click="toast = ''">{{ toast }}</div>

    <!-- Context Menu -->
    <div v-if="ctx.visible" class="context-menu" :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }">
      <div class="ctx-item" @click="editKey(ctx.key!)">{{ $t('keychain.editEntry') }}</div>
      <div class="ctx-sep"></div>
      <div class="ctx-item ctx-danger" @click="deleteKey(ctx.key!)">{{ $t('keychain.delete') }}</div>
    </div>

    <!-- Dialog -->
    <div v-if="dialog.visible" class="modal-overlay" @click.self="closeDialog">
      <div class="modal-box">
        <div class="modal-title">{{ dialog.editingId ? $t('keychain.editEntry') : $t('keychain.newEntry') }}</div>
        <div class="modal-field">
          <label>{{ $t('keychain.name') }}</label>
          <input v-model="dialog.name" :placeholder="$t('keychain.namePlaceholder')" />
        </div>
        <div class="modal-field">
          <label>{{ $t('keychain.keyType') }}</label>
          <select v-model="dialog.key_type">
            <option value="password">{{ $t('keychain.typePassword') }}</option>
            <option value="private_key">{{ $t('keychain.typePrivateKey') }}</option>
          </select>
        </div>
        <div class="modal-field">
          <label>{{ $t('keychain.username') }}</label>
          <input v-model="dialog.username" :placeholder="$t('keychain.usernamePlaceholder')" />
        </div>
        <div class="modal-field" v-if="dialog.key_type === 'password'">
          <label>{{ $t('keychain.password') }}</label>
          <input v-model="dialog.password" type="password" :placeholder="$t('keychain.passwordPlaceholder')" />
        </div>
        <template v-if="dialog.key_type === 'private_key'">
          <div class="modal-field">
            <label>{{ $t('keychain.privateKey') }}</label>
            <textarea v-model="dialog.private_key" :placeholder="$t('keychain.privateKeyPlaceholder')" rows="4"></textarea>
          </div>
          <div class="modal-field">
            <label>{{ $t('keychain.passphrase') }}</label>
            <input v-model="dialog.password" type="password" :placeholder="$t('keychain.passphrase')" />
          </div>
        </template>
        <div class="modal-field">
          <label>{{ $t('keychain.hostFilter') }}</label>
          <input v-model="dialog.host" :placeholder="$t('keychain.hostFilterPlaceholder')" />
        </div>
        <div class="modal-field">
          <label>{{ $t('keychain.remark') }}</label>
          <textarea v-model="dialog.remark" :placeholder="$t('keychain.remarkPlaceholder')" rows="2"></textarea>
        </div>
        <div class="modal-actions">
          <button class="modal-btn cancel" @click="closeDialog">{{ $t('keychain.cancel') }}</button>
          <button class="modal-btn primary" @click="saveDialog">{{ $t('keychain.save') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import type { SshKey } from '../types'

const { t } = useI18n({ useScope: 'global' })

const toast = ref('')
let toastTimer: ReturnType<typeof setTimeout> | null = null
function showToast(msg: string) {
  toast.value = msg
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toast.value = '' }, 3000)
}
interface Ctx { visible: boolean; x: number; y: number; key: SshKey | null }

const keys = ref<SshKey[]>([])
const ctx = reactive<Ctx>({ visible: false, x: 0, y: 0, key: null })
const dialog = reactive({ visible: false, editingId: 0, name: '', key_type: 'password', username: '', password: '', private_key: '', host: '', remark: '' })

window.addEventListener('click', () => { ctx.visible = false })

function showCtx(key: SshKey, e: MouseEvent) {
  ctx.visible = true; ctx.x = e.clientX; ctx.y = e.clientY; ctx.key = key
}

function openNew() {
  Object.assign(dialog, { visible: true, editingId: 0, name: '', key_type: 'password', username: '', password: '', private_key: '', host: '', remark: '' })
}

function editKey(k: SshKey) {
  ctx.visible = false
  Object.assign(dialog, { visible: true, editingId: k.id, name: k.name, key_type: k.key_type, username: k.username, password: '', private_key: '', host: k.host, remark: k.remark })
}

function closeDialog() { dialog.visible = false }

async function saveDialog() {
  if (!dialog.name) return
  try {
    await invoke('save_ssh_key', { key: { id: dialog.editingId, name: dialog.name, key_type: dialog.key_type, username: dialog.username, password: dialog.password, private_key: dialog.private_key, host: dialog.host, remark: dialog.remark } })
    closeDialog()
    await loadKeys()
  } catch (e) { showToast(t('sidebar.error.failed') + e) }
}

async function deleteKey(k: SshKey) {
  ctx.visible = false
  if (!confirm(t('keychain.deleteConfirm'))) return
  try { await invoke('delete_ssh_key', { id: k.id }); await loadKeys() } catch (e) { showToast(t('sidebar.error.failed') + e) }
}

async function loadKeys() {
  try { keys.value = await invoke<SshKey[]>('list_ssh_keys') } catch (_) { keys.value = [] }
}

onMounted(() => { loadKeys() })
</script>

<style scoped>
.kp-panel { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.kp-toolbar { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; }
.kp-title { font-size: 11px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.5px; }
.kp-btn { padding: 5px 12px; font-size: 11px; background: transparent; border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; }
.kp-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }
.kp-scroll { flex: 1; overflow-y: auto; padding: 12px; }
.kp-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 8px; }
.kp-card { padding: 12px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 6px; cursor: pointer; transition: all 0.12s; }
.kp-card:hover { border-color: var(--color-accent); }
.kp-icon { font-size: 20px; margin-bottom: 4px; }
.kp-name { font-size: 12px; font-weight: 500; color: var(--color-text-primary); }
.kp-meta { display: flex; gap: 6px; margin-top: 4px; }
.kp-type { font-size: 9px; padding: 1px 5px; background: var(--color-accent-bg); border-radius: 3px; color: var(--color-accent); }
.kp-user { font-size: 10px; color: var(--color-text-secondary); }
.kp-note { font-size: 10px; color: var(--color-text-tertiary); margin-top: 2px; }
.kp-empty { padding: 24px; text-align: center; color: var(--color-text-tertiary); font-size: 12px; line-height: 1.6; }
.toast-bar { padding: 6px 12px; font-size: 11px; color: var(--color-text-primary); background: var(--color-accent-bg); border-top: 1px solid var(--color-accent); cursor: pointer; flex-shrink: 0; }
.context-menu { position: fixed; z-index: 200; min-width: 150px; background: var(--color-bg-panel); border: 1px solid var(--color-border-primary); border-radius: 6px; box-shadow: var(--shadow-panel); padding: 4px; }
.ctx-item { padding: 6px 10px; font-size: 12px; color: var(--color-text-primary); border-radius: 4px; cursor: pointer; }
.ctx-item:hover { background: var(--color-bg-hover); }
.ctx-danger { color: var(--color-danger); }
.ctx-sep { height: 1px; background: var(--color-border-secondary); margin: 3px 6px; }
</style>
