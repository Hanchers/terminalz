<template>
  <div class="vault-layout">
    <VaultSidebar v-model:active="activePanel" />
    <div class="vault-content">
      <HostsPanel
        v-if="activePanel === 'hosts'"
        :groups="filteredGroups"
        :connections="filteredConnections"
        :host-tags-map="hostTagsMap"
        :current-group="currentGroup"
        :breadcrumb="breadcrumb"
        @open-host="onOpenHost"
        @navigate="navigateGroup"
        @new-group="openGroupDialog()"
        @new-host="openHostDialog()"
        @ctx-group="onCtxGroup"
        @ctx-host="onCtxHost"
      />
      <KeychainPanel v-if="activePanel === 'keychain'" />
      <PortForwardPanel v-if="activePanel === 'portforward'" />
      <SnippetsPanel v-if="activePanel === 'snippets'" />
      <SettingsPanel v-if="activePanel === 'settings'" @change-locale="onChangeLocale" />
    </div>

    <!-- Context Menu -->
    <div v-if="ctxMenu.visible" class="context-menu" :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }">
      <template v-if="ctxMenu.type === 'group'">
        <div class="ctx-item" @click="editGroup(ctxMenu.id)">Edit Group</div>
        <div class="ctx-item" @click="openHostDialog(ctxMenu.id)">New Host Here</div>
        <div class="ctx-item" @click="openGroupDialog(ctxMenu.id)">New Subgroup</div>
        <div class="ctx-sep"></div>
        <div class="ctx-item ctx-danger" @click="deleteGroup(ctxMenu.id)">Delete</div>
      </template>
      <template v-else-if="ctxMenu.type === 'host'">
        <div class="ctx-item" @click="editHost(ctxMenu.id)">Edit Host</div>
        <div class="ctx-sep"></div>
        <div class="ctx-item ctx-danger" @click="deleteHost(ctxMenu.id)">Delete</div>
      </template>
    </div>

    <!-- Dialogs -->
    <HostDialog
      ref="hostDialogRef"
      :host-dialog="hostDialog"
      :all-tags="allTags"
      :flat-group-options="flatGroupOptions"
      @save="saveHostDialog"
      @save-connect="saveConnectHost"
      @cancel="closeHostDialog"
      @saved="loadAll"
    />
    <GroupDialog
      :group-dialog="groupDialog"
      :group-select-options="groupSelectOptions"
      @save="saveGroupDialog"
      @cancel="closeGroupDialog"
    />
    <TagDialog
      :tag-dialog="tagDialog"
      :all-tags="allTags"
      @cancel="closeTagDialog"
      @saved="loadAll"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { saveLocale, type SupportedLocale } from '../i18n'
import VaultSidebar from './VaultSidebar.vue'
import HostsPanel from './HostsPanel.vue'
import KeychainPanel from './KeychainPanel.vue'
import PortForwardPanel from './PortForwardPanel.vue'
import SnippetsPanel from './SnippetsPanel.vue'
import SettingsPanel from './SettingsPanel.vue'
import HostDialog from './HostDialog.vue'
import GroupDialog from './GroupDialog.vue'
import TagDialog from './TagDialog.vue'

interface Group { id: number; parent_id: number; name: string; remark: string }
interface Connection { id: number; name: string; host: string; port: number; username: string; password: string; group_id: number; remark: string }
interface Tag { id: number; name: string; color: string }
interface FlatOption { id: number; label: string; disabled?: boolean }
interface CtxMenu { visible: boolean; x: number; y: number; type: string; id: number }
interface HostDialogState { visible: boolean; editingId: number; name: string; host: string; port: number; username: string; password: string; groupId: number; tagIds: number[]; remark: string }
interface GroupDialogState { visible: boolean; editingId: number; name: string; parentId: number; remark: string }
interface TagDialogState { visible: boolean; name: string; color: string }

const emit = defineEmits<{ 'open-host': [c: Record<string, any>]; 'connect-host': [c: Record<string, any>] }>()
const { locale } = useI18n({ useScope: 'global' })

const activePanel = ref('hosts')
const groups = ref<Group[]>([])
const connections = ref<Connection[]>([])
const allTags = ref<Tag[]>([])
const hostTagsMap = ref<Record<number, Tag[]>>({})
const currentGroup = ref(0)
const ctxMenu = reactive<CtxMenu>({ visible: false, x: 0, y: 0, type: '', id: 0 })

