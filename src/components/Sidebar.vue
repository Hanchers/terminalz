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
        <span v-if="!collapsed">{{ $t('sidebar.hosts') }}</span>
      </div>
      <div
        class="menu-item"
        :class="{ active: activeMenu === 'settings' }"
        @click="onMenuClick('settings')"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
          <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.33-.02-.64-.06-.94l2.03-1.58a.49.49 0 00.12-.61l-1.92-3.32a.49.49 0 00-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 00-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.3-.06.61-.06.94s.02.64.06.94l-2.03 1.58a.49.49 0 00-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6A3.6 3.6 0 1115.6 12 3.605 3.605 0 0112 15.6z"/>
        </svg>
        <span v-if="!collapsed">{{ $t('sidebar.settings') }}</span>
      </div>
    </div>

    <!-- Hosts 树形面板 -->
    <div v-if="!collapsed && activeMenu === 'hosts'" class="hosts-panel">
      <div class="hosts-header">
        <span>{{ $t('sidebar.hosts') }}</span>
        <div class="header-actions">
          <button class="btn-refresh" @click="loadAll" :title="$t('sidebar.refresh')">↻</button>
          <button class="btn-tags" @click.stop="openTagDialog" :title="$t('sidebar.manageTags')">🏷</button>
          <button class="btn-add" @click.stop="openLocalTerminal" :title="$t('sidebar.localTerminal')">💻</button>
          <button class="btn-add" @click.stop="openAddMenu" :title="$t('sidebar.add')">+</button>
          <div v-if="showAddMenu" class="mini-dropdown" @click.stop>
            <div class="mini-item" @click="openGroupDialog()">{{ $t('sidebar.newGroup') }}</div>
            <div class="mini-item" @click="openHostDialog()">{{ $t('sidebar.newHost') }}</div>
          </div>
        </div>
      </div>

      <!-- 无数据显示 -->
      <div v-if="!groups.length && !connections.length" class="hosts-empty">
        {{ $t('sidebar.noHosts') }}
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
          :host-tags="hostTagsMap"
          @toggle-group="toggleGroup"
          @select-host="selectHost"
          @ctx-group="onCtxGroup"
          @ctx-host="onCtxHost"
        />
      </div>
    </div>

    <!-- Settings 面板 -->
    <div v-if="!collapsed && activeMenu === 'settings'" class="settings-panel">
      <div class="settings-header">{{ $t('sidebar.settings') }}</div>
      <div class="settings-section">
        <div class="settings-label">{{ $t('sidebar.language') }}</div>
        <select class="settings-select" :value="currentLocale" @change="switchLanguage(($event.target as HTMLSelectElement).value as SupportedLocale)">
          <option v-for="l in languageOptions" :key="l.id" :value="l.id">{{ l.icon }} {{ l.name }}</option>
        </select>
      </div>
      <div class="settings-section">
        <div class="settings-label">{{ $t('sidebar.theme') }}</div>
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
    <HostDialog
      :host-dialog="hostDialog"
      :all-tags="allTags"
      :flat-group-options="flatGroupOptions"
      @save="saveHostDialog"
      @cancel="closeHostDialog"
      @saved="loadAll"
    />

    <!-- ======== 弹窗：编辑/新建 Group ======== -->
    <GroupDialog
      :group-dialog="groupDialog"
      :group-select-options="groupSelectOptions"
      @save="saveGroupDialog"
      @cancel="closeGroupDialog"
    />

    <!-- ======== 弹窗：管理标签 ======== -->
    <TagDialog
      :tag-dialog="tagDialog"
      :all-tags="allTags"
      @cancel="closeTagDialog"
      @saved="loadAll"
    />

    <!-- ======== 右键菜单 ======== -->
    <div
      v-if="ctxMenu.visible"
      class="context-menu"
      :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
    >
      <template v-if="ctxMenu.type === 'group'">
        <div class="ctx-item" @click="editGroup(ctxMenu.id)">{{ $t('sidebar.ctxMenu.editGroup') }}</div>
        <div class="ctx-item" @click="openHostDialog(ctxMenu.id)">{{ $t('sidebar.ctxMenu.newHost') }}</div>
        <div class="ctx-item" @click="openGroupDialog(ctxMenu.id)">{{ $t('sidebar.ctxMenu.newSubgroup') }}</div>
        <div class="ctx-sep"></div>
        <div class="ctx-item ctx-danger" @click="tryDeleteGroup(ctxMenu.id)">{{ $t('sidebar.ctxMenu.deleteGroup') }}</div>
      </template>
      <template v-else-if="ctxMenu.type === 'host'">
        <div class="ctx-item" @click="editHost(ctxMenu.id)">{{ $t('sidebar.ctxMenu.editHost') }}</div>
        <div class="ctx-sep"></div>
        <div class="ctx-item ctx-danger" @click="deleteHost(ctxMenu.id)">{{ $t('sidebar.ctxMenu.deleteHost') }}</div>
      </template>
      <template v-else>
        <div class="ctx-item" @click="openGroupDialog()">{{ $t('sidebar.ctxMenu.newGroup') }}</div>
        <div class="ctx-item" @click="openHostDialog()">{{ $t('sidebar.ctxMenu.newHost') }}</div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { currentTheme, themes, setTheme } from '../themes/index'
