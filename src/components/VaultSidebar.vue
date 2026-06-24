<template>
  <div class="vault-sidebar">
    <div class="vs-nav">
      <button class="vs-item" :class="{ active: active === 'hosts' }" @click="$emit('update:active', 'hosts')">
        <svg viewBox="0 0 24 24" width="15" height="15" fill="currentColor"><path d="M4 6h16v2H4zm0 5h16v2H4zm0 5h16v2H4z"/></svg>
        <span>Hosts</span>
      </button>
      <button class="vs-item" :class="{ active: active === 'keychain' }" @click="$emit('update:active', 'keychain')">
        <svg viewBox="0 0 24 24" width="15" height="15" fill="currentColor"><path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zM12 17c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2z"/></svg>
        <span>Keychain</span>
      </button>
      <button class="vs-item" :class="{ active: active === 'portforward' }" @click="$emit('update:active', 'portforward')">
        <svg viewBox="0 0 24 24" width="15" height="15" fill="currentColor"><path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/></svg>
        <span>Port Forward</span>
      </button>
      <button class="vs-item" :class="{ active: active === 'snippets' }" @click="$emit('update:active', 'snippets')">
        <svg viewBox="0 0 24 24" width="15" height="15" fill="currentColor"><path d="M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/></svg>
        <span>Snippets</span>
      </button>
    </div>
    <div class="vs-footer">
      <button class="vs-item" :class="{ active: active === 'settings' }" @click="$emit('update:active', 'settings')">
        <span>⚙️</span>
        <span>Settings</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ active: string }>()
defineEmits<{ 'update:active': [v: string] }>()
</script>

<style scoped>
.vault-sidebar { width: 180px; height: 100%; background: var(--color-bg-panel); border-right: 1px solid var(--color-border-primary); display: flex; flex-direction: column; overflow: hidden; flex-shrink: 0; }
.vs-nav { padding: 8px; display: flex; flex-direction: column; gap: 2px; }
.vs-item { display: flex; align-items: center; gap: 8px; padding: 7px 10px; border: none; border-radius: 5px; background: transparent; color: var(--color-text-secondary); font-size: 12px; cursor: pointer; text-align: left; transition: all 0.12s; }
.vs-item:hover { background: var(--color-bg-hover); color: var(--color-text-primary); }
.vs-item.active { background: var(--color-bg-hover); color: var(--color-accent); }
.vs-item svg { flex-shrink: 0; }
.vs-item span { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.vs-footer { margin-top: auto; padding: 8px; border-top: 1px solid var(--color-border-secondary); display: flex; flex-direction: column; gap: 6px; }
.vs-label { font-size: 10px; color: var(--color-text-tertiary); text-transform: uppercase; letter-spacing: 0.5px; }
.vs-select { width: 100%; padding: 4px 6px; font-size: 11px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-primary); outline: none; }
</style>
