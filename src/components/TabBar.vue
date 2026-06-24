<template>
  <div class="tab-bar">
    <div class="tab-list">
      <!-- Fixed tabs -->
      <button class="tab" :class="{ active: modelValue === 'vault' }" @click="$emit('update:modelValue', 'vault')">
        <svg viewBox="0 0 24 24" width="13" height="13" fill="currentColor">
          <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zM9 6c0-1.66 1.34-3 3-3s3 1.34 3 3v2H9V6z"/>
        </svg>
        <span>Vault</span>
      </button>
      <button class="tab" :class="{ active: modelValue === 'sftp' }" @click="$emit('update:modelValue', 'sftp')">
        <svg viewBox="0 0 24 24" width="13" height="13" fill="currentColor">
          <path d="M20 6h-8l-2-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2z"/>
        </svg>
        <span>SFTP</span>
      </button>

      <!-- Separator -->
      <div class="tab-sep"></div>

      <!-- Dynamic terminal tabs -->
      <button
        v-for="t in tabs"
        :key="t.id"
        class="tab"
        :class="{ active: modelValue === t.id }"
        @click="$emit('update:modelValue', t.id)"
      >
        <span class="tab-dot" :class="t.connected ? 'dot-on' : t.connecting ? 'dot-busy' : 'dot-off'"></span>
        <span class="tab-label">{{ t.name || t.host }}</span>
        <span class="tab-close" @click.stop="$emit('close-tab', t.id)">&times;</span>
      </button>

      <!-- New tab button -->
      <button class="tab tab-add" @click="$emit('new-tab')" title="New Tab">+</button>
    </div>
  </div>
</template>

<script setup lang="ts">
interface TabInfo { id: number; name?: string; host: string; connected: boolean; connecting: boolean }

defineProps<{
  modelValue: string | number
  tabs: TabInfo[]
}>()

defineEmits<{
  'update:modelValue': [v: string | number]
  'close-tab': [id: number]
  'new-tab': []
}>()
</script>

<style scoped>
.tab-bar { height: 34px; background: var(--color-bg-secondary); border-bottom: 1px solid var(--color-border-tab); display: flex; align-items: stretch; flex-shrink: 0; padding: 0 4px; overflow: hidden; }
.tab-list { display: flex; align-items: center; gap: 2px; min-width: 0; flex: 1; overflow-x: auto; }
.tab-list::-webkit-scrollbar { height: 0; }
.tab {
  display: flex; align-items: center; gap: 5px;
  padding: 3px 10px; height: 28px;
  background: transparent; border: none; border-radius: 5px;
  color: var(--color-text-secondary); font-size: 12px;
  cursor: pointer; white-space: nowrap; flex-shrink: 0;
  transition: all 0.12s;
}
.tab:hover { background: var(--color-bg-hover); color: var(--color-text-primary); }
.tab.active { background: var(--color-bg-primary); color: var(--color-text-heading); }
.tab-sep { width: 1px; height: 16px; background: var(--color-border-secondary); margin: 0 4px; flex-shrink: 0; }
.tab-dot { width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0; }
.dot-on { background: #3fb950; }
.dot-busy { background: #d29922; animation: pulse 1s infinite; }
.dot-off { background: var(--color-text-tertiary); }
@keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.4} }
.tab-label { max-width: 120px; overflow: hidden; text-overflow: ellipsis; }
.tab-close { width: 16px; height: 16px; display: flex; align-items: center; justify-content: center; border-radius: 3px; font-size: 14px; line-height: 1; opacity: 0; transition: opacity 0.1s; }
.tab:hover .tab-close { opacity: 1; }
.tab-close:hover { background: var(--color-danger-btn); color: var(--color-text-white); }
.tab-add { padding: 3px 8px; font-size: 16px; font-weight: 300; }
.tab-add:hover { background: var(--color-bg-hover); color: var(--color-accent); }
</style>
