<template>
  <template v-for="g in childGroups" :key="'g'+g.id">
    <div
      class="tree-row tree-group"
      :style="{ paddingLeft: (depth * 14 + 8) + 'px' }"
      @click.stop="$emit('toggle-group', g.id)"
      @contextmenu.prevent.stop="$emit('ctx-group', g.id, $event)"
    >
      <span class="tree-arrow">{{ isCollapsed(g.id) ? '▸' : '▾' }}</span>
      <span class="tree-icon">📁</span>
      <span class="tree-name">{{ g.name }}</span>
      <span v-if="g.remark" class="tree-remark">{{ g.remark }}</span>
    </div>
    <template v-if="!isCollapsed(g.id)">
      <TreeNode
        :groups="groups" :connections="connections"
        :selected-id="selectedId" :parent-id="g.id" :depth="depth + 1"
        :collapsed-groups="collapsedGroups"
        @toggle-group="(id) => $emit('toggle-group', id)"
        @select-host="(h) => $emit('select-host', h)"
        @ctx-group="(id, e) => $emit('ctx-group', id, e)"
        @ctx-host="(id, e) => $emit('ctx-host', id, e)"
      />
    </template>
  </template>
  <div
    v-for="c in directHosts" :key="'h'+c.id"
    class="tree-row tree-host"
    :class="{ selected: selectedId === c.id }"
    :style="{ paddingLeft: (depth * 14 + 8) + 'px' }"
    @click.stop="$emit('select-host', c)"
    @contextmenu.prevent.stop="$emit('ctx-host', c.id, $event)"
  >
    <span class="tree-arrow"></span>
    <span class="tree-icon">🖥</span>
    <span class="tree-name">{{ c.name || c.host }}</span>
    <span class="tree-detail">{{ c.username }}@{{ c.host }}:{{ c.port }}</span>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  groups: Array,
  connections: Array,
  selectedId: Number,
  parentId: Number,
  depth: Number,
  collapsedGroups: Object,  // Set 在 Vue props 中传过来会变成普通对象
})

defineEmits(['toggle-group', 'select-host', 'ctx-group', 'ctx-host'])

function isCollapsed(id) {
  // Set 通过 props 传递后变成了普通对象，需要特殊处理
  if (props.collapsedGroups instanceof Set) return props.collapsedGroups.has(id)
  // 兼容 Set-like 或 Array
  if (props.collapsedGroups && typeof props.collapsedGroups.has === 'function') return props.collapsedGroups.has(id)
  if (Array.isArray(props.collapsedGroups)) return props.collapsedGroups.includes(id)
  return false
}

const childGroups = computed(() =>
  (props.groups || []).filter(g => g.parent_id === props.parentId)
)

const directHosts = computed(() =>
  (props.connections || []).filter(c => (c.group_id || 0) === props.parentId)
)
</script>

<style scoped>
.tree-row { display: flex; align-items: center; gap: 4px; padding: 4px 8px; font-size: 12px; cursor: pointer; transition: background 0.1s; }
.tree-row:hover { background: var(--color-bg-hover); }
.tree-row.selected { background: var(--color-accent-bg); color: var(--color-text-white); }
.tree-arrow { width: 12px; text-align: center; font-size: 10px; color: var(--color-text-tertiary); flex-shrink: 0; }
.tree-icon { flex-shrink: 0; font-size: 13px; }
.tree-name { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--color-text-primary); }
.tree-row.selected .tree-name { color: var(--color-text-white); }
.tree-remark { font-size: 10px; color: var(--color-text-tertiary); font-style: italic; margin-left: 4px; }
.tree-detail { font-size: 10px; color: var(--color-text-tertiary); margin-left: 4px; }
.tree-row.selected .tree-detail { color: var(--color-accent-light); }
</style>