import { locales, saveLocale, type SupportedLocale } from '../i18n'
import TreeNode from './TreeNode.vue'
import HostDialog from './HostDialog.vue'
import GroupDialog from './GroupDialog.vue'
import TagDialog from './TagDialog.vue'

const { t, locale } = useI18n()

interface Group { id: number; parent_id: number; name: string; remark: string }
interface Connection { id: number; name: string; host: string; port: number; username: string; password: string; group_id: number; remark: string }
interface Tag { id: number; name: string; color: string }
interface FlatOption { id: number; label: string; disabled?: boolean }
interface CtxMenu { visible: boolean; x: number; y: number; type: string; id: number }
interface HostDialog { visible: boolean; editingId: number; name: string; host: string; port: number; username: string; password: string; groupId: number; tagIds: number[]; remark: string }
interface GroupDialog { visible: boolean; editingId: number; name: string; parentId: number; remark: string }
interface TagDialog { visible: boolean; name: string; color: string }

const props = defineProps<{ collapsed: boolean }>()
const emit = defineEmits<{ 'select-host': [config: Record<string, any>]; 'select-local': []; 'toggle': [] }>()

const activeMenu = ref('hosts')
const selectedId = ref(0)
const connections = ref<Connection[]>([])
const groups = ref<Group[]>([])
const collapsedGroups = ref(new Set<number>())
const showAddMenu = ref(false)

// ---- 主题 ----

const languageOptions = locales
const currentLocale = computed(() => locale.value as SupportedLocale)

function switchLanguage(id: SupportedLocale): void {
  locale.value = id
  saveLocale(id)
}

const ctxMenu = reactive<CtxMenu>({ visible: false, x: 0, y: 0, type: '', id: 0 })

const hostDialog = reactive<HostDialog>({
  visible: false, editingId: 0,
  name: '', host: '', port: 22, username: '', password: '', groupId: 0, tagIds: [], remark: ''
})
const groupDialog = reactive<GroupDialog>({
  visible: false, editingId: 0,
  name: '', parentId: 0, remark: ''
})
const tagDialog = reactive<TagDialog>({
  visible: false, name: '', color: '#3fb950'
})

const allTags = ref<Tag[]>([])
const hostTagsMap = ref<Map<number, Tag[]>>(new Map())

onMounted(() => { loadAll() })

onUnmounted(() => {
  window.removeEventListener('click', hideCtxAndMenus)
})