// ---- Breadcrumb ----
const breadcrumb = computed<Group[]>(() => {
  if (currentGroup.value === 0) return []
  const path: Group[] = []
  let id = currentGroup.value
  const visited = new Set<number>()
  while (id > 0 && !visited.has(id)) {
    visited.add(id)
    const g = groups.value.find(x => x.id === id)
    if (!g) break
    path.unshift(g)
    id = g.parent_id
  }
  return path
})

// ---- Filtered data for current group ----
const filteredGroups = computed(() => {
  if (currentGroup.value === 0) return groups.value.filter(g => g.parent_id === 0)
  return groups.value.filter(g => g.parent_id === currentGroup.value)
})
const filteredConnections = computed(() => {
  if (currentGroup.value === 0) return connections.value
  return connections.value.filter(c => c.group_id === currentGroup.value)
})

// ---- Dialogs ----
const hostDialog = reactive<HostDialogState>({ visible: false, editingId: 0, name: '', host: '', port: 22, username: '', password: '', groupId: 0, tagIds: [], remark: '' })
const hostDialogRef = ref<InstanceType<typeof HostDialog> | null>(null)
const groupDialog = reactive<GroupDialogState>({ visible: false, editingId: 0, name: '', parentId: 0, remark: '' })
const tagDialog = reactive<TagDialogState>({ visible: false, name: '', color: '#3fb950' })

const flatGroupOptions = computed<FlatOption[]>(() => flattenGroups(groups.value, 0, 0))

const groupSelectOptions = computed<FlatOption[]>(() => {
  if (!groupDialog.editingId) return flatGroupOptions.value
  const excludeIds = new Set([groupDialog.editingId])
  collectDescendants(groupDialog.editingId, excludeIds)
  return flatGroupOptions.value.filter(o => !excludeIds.has(o.id))
})

function collectDescendants(pid: number, set: Set<number>): void {
  for (const g of groups.value) { if (g.parent_id === pid) { set.add(g.id); collectDescendants(g.id, set) } }
}

function flattenGroups(list: Group[], parentId: number, depth: number): FlatOption[] {
  let result: FlatOption[] = []
  for (const g of list) {
    if (g.parent_id !== parentId) continue
    result.push({ id: g.id, label: '  '.repeat(depth) + g.name })
    result.push(...flattenGroups(list, g.id, depth + 1))
  }
  return result
}

// ---- Navigation & Locale ----
function navigateGroup(id: number) { currentGroup.value = id }

function onChangeLocale(id: string) {
  locale.value = id
  saveLocale(id as SupportedLocale)
}

function onOpenHost(c: Connection) {
  emit('open-host', { ...c, name: c.name || c.host })
}

// ---- Context menu ----
function onCtxGroup(id: number, e: MouseEvent) { ctxMenu.visible = true; ctxMenu.x = e.clientX; ctxMenu.y = e.clientY; ctxMenu.type = 'group'; ctxMenu.id = id }
function onCtxHost(id: number, e: MouseEvent) { ctxMenu.visible = true; ctxMenu.x = e.clientX; ctxMenu.y = e.clientY; ctxMenu.type = 'host'; ctxMenu.id = id }
window.addEventListener('click', () => { ctxMenu.visible = false })

