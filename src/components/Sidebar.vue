<template>
  <div class="sidebar" :class="{ collapsed: collapsed }">
    <!-- 菜单列表 -->
    <div class="menu-list">
      <div
        class="menu-item"
        :class="{ active: activeMenu === 'hosts' }"
        @click="onMenuClick('hosts')"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
          <path d="M4 6h16v2H4zm0 5h16v2H4zm0 5h16v2H4z"/>
        </svg>
        <span v-if="!collapsed">Hosts</span>
      </div>
      <div
        class="menu-item"
        :class="{ active: activeMenu === 'settings' }"
        @click="onMenuClick('settings')"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
          <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.33-.02-.64-.06-.94l2.03-1.58a.49.49 0 00.12-.61l-1.92-3.32a.49.49 0 00-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 00-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.3-.06.61-.06.94s.02.64.06.94l-2.03 1.58a.49.49 0 00-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6A3.6 3.6 0 1115.6 12 3.605 3.605 0 0112 15.6z"/>
        </svg>
        <span v-if="!collapsed">Settings</span>
      </div>
    </div>

    <!-- Hosts 树形面板 -->
    <div v-if="!collapsed && activeMenu === 'hosts'" class="hosts-panel">
      <div class="hosts-header">
        <span>Hosts</span>
        <div class="header-actions">
          <button class="btn-refresh" @click="loadAll" title="Refresh">↻</button>
          <button class="btn-add" @click.stop="openLocalTerminal" title="Local Terminal">💻</button>
          <button class="btn-add" @click.stop="openAddMenu" title="Add">+</button>
          <div v-if="showAddMenu" class="mini-dropdown" @click.stop>
            <div class="mini-item" @click="openGroupDialog()">New Group</div>
            <div class="mini-item" @click="openHostDialog()">New Host</div>
          </div>
        </div>
      </div>

      <!-- 无数据显示 -->
      <div v-if="!groups.length && !connections.length" class="hosts-empty">
        No hosts or groups — click + to add
      </div>

      <!-- 树形列表 -->
      <div class="hosts-tree" @click="showAddMenu = false">
        <TreeNode
          :groups="groups"
          :connections="connections"
          :selected-id="selectedId"
          :parent-id="0"
          :depth="0"
          :collapsed-groups="collapsedGroups"
          @toggle-group="toggleGroup"
          @select-host="selectHost"
          @ctx-group="onCtxGroup"
          @ctx-host="onCtxHost"
        />
      </div>
    </div>

    <!-- Settings 面板 -->
    <div v-if="!collapsed && activeMenu === 'settings'" class="settings-panel">
      <div class="settings-header">Settings</div>
      <div class="settings-section">
        <div class="settings-label">Theme</div>
        <div class="theme-options">
          <button
            v-for="t in themes"
            :key="t.id"
            class="theme-opt"
            :class="{ active: currentTheme === t.id }"
            @click="setTheme(t.id)"
          >
            <span class="theme-icon">{{ t.icon }}</span>
            <span class="theme-name">{{ t.name }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- ======== 弹窗：编辑/新建 Host ======== -->
    <div v-if="hostDialog.visible" class="modal-overlay" @click.self="closeHostDialog">
      <div class="modal-box">
        <div class="modal-title">{{ hostDialog.editingId ? 'Edit' : 'New' }} Host</div>
        <div class="modal-field">
          <label>Name</label>
          <input v-model="hostDialog.name" placeholder="My Server" @keyup.enter="saveHostDialog" />
        </div>
        <div class="modal-field">
          <label>Host</label>
          <input v-model="hostDialog.host" placeholder="192.168.1.1" @keyup.enter="saveHostDialog" />
        </div>
        <div class="modal-row">
          <div class="modal-field small">
            <label>Port</label>
            <input v-model.number="hostDialog.port" type="number" placeholder="22" />
          </div>
          <div class="modal-field">
            <label>Username</label>
            <input v-model="hostDialog.username" placeholder="root" @keyup.enter="saveHostDialog" />
          </div>
        </div>
        <div class="modal-field">
          <label>Password</label>
          <div class="password-wrap">
            <input
              v-model="hostDialog.password"
              :type="showHostPwd ? 'text' : 'password'"
              placeholder="password"
              @keyup.enter="saveHostDialog"
            />
            <button class="eye-btn" type="button" @click="showHostPwd = !showHostPwd" tabindex="-1">
              <svg v-if="showHostPwd" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/>
              </svg>
              <svg v-else viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                <path d="M12 7c2.76 0 5 2.24 5 5 0 .65-.13 1.26-.36 1.83l2.92 2.92c1.51-1.26 2.7-2.89 3.43-4.75-1.73-4.39-6-7.5-11-7.5-1.4 0-2.74.25-3.98.7l2.16 2.16C10.74 7.13 11.35 7 12 7zM2 4.27l2.28 2.28.46.46C3.08 8.3 1.78 10.02 1 12c1.73 4.39 6 7.5 11 7.5 1.55 0 3.03-.3 4.38-.84l.42.42L19.73 22 21 20.73 3.27 3 2 4.27zM7.53 9.8l1.55 1.55c-.05.21-.08.43-.08.65 0 1.66 1.34 3 3 3 .22 0 .44-.03.65-.08l1.55 1.55c-.67.33-1.41.53-2.2.53-2.76 0-5-2.24-5-5 0-.79.2-1.53.53-2.2zm4.31-.78l3.15 3.15.02-.16c0-1.66-1.34-3-3-3l-.17.01z"/>
              </svg>
            </button>
          </div>
        </div>
        <div class="modal-field">
          <label>Group</label>
          <select v-model="hostDialog.groupId">
            <option :value="0">(No Group)</option>
            <option v-for="g in flatGroupOptions" :key="g.id" :value="g.id">
              {{ g.label }}
            </option>
          </select>
        </div>
        <div class="modal-actions">
          <button class="modal-btn cancel" @click="closeHostDialog">Cancel</button>
          <button class="modal-btn primary" @click="saveHostDialog">Save</button>
        </div>
      </div>
    </div>

    <!-- ======== 弹窗：编辑/新建 Group ======== -->
    <div v-if="groupDialog.visible" class="modal-overlay" @click.self="closeGroupDialog">
      <div class="modal-box">
        <div class="modal-title">{{ groupDialog.editingId ? 'Edit' : 'New' }} Group</div>
        <div class="modal-field">
          <label>Name</label>
          <input v-model="groupDialog.name" placeholder="Production" @keyup.enter="saveGroupDialog" />
        </div>
        <div class="modal-field">
          <label>Parent Group</label>
          <select v-model="groupDialog.parentId">
            <option :value="0">(Root)</option>
            <option v-for="g in groupSelectOptions" :key="g.id" :value="g.id" :disabled="g.disabled">
              {{ g.label }}
            </option>
          </select>
        </div>
        <div class="modal-field">
          <label>Remark</label>
          <textarea v-model="groupDialog.remark" placeholder="Optional notes..." rows="2"></textarea>
        </div>
        <div class="modal-actions">
          <button class="modal-btn cancel" @click="closeGroupDialog">Cancel</button>
          <button class="modal-btn primary" @click="saveGroupDialog">Save</button>
        </div>
      </div>
    </div>

    <!-- ======== 右键菜单 ======== -->
    <div
      v-if="ctxMenu.visible"
      class="context-menu"
      :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
    >
      <template v-if="ctxMenu.type === 'group'">
        <div class="ctx-item" @click="editGroup(ctxMenu.id)">Edit Group</div>
        <div class="ctx-item" @click="openHostDialog(ctxMenu.id)">New Host</div>
        <div class="ctx-item" @click="openGroupDialog(ctxMenu.id)">New Subgroup</div>
        <div class="ctx-sep"></div>
        <div class="ctx-item ctx-danger" @click="tryDeleteGroup(ctxMenu.id)">Delete Group</div>
      </template>
      <template v-else-if="ctxMenu.type === 'host'">
        <div class="ctx-item" @click="editHost(ctxMenu.id)">Edit Host</div>
        <div class="ctx-sep"></div>
        <div class="ctx-item ctx-danger" @click="deleteHost(ctxMenu.id)">Delete Host</div>
      </template>
      <template v-else>
        <div class="ctx-item" @click="openGroupDialog()">New Group</div>
        <div class="ctx-item" @click="openHostDialog()">New Host</div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { currentTheme, themes, setTheme } from '../themes/index'
import TreeNode from './TreeNode.vue'

interface Group { id: number; parent_id: number; name: string; remark: string }
interface Connection { id: number; name: string; host: string; port: number; username: string; password: string; group_id: number }
interface FlatOption { id: number; label: string }
interface CtxMenu { visible: boolean; x: number; y: number; type: string; id: number }
interface HostDialog { visible: boolean; editingId: number; name: string; host: string; port: number; username: string; password: string; groupId: number }
interface GroupDialog { visible: boolean; editingId: number; name: string; parentId: number; remark: string }

const props = defineProps<{ collapsed: boolean }>()
const emit = defineEmits<{ 'select-host': [config: Record<string, any>]; 'select-local': []; 'toggle': [] }>()

const activeMenu = ref('hosts')
const selectedId = ref(0)
const connections = ref<Connection[]>([])
const groups = ref<Group[]>([])
const collapsedGroups = ref(new Set<number>())
const showAddMenu = ref(false)

const ctxMenu = reactive<CtxMenu>({ visible: false, x: 0, y: 0, type: '', id: 0 })

const showHostPwd = ref(false)
const hostDialog = reactive<HostDialog>({
  visible: false, editingId: 0,
  name: '', host: '', port: 22, username: '', password: '', groupId: 0
})
const groupDialog = reactive<GroupDialog>({
  visible: false, editingId: 0,
  name: '', parentId: 0, remark: ''
})

onMounted(() => { loadAll() })

async function loadAll(): Promise<void> {
  try { connections.value = await invoke<Connection[]>('list_connections') } catch (_) { connections.value = [] }
  try { groups.value = await invoke<Group[]>('list_groups') } catch (_) { groups.value = [] }
}

function toggleGroup(id: number): void {
  const s = new Set(collapsedGroups.value)
  s.has(id) ? s.delete(id) : s.add(id)
  collapsedGroups.value = s
}

function selectHost(item: Connection): void {
  selectedId.value = item.id
  emit('select-host', { ...item, name: item.name || item.host })
}

const flatGroupOptions = computed<FlatOption[]>(() => flattenGroups(groups.value, 0, 0))

function flattenGroups(list: Group[], parentId: number, depth: number): FlatOption[] {
  let result: FlatOption[] = []
  for (const g of list) {
    if (g.parent_id !== parentId) continue
    result.push({ id: g.id, label: '  '.repeat(depth) + g.name })
    result.push(...flattenGroups(list, g.id, depth + 1))
  }
  return result
}

const groupSelectOptions = computed<FlatOption[]>(() => {
  if (!groupDialog.editingId) return flatGroupOptions.value
  const excludeIds = new Set([groupDialog.editingId])
  function collectDescendants(pid: number): void {
    for (const g of groups.value) {
      if (g.parent_id === pid) { excludeIds.add(g.id); collectDescendants(g.id) }
    }
  }
  collectDescendants(groupDialog.editingId)
  return flatGroupOptions.value.filter(o => !excludeIds.has(o.id))
})

function openHostDialog(groupId = 0): void {
  showAddMenu.value = false
  ctxMenu.visible = false
  Object.assign(hostDialog, {
    visible: true, editingId: 0,
    name: '', host: '', port: 22, username: '', password: '', groupId
  })
}

async function editHost(id: number): Promise<void> {
  ctxMenu.visible = false
  const c = connections.value.find(x => x.id === id)
  if (!c) return
  Object.assign(hostDialog, {
    visible: true, editingId: c.id,
    name: c.name || '', host: c.host, port: c.port || 22,
    username: c.username, password: c.password, groupId: c.group_id || 0
  })
}

async function saveHostDialog(): Promise<void> {
  if (!hostDialog.host || !hostDialog.username) return
  try {
    await invoke('save_connection', { config: {
      id: hostDialog.editingId,
      name: hostDialog.name || `${hostDialog.username}@${hostDialog.host}`,
      host: hostDialog.host, port: hostDialog.port,
      username: hostDialog.username, password: hostDialog.password,
      group_id: hostDialog.groupId
    }})
    closeHostDialog()
    await loadAll()
  } catch (e) { alert('Save failed: ' + e) }
}

function closeHostDialog(): void { hostDialog.visible = false; showHostPwd.value = false }

async function deleteHost(id: number): Promise<void> {
  ctxMenu.visible = false
  if (!confirm('Delete this host?')) return
  try { await invoke('delete_connection', { id }); await loadAll() } catch (e) { alert('' + e) }
}

function openGroupDialog(parentId = 0): void {
  showAddMenu.value = false
  ctxMenu.visible = false
  Object.assign(groupDialog, { visible: true, editingId: 0, name: '', parentId, remark: '' })
}

async function editGroup(id: number): Promise<void> {
  ctxMenu.visible = false
  const g = groups.value.find(x => x.id === id)
  if (!g) return
  Object.assign(groupDialog, { visible: true, editingId: g.id, name: g.name, parentId: g.parent_id, remark: g.remark || '' })
}

async function saveGroupDialog(): Promise<void> {
  if (!groupDialog.name) return
  try {
    await invoke('save_group', { group: {
      id: groupDialog.editingId,
      parent_id: groupDialog.parentId,
      name: groupDialog.name,
      remark: groupDialog.remark
    }})
    closeGroupDialog()
    await loadAll()
  } catch (e) { alert('Save failed: ' + e) }
}

function closeGroupDialog(): void { groupDialog.visible = false }

async function tryDeleteGroup(id: number): Promise<void> {
  ctxMenu.visible = false
  try {
    await invoke('delete_group', { id })
    await loadAll()
  } catch (e) { alert(e) }
}

function onCtxGroup(id: number, e: MouseEvent): void {
  ctxMenu.visible = true; ctxMenu.x = e.clientX; ctxMenu.y = e.clientY
  ctxMenu.type = 'group'; ctxMenu.id = id
}
function onCtxHost(id: number, e: MouseEvent): void {
  ctxMenu.visible = true; ctxMenu.x = e.clientX; ctxMenu.y = e.clientY
  ctxMenu.type = 'host'; ctxMenu.id = id
}

function openAddMenu(): void { showAddMenu.value = !showAddMenu.value }
function openLocalTerminal(): void {
  showAddMenu.value = false
  emit('select-local')
}
function onMenuClick(menu: string): void {
  if (props.collapsed) emit('toggle')
  activeMenu.value = menu
}

function hideCtxAndMenus(): void {
  ctxMenu.visible = false
  showAddMenu.value = false
}
if (typeof window !== 'undefined') {
  window.addEventListener('click', hideCtxAndMenus)
}
</script>

<style scoped>
.sidebar { width: 220px; min-width: 220px; height: 100%; background: var(--color-bg-panel); border-right: 1px solid var(--color-border-primary); display: flex; flex-direction: column; overflow: hidden; user-select: none; transition: width 0.2s ease, min-width 0.2s ease; }
.sidebar.collapsed { width: 48px; min-width: 48px; }
.menu-list { padding: 8px; border-bottom: 1px solid var(--color-border-secondary); }
.sidebar.collapsed .menu-list { padding: 8px 4px; }
.menu-item { display: flex; align-items: center; gap: 8px; padding: 7px 12px; border-radius: 6px; color: var(--color-text-secondary); font-size: 13px; cursor: pointer; transition: all 0.15s; }
.sidebar.collapsed .menu-item { justify-content: center; padding: 7px 0; }
.menu-item:hover { background: var(--color-bg-hover); color: var(--color-text-primary); }
.menu-item.active { background: var(--color-bg-hover); color: var(--color-accent); }
.hosts-panel { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.hosts-header { display: flex; align-items: center; justify-content: space-between; padding: 10px 12px; font-size: 11px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.5px; border-bottom: 1px solid var(--color-border-secondary); }
.btn-refresh { width: 24px; height: 24px; background: transparent; border: none; border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; font-size: 14px; display: flex; align-items: center; justify-content: center; }
.btn-refresh:hover { background: var(--color-bg-hover-alt); color: var(--color-text-primary); }
.hosts-empty { padding: 24px 12px; text-align: center; color: var(--color-text-tertiary); font-size: 12px; }
.settings-panel { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.settings-header { padding: 10px 12px; font-size: 11px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.5px; border-bottom: 1px solid var(--color-border-secondary); }
.settings-section { padding: 12px; }
.settings-label { font-size: 11px; color: var(--color-text-tertiary); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px; }
.theme-options { display: flex; flex-direction: column; gap: 4px; }
.theme-opt { display: flex; align-items: center; gap: 8px; padding: 8px 10px; background: transparent; border: 1px solid var(--color-border-secondary); border-radius: 6px; color: var(--color-text-secondary); font-size: 13px; cursor: pointer; transition: all 0.15s; text-align: left; }
.theme-opt:hover { background: var(--color-bg-hover); color: var(--color-text-primary); }
.theme-opt.active { background: var(--color-accent-bg); border-color: var(--color-accent); color: var(--color-text-white); }
.theme-icon { font-size: 16px; }
.theme-name { font-weight: 500; }
.hosts-tree { flex: 1; overflow-y: auto; padding: 4px 0; }
.header-actions { display: flex; align-items: center; gap: 2px; position: relative; }
.btn-add { width: 24px; height: 24px; background: transparent; border: none; border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; font-size: 16px; display: flex; align-items: center; justify-content: center; }
.btn-add:hover { background: var(--color-bg-hover-alt); color: var(--color-text-primary); }
.mini-dropdown { position: absolute; right: 0; top: 100%; z-index: 20; min-width: 120px; background: var(--color-bg-panel); border: 1px solid var(--color-border-primary); border-radius: 6px; box-shadow: var(--shadow-panel); padding: 4px; }
.mini-item { padding: 6px 10px; font-size: 12px; color: var(--color-text-primary); border-radius: 4px; cursor: pointer; }
.mini-item:hover { background: var(--color-bg-hover); }
.mini-sep { height: 1px; background: var(--color-border-secondary); margin: 3px 6px; }
.context-menu { position: fixed; z-index: 200; min-width: 150px; background: var(--color-bg-panel); border: 1px solid var(--color-border-primary); border-radius: 6px; box-shadow: var(--shadow-panel); padding: 4px; }
.ctx-item { padding: 6px 10px; font-size: 12px; color: var(--color-text-primary); border-radius: 4px; cursor: pointer; }
.ctx-item:hover { background: var(--color-bg-hover); }
.ctx-danger { color: var(--color-danger); }
.ctx-sep { height: 1px; background: var(--color-border-secondary); margin: 3px 6px; }
.modal-overlay { position: fixed; inset: 0; z-index: 300; background: var(--shadow-overlay); display: flex; align-items: center; justify-content: center; }
.modal-box { width: 400px; max-height: 80vh; overflow-y: auto; background: var(--color-bg-panel); border: 1px solid var(--color-border-primary); border-radius: 10px; box-shadow: var(--shadow-panel); padding: 20px; }
.modal-title { font-size: 15px; font-weight: 600; color: var(--color-text-primary); margin-bottom: 16px; }
.modal-field { margin-bottom: 12px; }
.modal-field label { display: block; font-size: 11px; color: var(--color-text-tertiary); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px; }
.modal-field input, .modal-field select, .modal-field textarea { width: 100%; padding: 7px 10px; font-size: 13px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 5px; color: var(--color-text-primary); outline: none; box-sizing: border-box; font-family: inherit; }
.modal-field input:focus, .modal-field select:focus, .modal-field textarea:focus { border-color: var(--color-accent); }
.modal-field textarea { resize: vertical; }
.modal-row { display: flex; gap: 10px; }
.modal-row .modal-field.small { flex: 0 0 100px; }
.modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }
.modal-btn { padding: 7px 18px; font-size: 13px; border: none; border-radius: 5px; cursor: pointer; }
.modal-btn.cancel { background: var(--color-bg-hover-alt); color: var(--color-text-primary); }
.modal-btn.cancel:hover { background: var(--color-bg-active); }
.modal-btn.primary { background: var(--color-btn-primary); color: var(--color-text-white); }
.modal-btn.primary:hover { background: var(--color-btn-primary-hover); }
.password-wrap { position: relative; display: flex; }
.password-wrap input { flex: 1; padding-right: 32px; }
.eye-btn { position: absolute; right: 2px; top: 50%; transform: translateY(-50%); width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; background: transparent; border: none; border-radius: 4px; color: var(--color-text-tertiary); cursor: pointer; }
.eye-btn:hover { color: var(--color-text-secondary); }
</style>