async function loadAll(): Promise<void> {
  // Parallel loading: connections, groups, tags fetched concurrently
  const [conns, grps, tags] = await Promise.all([
    invoke<Connection[]>('list_connections').catch(() => [] as Connection[]),
    invoke<Group[]>('list_groups').catch(() => [] as Group[]),
    invoke<Tag[]>('list_tags').catch(() => [] as Tag[]),
  ])
  connections.value = conns
  groups.value = grps
  allTags.value = tags
  // Batch load host tags (single query instead of N individual calls)
  try {
    const ids = connections.value.map(c => c.id)
    if (ids.length > 0) {
      const result = await invoke<Record<number, Tag[]>>('list_all_host_tags', { hostIds: ids })
      const map = new Map<number, Tag[]>()
      for (const c of connections.value) {
        map.set(c.id, result[c.id] || [])
      }
      hostTagsMap.value = map
    }
  } catch (_) { hostTagsMap.value = new Map() }
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

async function openHostDialog(groupId = 0): Promise<void> {
  showAddMenu.value = false
  ctxMenu.visible = false
  Object.assign(hostDialog, {
    visible: true, editingId: 0, tagIds: [],
    name: '', host: '', port: 22, username: '', password: '', groupId, remark: ''
  })
}

async function editHost(id: number): Promise<void> {
  ctxMenu.visible = false
  const c = connections.value.find(x => x.id === id)
  if (!c) return
  // 加载该 host 的标签
  let tagIds: number[] = []
  try { const tags = await invoke<Tag[]>('get_host_tags', { hostId: id }); tagIds = tags.map(t => t.id) } catch (_) {}
  Object.assign(hostDialog, {
    visible: true, editingId: c.id, tagIds,
    name: c.name || '', host: c.host, port: c.port || 22,
    username: c.username, password: c.password, groupId: c.group_id || 0, remark: c.remark || ''
  })
}

async function saveHostDialog(): Promise<void> {
  if (!hostDialog.host || !hostDialog.username) return
  try {
    const saved = await invoke<{ id: number }>('save_connection', { config: {
      id: hostDialog.editingId,
      name: hostDialog.name || `${hostDialog.username}@${hostDialog.host}`,
      host: hostDialog.host, port: hostDialog.port,
      username: hostDialog.username, password: hostDialog.password,
      group_id: hostDialog.groupId,
      remark: hostDialog.remark
    }})
    // 保存标签关联
    const hostId = saved.id
    await invoke('set_host_tags', { hostId, tagIds: hostDialog.tagIds }).catch(() => {})
    closeHostDialog()
    await loadAll()
  } catch (e) { alert(t('sidebar.error.saveFailed') + e) }
}

function closeHostDialog(): void { hostDialog.visible = false }

async function deleteHost(id: number): Promise<void> {
  ctxMenu.visible = false
  if (!confirm('Delete this host?')) return
  try { await invoke('delete_connection', { id }); await loadAll() } catch (e) { alert(String(e)) }
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
  } catch (e) { alert(t('sidebar.error.saveFailed') + e) }
}

function closeGroupDialog(): void { groupDialog.visible = false }

// ---- 标签管理 ----

function openTagDialog(): void {
  tagDialog.visible = true
  tagDialog.name = ''
  tagDialog.color = '#3fb950'
}

function closeTagDialog(): void { tagDialog.visible = false }

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
.settings-select {
  width: 100%; padding: 8px 10px; margin-top: 4px;
  font-size: 13px; background: var(--color-bg-input);
  border: 1px solid var(--color-border-input); border-radius: 6px;
  color: var(--color-text-primary); outline: none; cursor: pointer;
}
.settings-select:focus { border-color: var(--color-accent); }
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

.btn-tags {
  width: 24px; height: 24px; background: transparent; border: none; border-radius: 4px;
  color: var(--color-text-secondary); cursor: pointer; font-size: 12px;
  display: flex; align-items: center; justify-content: center;
}
.btn-tags:hover { background: var(--color-bg-hover-alt); color: var(--color-text-primary); }
</style>