// ---- Host CRUD ----
function openHostDialog(groupId = 0) {
  ctxMenu.visible = false
  Object.assign(hostDialog, { visible: true, editingId: 0, tagIds: [], name: '', host: '', port: 22, username: '', password: '', groupId, remark: '' })
}
async function editHost(id: number) {
  ctxMenu.visible = false
  const c = connections.value.find(x => x.id === id)
  if (!c) return
  let tagIds: number[] = []
  try { const tags = await invoke<Tag[]>('get_host_tags', { hostId: id }); tagIds = tags.map(t => t.id) } catch (_) {}
  Object.assign(hostDialog, { visible: true, editingId: c.id, tagIds, name: c.name || '', host: c.host, port: c.port || 22, username: c.username, password: '', groupId: c.group_id || 0, remark: c.remark || '' })
}
async function saveHostDialog(form: HostDialogState) {
  if (!form.host || !form.username) return
  try {
    const saved = await invoke<{ id: number }>('save_connection', { config: { id: form.editingId, name: form.name || `${form.username}@${form.host}`, host: form.host, port: form.port, username: form.username, password: form.password, group_id: form.groupId, remark: form.remark } })
    await invoke('set_host_tags', { hostId: saved.id, tagIds: form.tagIds }).catch(() => {})
    closeHostDialog()
    await loadAll()
  } catch (e) { alert('Save failed: ' + e) }
}
async function saveConnectHost(form: HostDialogState) {
  if (!form.host || !form.username) return
  hostDialogRef.value?.setConnecting(true)
  try {
    const saved = await invoke<{ id: number }>('save_connection', { config: { id: form.editingId, name: form.name || `${form.username}@${form.host}`, host: form.host, port: form.port, username: form.username, password: form.password, group_id: form.groupId, remark: form.remark } })
    await invoke('set_host_tags', { hostId: saved.id, tagIds: form.tagIds }).catch(() => {})
    closeHostDialog()
    await loadAll()
    const conn = connections.value.find(c => c.id === saved.id)
    if (conn) emit('connect-host', { ...conn, name: conn.name || conn.host })
  } catch (e) { hostDialogRef.value?.showError(String(e)) } finally { hostDialogRef.value?.setConnecting(false) }
}
function closeHostDialog() { hostDialog.visible = false }
async function deleteHost(id: number) { ctxMenu.visible = false; try { await invoke('delete_connection', { id }); await loadAll() } catch (e) { alert(String(e)) } }

// ---- Group CRUD ----
function openGroupDialog(parentId = 0) { ctxMenu.visible = false; Object.assign(groupDialog, { visible: true, editingId: 0, name: '', parentId, remark: '' }) }
async function editGroup(id: number) {
  ctxMenu.visible = false
  const g = groups.value.find(x => x.id === id)
  if (!g) return
  Object.assign(groupDialog, { visible: true, editingId: g.id, name: g.name, parentId: g.parent_id, remark: g.remark || '' })
}
async function saveGroupDialog() {
  if (!groupDialog.name) return
  try {
    await invoke('save_group', { group: { id: groupDialog.editingId, parent_id: groupDialog.parentId, name: groupDialog.name, remark: groupDialog.remark } })
    closeGroupDialog(); await loadAll()
  } catch (e) { alert('Save failed: ' + e) }
}
function closeGroupDialog() { groupDialog.visible = false }
async function deleteGroup(id: number) { ctxMenu.visible = false; try { await invoke('delete_group', { id }); await loadAll() } catch (e) { alert(e) } }

// ---- Tags ----
function closeTagDialog() { tagDialog.visible = false }

// ---- Data load ----
async function loadAll() {
  const [conns, grps, tags] = await Promise.all([
    invoke<Connection[]>('list_connections').catch(() => [] as Connection[]),
    invoke<Group[]>('list_groups').catch(() => [] as Group[]),
    invoke<Tag[]>('list_tags').catch(() => [] as Tag[]),
  ])
  connections.value = conns; groups.value = grps; allTags.value = tags
  try {
    const ids = conns.map(c => c.id)
    if (ids.length > 0) {
      const result = await invoke<Record<number, Tag[]>>('list_all_host_tags', { hostIds: ids })
      const map: Record<number, Tag[]> = {}
      for (const c of conns) map[c.id] = result[c.id] || []
      hostTagsMap.value = map
    }
  } catch (_) { hostTagsMap.value = {} }
}

onMounted(() => { loadAll() })
</script>

<style scoped>
.vault-layout { display: flex; height: 100%; overflow: hidden; }
.vault-content { flex: 1; min-width: 0; }

.context-menu { position: fixed; z-index: 200; min-width: 150px; background: var(--color-bg-panel); border: 1px solid var(--color-border-primary); border-radius: 6px; box-shadow: var(--shadow-panel); padding: 4px; }
.ctx-item { padding: 6px 10px; font-size: 12px; color: var(--color-text-primary); border-radius: 4px; cursor: pointer; }
.ctx-item:hover { background: var(--color-bg-hover); }
.ctx-danger { color: var(--color-danger); }
.ctx-sep { height: 1px; background: var(--color-border-secondary); margin: 3px 6px; }
</style>
