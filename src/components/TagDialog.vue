<template>
  <div v-if="tagDialog.visible" class="modal-overlay" @click.self="$emit('cancel')">
    <div class="modal-box" style="max-width:360px">
      <div class="modal-title">{{ $t('sidebar.tagDialog.title') }}</div>
      <div v-if="allTags.length === 0" class="tag-none" style="padding:8px 0">{{ $t('sidebar.tagDialog.none') }}</div>
      <div v-for="t in allTags" :key="t.id" class="tag-mgr-row">
        <template v-if="editingTag && editingTag.id === t.id">
          <label class="color-swatch" style="width:18px;height:18px" :style="{ background: editingTag.color }">
            <input v-model="editingTag.color" type="color" />
          </label>
          <input v-model="editingTag.name" class="tag-edit-input" @keyup.enter="saveEditTag" @keyup.escape="cancelEditTag" />
          <button class="tag-mgr-save" @click="saveEditTag" title="Save">✓</button>
          <button class="tag-mgr-del" @click="cancelEditTag" title="Cancel">×</button>
        </template>
        <template v-else>
          <span class="tag-mgr-swatch" :style="{ background: t.color }"></span>
          <span class="tag-mgr-name tag-clickable" @click="startEditTag(t)">{{ t.name }}</span>
          <button class="tag-mgr-del" @click="doDeleteTag(t.id)" title="Delete tag">×</button>
        </template>
      </div>
      <div class="modal-field" style="margin-top:12px">
        <label>{{ $t('sidebar.tagDialog.name') }}</label>
        <div style="display:flex;gap:8px">
          <input v-model="tagDialog.name" :placeholder="$t('sidebar.hostDialog.tagPlaceholder')" style="flex:1" @keyup.enter="doSaveTag" />
          <label class="color-swatch" :style="{ background: tagDialog.color }">
            <input v-model="tagDialog.color" type="color" />
          </label>
          <button class="modal-btn primary" style="padding:4px 12px;font-size:12px" @click="doSaveTag">{{ $t('sidebar.tagDialog.create') }}</button>
        </div>
      </div>
      <div class="modal-actions">
        <button class="modal-btn cancel" @click="$emit('cancel')">{{ $t('sidebar.tagDialog.done') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n({ useScope: 'global' })

interface Tag { id: number; name: string; color: string }
interface TagDialogState { visible: boolean; name: string; color: string }

const props = defineProps<{
  tagDialog: TagDialogState
  allTags: Tag[]
}>()

const emit = defineEmits<{
  cancel: []
  saved: []
}>()

const editingTag = ref<{ id: number; name: string; color: string } | null>(null)

function startEditTag(tag: Tag): void {
  editingTag.value = { id: tag.id, name: tag.name, color: tag.color }
}

function cancelEditTag(): void { editingTag.value = null }

async function saveEditTag(): Promise<void> {
  if (!editingTag.value || !editingTag.value.name.trim()) return
  try {
    await invoke('update_tag', {
      id: editingTag.value.id,
      name: editingTag.value.name.trim(),
      color: editingTag.value.color,
    })
    editingTag.value = null
    emit('saved')
  } catch (e) { alert(String(e)) }
}

async function doSaveTag(): Promise<void> {
  if (!props.tagDialog.name.trim()) return
  try {
    await invoke('save_tag', { name: props.tagDialog.name.trim(), color: props.tagDialog.color })
    props.tagDialog.name = ''
    emit('saved')
  } catch (e) { alert(t('sidebar.error.failed') + e) }
}

async function doDeleteTag(id: number): Promise<void> {
  if (!confirm(t('sidebar.tagDialog.deleteConfirm'))) return
  try { await invoke('delete_tag', { id }); emit('saved') } catch (e) { alert(String(e)) }
}
</script>

<style scoped>
.tag-none { font-size: 11px; color: var(--color-text-tertiary); }
.tag-mgr-row {
  display: flex; align-items: center; gap: 8px; padding: 6px 0;
  border-bottom: 1px solid var(--color-border-secondary);
}
.tag-mgr-swatch { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
.tag-mgr-name { flex: 1; font-size: 13px; color: var(--color-text-primary); }
.tag-mgr-del {
  width: 22px; height: 22px; display: flex; align-items: center; justify-content: center;
  background: transparent; border: none; border-radius: 4px; color: var(--color-text-tertiary);
  cursor: pointer; font-size: 16px;
}
.tag-mgr-del:hover { background: var(--color-danger-btn); color: var(--color-text-white); }
.tag-mgr-save {
  width: 22px; height: 22px; display: flex; align-items: center; justify-content: center;
  background: transparent; border: none; border-radius: 4px; color: var(--color-success);
  cursor: pointer; font-size: 14px;
}
.tag-mgr-save:hover { background: var(--color-success); color: var(--color-text-white); }
.tag-clickable { cursor: pointer; border-radius: 4px; padding: 2px 4px; margin: -2px -4px; }
.tag-clickable:hover { background: var(--color-bg-hover); }
.tag-edit-input {
  flex: 1; padding: 3px 6px; font-size: 12px;
  background: var(--color-bg-input); border: 1px solid var(--color-accent);
  border-radius: 4px; color: var(--color-text-primary); outline: none;
}
</style>
