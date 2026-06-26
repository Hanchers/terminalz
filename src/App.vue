<template>
  <div id="app-container">
    <div class="app-layout">
      <!-- Tab Bar -->
      <TabBar
        v-model="activeTab"
        :tabs="openTabs"
        @close-tab="closeTab"
        @new-tab="newEmptyTab"
      />

      <!-- Content Area -->
      <div class="app-content">
        <!-- Vault Tab -->
        <VaultTab
          v-if="activeTab === 'vault'"
          @open-host="openHostTab"
          @connect-host="connectHost"
          @new-tab="newEmptyTab"
          @send-command="sendCommand"
        />

        <!-- SFTP Tab -->
        <SftpTab
          v-if="activeTab === 'sftp'"
          :active-host-id="activeTerminalId"
        />

        <!-- Terminal Tabs -->
        <TerminalView
          v-for="tab in openTabs"
          v-show="activeTab === tab.id"
          :key="tab.id"
          :tab="tab"
          @connection-change="(connected: boolean) => onTabConnection(tab.id, connected)"
          @close="closeTab(tab.id)"
        />

        <!-- Welcome (no terminals open) -->
        <div v-if="activeTab === 'vault' || activeTab === 'sftp' || openTabs.length === 0" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import TabBar from './components/TabBar.vue'
import VaultTab from './components/VaultTab.vue'
import SftpTab from './components/SftpTab.vue'
import TerminalView from './components/TerminalView.vue'

interface TabInfo { id: number; name?: string; host: string; port: number; username: string; connected: boolean; connecting: boolean }
interface HostConfig { id: number; name?: string; host: string; port: number; username: string; remark?: string }

const activeTab = ref<string | number>('vault')
const openTabs = ref<TabInfo[]>([])
const activeTerminalId = ref<number | null>(null)
let nextTabId = 1000

function openHostTab(config: HostConfig) {
  const existing = openTabs.value.find(t => t.id === config.id)
  if (existing) {
    activeTab.value = existing.id
    return
  }
  const tab: TabInfo = {
    id: config.id,
    name: config.name || config.host,
    host: config.host,
    port: config.port,
    username: config.username,
    connected: false,
    connecting: false,
  }
  openTabs.value.push(tab)
  activeTab.value = tab.id
  activeTerminalId.value = tab.id
}

function connectHost(config: HostConfig) {
  // Ensure tab exists
  let tab = openTabs.value.find(t => t.id === config.id)
  if (!tab) {
    tab = { id: config.id, name: config.name || config.host, host: config.host, port: config.port, username: config.username, connected: false, connecting: false }
    openTabs.value.push(tab)
  }
  tab.connecting = false // reset for re-connect
  activeTab.value = tab.id
  activeTerminalId.value = tab.id
}

async function sendCommand(command: string) {
  try {
    await invoke('ssh_write', { data: command + '\n' })
  } catch (e) {
    console.error('Failed to send command:', e)
  }
}

function newEmptyTab() {
  const id = nextTabId++
  const tab: TabInfo = { id, name: 'New Tab', host: 'localhost', port: 22, username: '', connected: false, connecting: false }
  openTabs.value.push(tab)
  activeTab.value = tab.id
}

function closeTab(id: number) {
  openTabs.value = openTabs.value.filter(t => t.id !== id)
  if (activeTab.value === id) {
    if (openTabs.value.length > 0) {
      const idx = openTabs.value.findIndex(t => t.id === id)
      const next = openTabs.value[Math.min(idx, openTabs.value.length - 1)]
      activeTab.value = next ? next.id : 'vault'
    } else {
      activeTab.value = 'vault'
    }
  }
  if (activeTerminalId.value === id) activeTerminalId.value = openTabs.value.length > 0 ? openTabs.value[openTabs.value.length - 1].id : null
}

function onTabConnection(id: number, connected: boolean) {
  const tab = openTabs.value.find(t => t.id === id)
  if (tab) { tab.connected = connected; tab.connecting = false }
  if (connected) activeTerminalId.value = id
}
</script>

<style>
#app-container { width: 100%; height: 100vh; overflow: hidden; }
.app-layout { display: flex; flex-direction: column; width: 100%; height: 100%; }
.app-content { flex: 1; min-height: 0; overflow: hidden; }
</style>
