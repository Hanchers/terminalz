<template>
  <div id="app-container">
    <div class="app-layout">
      <!-- 左侧面板：Hosts 菜单 -->
      <Sidebar
        :collapsed="sidebarCollapsed"
        @select-host="onSelectHost"
        @select-local="onSelectLocal"
        @toggle="sidebarCollapsed = !sidebarCollapsed"
      />

      <!-- 左侧切换按钮 -->
      <button
        class="edge-toggle toggle-left"
        @click="sidebarCollapsed = !sidebarCollapsed"
        :title="sidebarCollapsed ? $t('app.expandSidebar') : $t('app.collapseSidebar')"
      >
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <path v-if="sidebarCollapsed" d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
          <path v-else d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/>
        </svg>
      </button>

      <!-- 中间区域：终端 -->
      <div class="main-area">
        <Terminal
          :prefill="selectedHost"
          :mode="terminalMode"
          @connection-change="onConnectionChange"
        />
      </div>

      <!-- 右侧切换按钮 -->
      <button
        v-if="isConnected"
        class="edge-toggle toggle-right"
        @click="statusCollapsed = !statusCollapsed"
        :title="statusCollapsed ? $t('app.expandStatus') : $t('app.collapseStatus')"
      >
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <path v-if="statusCollapsed" d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/>
          <path v-else d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
        </svg>
      </button>

      <!-- 右侧面板：服务器状态 -->
      <ServerStatus
        v-if="isConnected"
        :active="isConnected"
        :collapsed="statusCollapsed"
        :connection-mode="terminalMode"
        @toggle="statusCollapsed = !statusCollapsed"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Sidebar from './components/Sidebar.vue'
import Terminal from './components/Terminal.vue'
import ServerStatus from './components/ServerStatus.vue'

interface HostConfig { id: number; name?: string; host: string; port: number; username: string; remark?: string }

const isConnected = ref(false)
const selectedHost = ref<HostConfig | null>(null)
const terminalMode = ref<'ssh' | 'local' | null>(null)
const sidebarCollapsed = ref(false)
const statusCollapsed = ref(false)

function onSelectHost(config: HostConfig) {
  selectedHost.value = { ...config }
  terminalMode.value = 'ssh'
}

function onSelectLocal() {
  selectedHost.value = null
  terminalMode.value = 'local'
}

function onConnectionChange(connected: boolean) {
  isConnected.value = connected
  if (!connected) {
    selectedHost.value = null
    terminalMode.value = null
  }
}
</script>

<style>
/* ---- 左-中-右 布局 ---- */
.app-layout {
  display: flex;
  width: 100%;
  height: 100%;
}

.main-area {
  flex: 1;
  min-width: 0;
  height: 100%;
  overflow: hidden;
}

/* ---- 边缘切换按钮 ---- */
.edge-toggle {
  width: 20px;
  min-width: 20px;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all 0.15s;
  z-index: 5;
}

.edge-toggle:hover {
  background: var(--color-bg-hover-alt);
  color: var(--color-text-secondary);
}

.toggle-left {
  border-right: 1px solid var(--color-border-primary);
}

.toggle-right {
  border-left: 1px solid var(--color-border-primary);
}
</style>
