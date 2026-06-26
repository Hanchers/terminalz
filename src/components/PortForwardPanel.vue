<template>
  <div class="pf-panel">
    <div class="pf-toolbar">
      <span class="pf-title">{{ $t('portForward.title') }}</span>
      <button class="pf-btn" @click="openNew">{{ $t('portForward.newRule') }}</button>
    </div>
    <div class="pf-scroll">
      <div v-if="rules.length > 0" class="pf-cards">
        <div
          v-for="r in rules"
          :key="r.id"
          class="pf-card"
          @dblclick="editRule(r)"
          @contextmenu.prevent="showCtx(r, $event)"
        >
          <div class="pf-icon">🔗</div>
          <div class="pf-name">{{ r.name }}</div>
          <div class="pf-note">{{ r.direction === 'local' ? 'localhost:' + r.local_port : r.remote_host + ':' + r.remote_port }} → {{ r.direction === 'local' ? r.remote_host + ':' + r.remote_port : 'localhost:' + r.local_port }}</div>
          <div class="pf-conn" v-if="r.connection_id > 0">{{ connMap[r.connection_id] || 'Host #' + r.connection_id }}</div>
          <div class="pf-status" :class="{ running: r.enabled }">
            {{ r.enabled ? $t('portForward.running') : $t('portForward.stopped') }}
          </div>
        </div>
      </div>
      <div v-else class="pf-empty">{{ $t('portForward.empty') }}</div>
    </div>
    <div v-if="toast" class="toast-bar" @click="toast = ''">{{ toast }}</div>

    <!-- Context Menu -->
    <div v-if="ctx.visible" class="context-menu" :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }">
      <div class="ctx-item" @click="editRule(ctx.rule!)">{{ $t('portForward.editRule') }}</div>
      <div class="ctx-item" @click="toggleRule(ctx.rule!)">{{ ctx.rule!.enabled ? 'Stop' : 'Start' }}</div>
      <div class="ctx-sep"></div>
      <div class="ctx-item ctx-danger" @click="deleteRule(ctx.rule!)">{{ $t('portForward.delete') }}</div>
    </div>

    <!-- Dialog -->
    <div v-if="dialog.visible" class="modal-overlay" @click.self="closeDialog">
      <div class="modal-box">
        <div class="modal-title">{{ dialog.editingId ? $t('portForward.editRule') : $t('portForward.newRule') }}</div>
        <div class="modal-field">
          <label>{{ $t('portForward.name') }}</label>
          <input v-model="dialog.name" :placeholder="$t('portForward.namePlaceholder')" />
        </div>
        <div class="modal-field">
          <label>{{ $t('portForward.connection') }}</label>
          <select v-model="dialog.connection_id">
            <option :value="0">{{ $t('portForward.noConnection') }}</option>
            <option v-for="c in connList" :key="c[0]" :value="c[0]">{{ c[1] }}</option>
          </select>
        </div>
        <div class="modal-row">
          <div class="modal-field small">
            <label>{{ $t('portForward.localPort') }}</label>
            <input v-model.number="dialog.local_port" type="number" />
          </div>
          <div class="modal-field">
            <label>{{ $t('portForward.remoteHost') }}</label>
            <input v-model="dialog.remote_host" placeholder="localhost" />
          </div>
          <div class="modal-field small">
            <label>{{ $t('portForward.remotePort') }}</label>
            <input v-model.number="dialog.remote_port" type="number" />
          </div>
        </div>
        <div class="modal-field">
          <label>{{ $t('portForward.direction') }}</label>
          <select v-model="dialog.direction">
            <option value="local">{{ $t('portForward.directionLocal') }}</option>
            <option value="remote">{{ $t('portForward.directionRemote') }}</option>
          </select>
        </div>
        <div class="modal-field">
          <label>{{ $t('portForward.remark') }}</label>
          <textarea v-model="dialog.remark" :placeholder="$t('portForward.remarkPlaceholder')" rows="2"></textarea>
        </div>
        <div class="modal-actions">
          <button class="modal-btn cancel" @click="closeDialog">{{ $t('portForward.cancel') }}</button>
          <button class="modal-btn primary" @click="saveDialog">{{ $t('portForward.save') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import type { PortForward } from '../types'

const { t } = useI18n({ useScope: 'global' })

const toast = ref('')
let toastTimer: ReturnType<typeof setTimeout> | null = null
function showToast(msg: string) {
  toast.value = msg
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toast.value = '' }, 3000)
}
interface Ctx { visible: boolean; x: number; y: number; rule: PortForward | null }

