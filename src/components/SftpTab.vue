<template>
  <div class="sftp-layout">
    <div class="sf-toolbar">
      <span class="sf-title">SFTP File Manager</span>
      <span class="sf-note" v-if="!hasActive">No host connected — open a terminal tab first.</span>
      <span class="sf-note" v-else>Active host will appear here when connected.</span>
    </div>
    <div class="sf-body">
      <div class="sf-empty">
        <div class="sf-empty-icon">📂</div>
        <div class="sf-empty-title">File Transfer</div>
        <div class="sf-empty-desc">
          Upload and download files between local and remote hosts.<br/>
          This panel will be active when a terminal session is connected.
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ activeHostId: number | null }>()

const hasActive = computed(() => props.activeHostId !== null && props.activeHostId > 0)
</script>

<style scoped>
.sftp-layout { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.sf-toolbar { display: flex; align-items: center; gap: 12px; padding: 8px 12px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; }
.sf-title { font-size: 12px; font-weight: 600; color: var(--color-text-primary); }
.sf-note { font-size: 11px; color: var(--color-text-tertiary); }
.sf-body { flex: 1; display: flex; align-items: center; justify-content: center; }
.sf-empty { text-align: center; padding: 40px; }
.sf-empty-icon { font-size: 48px; margin-bottom: 12px; opacity: 0.3; }
.sf-empty-title { font-size: 16px; font-weight: 600; color: var(--color-text-secondary); margin-bottom: 8px; }
.sf-empty-desc { font-size: 12px; color: var(--color-text-tertiary); line-height: 1.6; }
</style>
