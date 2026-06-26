<template>
  <div class="hosts-panel">
    <!-- Toolbar -->
    <div class="hp-toolbar">
      <input v-model="search" class="hp-search" :placeholder="$t('hostsPanel.filterPlaceholder')" @input="onSearch" />
      <button class="hp-btn" @click="$emit('new-group')">{{ $t('hostsPanel.newGroup') }}</button>
      <button class="hp-btn" @click="$emit('new-host')">{{ $t('hostsPanel.newHost') }}</button>
    </div>

    <!-- Breadcrumb -->
    <div class="hp-breadcrumb">
      <span class="hp-bc-item" :class="{ active: currentGroup === 0 }" @click="navigateGroup(0)">{{ $t('hostsPanel.allHosts') }}</span>
      <template v-for="g in breadcrumb" :key="g.id">
        <span class="hp-bc-arrow">›</span>
        <span class="hp-bc-item" :class="{ active: g.id === currentGroup }" @click="navigateGroup(g.id)">{{ g.name }}</span>
      </template>
    </div>

    <div class="hp-scroll">
      <!-- Groups -->
      <div class="hp-section" v-if="groups.length > 0">
        <div class="hp-section-title">{{ $t('hostsPanel.groups') }}</div>
        <div class="hp-cards">
          <div
            v-for="g in groups"
            :key="g.id"
            class="hp-card"
            @dblclick="navigateGroup(g.id)"
            @contextmenu.prevent="$emit('ctx-group', g.id, $event)"
          >
            <div class="hp-card-icon">📁</div>
            <div class="hp-card-name">{{ g.name }}</div>
            <div class="hp-card-note" v-if="g.remark">{{ g.remark }}</div>
          </div>
        </div>
      </div>

      <!-- Hosts -->
      <div class="hp-section">
        <div class="hp-section-title">{{ $t('hostsPanel.hosts') }}</div>
        <div class="hp-cards" v-if="hosts.length > 0">
          <div
            v-for="h in hosts"
            :key="h.id"
            class="hp-card"
            @dblclick="$emit('open-host', h)"
            @contextmenu.prevent="$emit('ctx-host', h.id, $event)"
          >
            <div class="hp-card-icon">🖥️</div>
            <div class="hp-card-name">{{ h.name || h.host }}</div>
            <div class="hp-card-note">{{ h.username }}@{{ h.host }}:{{ h.port }}</div>
            <div class="hp-card-tags" v-if="hostTagsMap[h.id]?.length">
              <span v-for="t in hostTagsMap[h.id]" :key="t.id" class="hp-tag" :style="{background:t.color}">{{ t.name }}</span>
            </div>
          </div>
        </div>
        <div v-else class="hp-empty">{{ $t('hostsPanel.noHosts') }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Group, Connection, Tag } from '../types'

const props = defineProps<{
  groups: Group[]
  connections: Connection[]
  hostTagsMap: Record<number, Tag[]>
  currentGroup: number
  breadcrumb: Group[]
}>()

const emit = defineEmits<{
  'open-host': [c: Connection]
  'navigate': [groupId: number]
  'new-group': []
  'new-host': []
  'ctx-group': [id: number, e: MouseEvent]
  'ctx-host': [id: number, e: MouseEvent]
}>()

const search = ref('')

const filteredGroups = computed(() => {
  if (!search.value) return props.groups
  const q = search.value.toLowerCase()
  return props.groups.filter(g => g.name.toLowerCase().includes(q))
})

const hosts = computed(() => {
  if (!search.value) return props.connections
  const q = search.value.toLowerCase()
  return props.connections.filter(c =>
    (c.name || '').toLowerCase().includes(q) ||
    c.host.toLowerCase().includes(q) ||
    c.username.toLowerCase().includes(q)
  )
})

const groups = computed(() => filteredGroups.value)

function navigateGroup(id: number) { emit('navigate', id) }
function onSearch() {}
</script>

<style scoped>
.hosts-panel { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.hp-toolbar { display: flex; align-items: center; gap: 6px; padding: 8px 12px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; }
.hp-search { flex: 1; padding: 5px 10px; font-size: 12px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-primary); outline: none; }
.hp-search:focus { border-color: var(--color-accent); }
.hp-btn { padding: 5px 12px; font-size: 11px; background: transparent; border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; white-space: nowrap; }
.hp-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }
.hp-breadcrumb { display: flex; align-items: center; gap: 4px; padding: 6px 12px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; font-size: 11px; }
.hp-bc-item { color: var(--color-text-tertiary); cursor: pointer; padding: 2px 6px; border-radius: 3px; }
.hp-bc-item:hover { color: var(--color-accent); background: var(--color-bg-hover); }
.hp-bc-item.active { color: var(--color-text-primary); font-weight: 500; }
.hp-bc-arrow { color: var(--color-text-tertiary); }
.hp-scroll { flex: 1; overflow-y: auto; padding: 8px 12px; }
.hp-section { margin-bottom: 16px; }
.hp-section-title { font-size: 11px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 6px; }
.hp-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 8px; }
.hp-card { padding: 10px 12px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 6px; cursor: pointer; transition: all 0.12s; min-width: 0; }
.hp-card:hover { border-color: var(--color-accent); transform: translateY(-1px); }
.hp-card-icon { font-size: 18px; margin-bottom: 4px; }
.hp-card-name { font-size: 12px; font-weight: 500; color: var(--color-text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.hp-card-note { font-size: 10px; color: var(--color-text-tertiary); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.hp-card-tags { display: flex; gap: 3px; margin-top: 4px; flex-wrap: wrap; }
.hp-tag { padding: 1px 6px; border-radius: 8px; font-size: 9px; color: #fff; white-space: nowrap; }
.hp-empty { padding: 20px; text-align: center; color: var(--color-text-tertiary); font-size: 12px; }
</style>