const rules = ref<PortForward[]>([])
const connList = ref<[number, string][]>([])
const connMap = ref<Record<number, string>>({})
const ctx = reactive<Ctx>({ visible: false, x: 0, y: 0, rule: null })
const dialog = reactive({ visible: false, editingId: 0, name: '', connection_id: 0, local_port: 8080, remote_host: 'localhost', remote_port: 80, direction: 'local', remark: '' })

window.addEventListener('click', () => { ctx.visible = false })

function showCtx(rule: PortForward, e: MouseEvent) {
  ctx.visible = true; ctx.x = e.clientX; ctx.y = e.clientY; ctx.rule = rule
}

function openNew() {
  Object.assign(dialog, { visible: true, editingId: 0, name: '', connection_id: 0, local_port: 8080, remote_host: 'localhost', remote_port: 80, direction: 'local', remark: '' })
}

function editRule(r: PortForward) {
  ctx.visible = false
  Object.assign(dialog, { visible: true, editingId: r.id, name: r.name, connection_id: r.connection_id, local_port: r.local_port, remote_host: r.remote_host, remote_port: r.remote_port, direction: r.direction, remark: r.remark })
}

function closeDialog() { dialog.visible = false }

async function saveDialog() {
  if (!dialog.name || !dialog.local_port || !dialog.remote_port) return
  try {
    await invoke('save_port_forward', { config: { id: dialog.editingId, name: dialog.name, connection_id: dialog.connection_id, local_port: dialog.local_port, remote_host: dialog.remote_host, remote_port: dialog.remote_port, direction: dialog.direction, enabled: false, remark: dialog.remark } })
    closeDialog()
    await loadRules()
  } catch (e) { showToast(t('sidebar.error.failed') + e) }
}

async function toggleRule(r: PortForward) {
  ctx.visible = false
  try {
    await invoke('save_port_forward', { config: { id: r.id, name: r.name, connection_id: r.connection_id, local_port: r.local_port, remote_host: r.remote_host, remote_port: r.remote_port, direction: r.direction, enabled: !r.enabled, remark: r.remark } })
    await loadRules()
  } catch (e) { showToast(t('sidebar.error.failed') + e) }
}

async function deleteRule(r: PortForward) {
  ctx.visible = false
  if (!confirm(t('portForward.deleteConfirm'))) return
  try { await invoke('delete_port_forward', { id: r.id }); await loadRules() } catch (e) { showToast(t('sidebar.error.failed') + e) }
}

async function loadRules() {
  try {
    rules.value = await invoke<PortForward[]>('list_port_forwards')
    const compact = await invoke<[number, string][]>('list_connections_compact')
    connList.value = compact
    connMap.value = {}
    for (const [id, name] of compact) { connMap.value[id] = name }
  } catch (_) { rules.value = [] }
}

onMounted(() => { loadRules() })
</script>

<style scoped>
.pf-panel { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.pf-toolbar { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; }
.pf-title { font-size: 11px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.5px; }
.pf-btn { padding: 5px 12px; font-size: 11px; background: transparent; border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; }
.pf-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }
.pf-scroll { flex: 1; overflow-y: auto; padding: 12px; }
.pf-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 8px; }
.pf-card { padding: 12px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 6px; cursor: pointer; transition: all 0.12s; }
.pf-card:hover { border-color: var(--color-accent); }
.pf-icon { font-size: 20px; margin-bottom: 4px; }
.pf-name { font-size: 12px; font-weight: 500; color: var(--color-text-primary); }
.pf-note { font-size: 10px; color: var(--color-text-tertiary); margin-top: 2px; font-family: monospace; }
.pf-conn { font-size: 10px; color: var(--color-text-secondary); margin-top: 2px; }
.pf-status { font-size: 10px; color: var(--color-text-tertiary); margin-top: 4px; font-weight: 500; }
.pf-status.running { color: var(--color-success); }
.pf-empty { padding: 24px; text-align: center; color: var(--color-text-tertiary); font-size: 12px; line-height: 1.6; }
.toast-bar { padding: 6px 12px; font-size: 11px; color: var(--color-text-primary); background: var(--color-accent-bg); border-top: 1px solid var(--color-accent); cursor: pointer; flex-shrink: 0; }
.context-menu { position: fixed; z-index: 200; min-width: 150px; background: var(--color-bg-panel); border: 1px solid var(--color-border-primary); border-radius: 6px; box-shadow: var(--shadow-panel); padding: 4px; }
.ctx-item { padding: 6px 10px; font-size: 12px; color: var(--color-text-primary); border-radius: 4px; cursor: pointer; }
.ctx-item:hover { background: var(--color-bg-hover); }
.ctx-danger { color: var(--color-danger); }
.ctx-sep { height: 1px; background: var(--color-border-secondary); margin: 3px 6px; }
</style>
