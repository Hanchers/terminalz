<template>
  <div class="tv-wrapper">
    <Terminal
      v-if="tab"
      :key="tab.id"
      :prefill="prefill"
      :mode="'ssh'"
      @connection-change="onChange"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import Terminal from './Terminal.vue'

interface TabInfo { id: number; name?: string; host: string; port: number; username: string; connected: boolean; connecting: boolean }

const props = defineProps<{ tab: TabInfo }>()
const emit = defineEmits<{ 'connection-change': [connected: boolean]; close: [] }>()

const prefill = computed(() => {
  const p = {
    id: props.tab.id,
    name: props.tab.name,
    host: props.tab.host,
    port: props.tab.port,
    username: props.tab.username,
  }
  console.log('[TerminalView] prefill computed:', p)
  return p
})

function onChange(connected: boolean) {
  emit('connection-change', connected)
}
</script>

<style scoped>
.tv-wrapper { width: 100%; height: 100%; display: flex; flex-direction: column; overflow: hidden; }
</style>
