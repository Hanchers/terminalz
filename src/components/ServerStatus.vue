<template>
  <div class="status-panel" :class="{ collapsed: collapsed }">
    <!-- 展开模式 -->
    <template v-if="!collapsed">
      <div class="status-header">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <path d="M22 12c0 5.523-4.477 10-10 10S2 17.523 2 12 6.477 2 12 2s10 4.477 10 10zm-2 0a8 8 0 10-16 0 8 8 0 0016 0zm-8-6v6l4 2.5"/>
        </svg>
        <span>Status</span>
        <span v-if="polling" class="poll-dot"></span>
      </div>

      <div v-if="error" class="status-error">{{ error }}</div>

      <template v-else-if="info">
        <div class="info-card host-card">
          <div class="card-label">Hostname</div>
          <div class="card-value host-value">{{ info.hostname }}</div>
        </div>
        <div class="info-card">
          <div class="card-label">System</div>
          <div class="card-value sys-value">{{ info.os_name }}</div>
          <div class="card-sub">Kernel: {{ info.kernel }}</div>
        </div>
        <div class="info-card">
          <div class="card-label">Uptime</div>
          <div class="card-value grey">{{ info.uptime }}</div>
        </div>
        <div class="info-card">
          <div class="card-label">CPU</div>
          <div class="gauge-row">
            <span class="gauge-pct" :style="{ color: cpuColor }">{{ info.cpu_pct.toFixed(1) }}%</span>
            <div class="gauge-track">
              <div
                class="gauge-fill cpu-fill"
                :style="{ width: info.cpu_pct + '%', background: cpuColor }"
              ></div>
            </div>
          </div>
          <div class="card-sub">
            Load: {{ info.load_1min.toFixed(2) }} / {{ info.load_5min.toFixed(2) }} / {{ info.load_15min.toFixed(2) }}
          </div>
        </div>
        <div class="info-card">
          <div class="card-label">Memory</div>
          <div class="gauge-row">
            <span class="gauge-pct" :style="{ color: memColor }">{{ info.mem_pct.toFixed(1) }}%</span>
            <div class="gauge-track">
              <div
                class="gauge-fill mem-fill"
                :style="{ width: info.mem_pct + '%', background: memColor }"
              ></div>
            </div>
          </div>
          <div class="card-sub">
            {{ formatBytes(info.mem_used) }} / {{ formatBytes(info.mem_total) }}
          </div>
        </div>
        <div class="info-card">
          <div class="card-label">Disks</div>
          <div
            v-for="d in info.disks"
            :key="d.mount"
            class="disk-item"
          >
            <div class="disk-row">
              <span class="disk-mount">{{ d.mount }}</span>
              <span class="disk-pct" :style="{ color: diskItemColor(d.pct) }">{{ d.pct.toFixed(1) }}%</span>
            </div>
            <div class="gauge-track disk-track">
              <div
                class="gauge-fill disk-fill"
                :style="{ width: d.pct + '%', background: diskItemColor(d.pct) }"
              ></div>
            </div>
            <div class="disk-usage">
              {{ formatDisk(d.used) }} / {{ formatDisk(d.total) }}
            </div>
          </div>
        </div>
        <div class="status-footer">Updated {{ lastUpdate }}</div>
      </template>

      <div v-else class="status-loading">Loading...</div>
    </template>

    <!-- 收起模式：紧凑指标 -->
    <template v-else>
      <div class="compact-header" @click="$emit('toggle')">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
          <path d="M22 12c0 5.523-4.477 10-10 10S2 17.523 2 12 6.477 2 12 2s10 4.477 10 10zm-2 0a8 8 0 10-16 0 8 8 0 0016 0zm-8-6v6l4 2.5"/>
        </svg>
      </div>
      <div v-if="info" class="compact-metrics">
        <div class="compact-metric" :style="{ color: cpuColor }" title="CPU">
          <span class="compact-val">{{ info.cpu_pct.toFixed(0) }}%</span>
          <span class="compact-label">CPU</span>
        </div>
        <div class="compact-metric" :style="{ color: memColor }" title="Memory">
          <span class="compact-val">{{ info.mem_pct.toFixed(0) }}%</span>
          <span class="compact-label">MEM</span>
        </div>
        <div class="compact-metric" :style="{ color: rootDiskColor }" title="Disk">
          <span class="compact-val">{{ rootDiskPct }}%</span>
          <span class="compact-label">DISK</span>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  active: { type: Boolean, default: false },
  collapsed: { type: Boolean, default: false },
})

const info = ref(null)
const error = ref('')
const polling = ref(false)
const lastUpdate = ref('')
let timer = null

// CPU 颜色：<50% 绿, <80% 橙, >=80% 红
const cpuColor = computed(() => {
  if (!info.value) return getVar('--color-success')
  const pct = info.value.cpu_pct
  if (pct >= 80) return getVar('--color-danger')
  if (pct >= 50) return getVar('--color-warning')
  return getVar('--color-success')
})

