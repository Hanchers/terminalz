<template>
  <div id="app-container">
    <div class="app-layout">
      <!-- 左侧面板：Hosts 菜单 -->
      <Sidebar
        :collapsed="sidebarCollapsed"
        @select-host="onSelectHost"
        @toggle="sidebarCollapsed = !sidebarCollapsed"
      />

      <!-- 左侧切换按钮 -->
      <button
        class="edge-toggle toggle-left"
        @click="sidebarCollapsed = !sidebarCollapsed"
        :title="sidebarCollapsed ? '展开侧栏' : '收起侧栏'"
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
          @connection-change="onConnectionChange"
        />
      </div>

      <!-- 右侧切换按钮 -->
      <button
        v-if="isConnected"
        class="edge-toggle toggle-right"
        @click="statusCollapsed = !statusCollapsed"
        :title="statusCollapsed ? '展开状态栏' : '收起状态栏'"
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
        @toggle="statusCollapsed = !statusCollapsed"
      />
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import Sidebar from './components/Sidebar.vue'
import Terminal from './components/Terminal.vue'
import ServerStatus from './components/ServerStatus.vue'

const isConnected = ref(false)
const selectedHost = ref(null)
const sidebarCollapsed = ref(false)
const statusCollapsed = ref(false)

function onSelectHost(config) {
  selectedHost.value = { ...config }
}

function onConnectionChange(connected) {
  isConnected.value = connected
  if (!connected) selectedHost.value = null
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
