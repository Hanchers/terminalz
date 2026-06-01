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

    <!-- Hosts 列表 -->
    <div v-if="!collapsed && activeMenu === 'hosts'" class="hosts-panel">
      <div class="hosts-header">
        <span>Saved Hosts</span>
        <button class="btn-refresh" @click="loadList" title="Refresh">↻</button>
      </div>
      <div v-if="savedList.length === 0" class="hosts-empty">
        No saved connections
      </div>
      <div v-else class="hosts-list">
        <div
          v-for="item in savedList"
          :key="item.id"
          class="hosts-item"
          :class="{ selected: selectedId === item.id }"
          @click="selectHost(item)"
        >
          <div class="host-icon">
            <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
              <path d="M2 20v-2h20v2H2zm0-6v-2h6v2H2zm0-6V6h6v2H2zm9 12v-2h5v2h-5zm3-6v-2h6v2h-6zm3-6V6h3v2h-3z"/>
            </svg>
          </div>
          <div class="host-info">
            <div class="host-name">{{ item.name || item.host }}</div>
            <div class="host-detail">{{ item.username }}@{{ item.host }}:{{ item.port }}</div>
          </div>
          <button class="btn-del" @click.stop="doDelete(item.id)" title="Delete">✕</button>
        </div>
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
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { currentTheme, themes, setTheme } from '../themes/index.js'

const props = defineProps({
  collapsed: { type: Boolean, default: false },
})

const emit = defineEmits(['select-host', 'toggle'])

const activeMenu = ref('hosts')
const savedList = ref([])
const selectedId = ref(0)

onMounted(() => {
  loadList()
})

async function loadList() {
  try {
    savedList.value = await invoke('list_connections')
  } catch (_) {
    savedList.value = []
  }
}

function onMenuClick(menu) {
  if (props.collapsed) {
    // 收起状态下点击图标 → 展开面板
    emit('toggle')
  }
  activeMenu.value = menu
}

function selectHost(item) {
  selectedId.value = item.id
  emit('select-host', {
    name: item.name || '',
    host: item.host,
    port: item.port,
    username: item.username,
    password: item.password,
    id: item.id,
  })
}

async function doDelete(id) {
  try {
    await invoke('delete_connection', { id })
    if (selectedId.value === id) selectedId.value = 0
    await loadList()
  } catch (_) {}
}
</script>

<style scoped>
.sidebar {
  width: 220px;
  min-width: 220px;
  height: 100%;
  background: var(--color-bg-panel);
  border-right: 1px solid var(--color-border-primary);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  user-select: none;
  transition: width 0.2s ease, min-width 0.2s ease;
}

.sidebar.collapsed {
  width: 48px;
  min-width: 48px;
}

/* 菜单列表 */
.menu-list {
  padding: 8px;
  border-bottom: 1px solid var(--color-border-secondary);
}

.sidebar.collapsed .menu-list {
  padding: 8px 4px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  border-radius: 6px;
  color: var(--color-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.sidebar.collapsed .menu-item {
  justify-content: center;
  padding: 7px 0;
}

.menu-item:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.menu-item.active {
  background: var(--color-bg-hover);
  color: var(--color-accent);
}

/* Hosts 面板 */
.hosts-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.hosts-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--color-border-secondary);
}

.btn-refresh {
  width: 24px;
  height: 24px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-refresh:hover {
  background: var(--color-bg-hover-alt);
  color: var(--color-text-primary);
}

.hosts-empty {
  padding: 24px 12px;
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: 12px;
}

.hosts-list {
  flex: 1;
  overflow-y: auto;
  padding: 6px 8px;
}

.hosts-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
  margin-bottom: 2px;
}

.hosts-item:hover {
  background: var(--color-bg-hover);
}

.hosts-item.selected {
  background: var(--color-accent-bg);
}

.host-icon {
  color: var(--color-accent);
  flex-shrink: 0;
  display: flex;
}

.hosts-item.selected .host-icon {
  color: var(--color-text-white);
}

.host-info {
  flex: 1;
  overflow: hidden;
  min-width: 0;
}

.host-name {
  font-size: 13px;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 500;
}

.hosts-item.selected .host-name {
  color: var(--color-text-white);
}

.host-detail {
  font-size: 11px;
  color: var(--color-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.hosts-item.selected .host-detail {
  color: var(--color-accent-light);
}

.btn-del {
  width: 22px;
  height: 22px;
  padding: 0;
  background: transparent;
  color: var(--color-text-tertiary);
  border: none;
  border-radius: 3px;
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  opacity: 0;
}

.hosts-item:hover .btn-del {
  opacity: 1;
}

.btn-del:hover {
  background: var(--color-danger-hover);
  color: var(--color-text-white);
  opacity: 1;
}

/* ===== Settings 面板 ===== */
.settings-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.settings-header {
  padding: 10px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--color-border-secondary);
}

.settings-section {
  padding: 12px;
}

.settings-label {
  font-size: 11px;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.theme-options {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.theme-opt {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: transparent;
  border: 1px solid var(--color-border-secondary);
  border-radius: 6px;
  color: var(--color-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
  text-align: left;
}

.theme-opt:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.theme-opt.active {
  background: var(--color-accent-bg);
  border-color: var(--color-accent);
  color: var(--color-text-white);
}

.theme-icon {
  font-size: 16px;
}

.theme-name {
  font-weight: 500;
}
</style>