const memColor = computed(() => {
  if (!info.value) return getVar('--color-success')
  const pct = info.value.mem_pct
  if (pct >= 80) return getVar('--color-danger')
  if (pct >= 60) return getVar('--color-warning')
  return getVar('--color-success')
})

function diskItemColor(pct) {
  if (pct >= 90) return getVar('--color-danger')
  if (pct >= 75) return getVar('--color-warning')
  return getVar('--color-success')
}

// 收起模式用根分区
const rootDisk = computed(() =>
  info.value?.disks?.find(d => d.mount === '/') || info.value?.disks?.[0]
)
const rootDiskPct = computed(() =>
  rootDisk.value ? rootDisk.value.pct.toFixed(0) : '--'
)
const rootDiskColor = computed(() =>
  rootDisk.value ? diskItemColor(rootDisk.value.pct) : getVar('--color-success')
)

function getVar(name) {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim()
}

async function fetchInfo() {
  if (!props.active) return
  polling.value = true
  error.value = ''
  try {
    info.value = await invoke('sys_info')
    lastUpdate.value = new Date().toLocaleTimeString()
  } catch (e) {
    error.value = e
  } finally {
    polling.value = false
  }
}

function formatBytes(mb) {
  if (!mb || mb === 0) return '0 MB'
  if (mb >= 1024) return (mb / 1024).toFixed(1) + ' GB'
  return mb + ' MB'
}

function formatDisk(mb) {
  if (!mb || mb === 0) return '0 GB'
  const gb = mb / 1024
  if (gb >= 1000) return (gb / 1024).toFixed(1) + ' TB'
  if (gb >= 1) return gb.toFixed(1) + ' GB'
  return mb + ' MB'
}

watch(() => props.active, (val) => {
  if (val) {
    fetchInfo()
    timer = setInterval(fetchInfo, 5000)
  } else {
    if (timer) { clearInterval(timer); timer = null }
    info.value = null
    error.value = ''
  }
}, { immediate: true })

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<style scoped>
.status-panel {
  width: 260px;
  min-width: 260px;
  height: 100%;
  background: var(--color-bg-panel);
  border-left: 1px solid var(--color-border-primary);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  padding: 0;
  user-select: none;
  transition: width 0.2s ease, min-width 0.2s ease;
}

.status-panel.collapsed {
  width: 48px;
  min-width: 48px;
  overflow: hidden;
}

/* ---- header ---- */
.status-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--color-border-secondary);
}

.poll-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-success);
  margin-left: auto;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.status-error {
  padding: 12px;
  color: var(--color-danger);
  font-size: 12px;
}

.status-loading {
  padding: 24px 12px;
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: 12px;
}

/* 信息卡片 */
.info-card {
  padding: 10px 12px;
  border-bottom: 1px solid var(--color-border-secondary);
}

.card-label {
  font-size: 10px;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}

.card-value {
  font-size: 13px;
  color: var(--color-text-primary);
  font-weight: 500;
}

.host-value {
  font-size: 15px;
  color: var(--color-accent);
}

.sys-value {
  color: var(--color-success-light);
}

.card-sub {
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-top: 2px;
}

.grey { color: var(--color-text-secondary); }

/* 仪表盘 */
.gauge-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.gauge-pct {
  font-size: 16px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  min-width: 48px;
}

.gauge-track {
  flex: 1;
  height: 6px;
  background: var(--color-border-secondary);
  border-radius: 3px;
  overflow: hidden;
}

.gauge-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.5s ease;
}

.cpu-fill { background: var(--color-success); }
.mem-fill { background: var(--color-success); }
.disk-fill { background: var(--color-success); }

/* 磁盘列表项 */
.disk-item {
  padding: 6px 0;
}

.disk-item + .disk-item {
  margin-top: 2px;
  padding-top: 8px;
  border-top: 1px solid var(--color-border-secondary);
}

.disk-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}

.disk-mount {
  font-size: 12px;
  color: var(--color-text-primary);
  font-weight: 500;
  font-family: monospace;
}

.disk-pct {
  font-size: 13px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.disk-usage {
  font-size: 10px;
  color: var(--color-text-secondary);
  margin-top: 3px;
}

.disk-track {
  height: 4px;
}

/* 底部 */
.status-footer {
  padding: 8px 12px;
  margin-top: auto;
  font-size: 10px;
  color: var(--color-text-tertiary);
  border-top: 1px solid var(--color-border-secondary);
}

/* ===== 收起模式 ===== */
.compact-header {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 12px 0;
  color: var(--color-text-secondary);
  cursor: pointer;
  border-bottom: 1px solid var(--color-border-secondary);
  transition: color 0.15s;
}

.compact-header:hover {
  color: var(--color-accent);
}

.compact-metrics {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 20px;
  padding: 16px 0;
}

.compact-metric {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.compact-val {
  font-size: 16px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.compact-label {
  font-size: 10px;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
</style>
