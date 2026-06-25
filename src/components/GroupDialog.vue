<template>
  <div v-if="groupDialog.visible" class="modal-overlay" @click.self="$emit('cancel')">
    <div class="modal-box">
      <div class="modal-title">{{ groupDialog.editingId ? $t('sidebar.groupDialog.edit') : $t('sidebar.groupDialog.new') }}</div>
      <div class="modal-field">
        <label>{{ $t('sidebar.groupDialog.name') }}</label>
        <input v-model="groupDialog.name" placeholder="Production" @keyup.enter="$emit('save')" />
      </div>
      <div class="modal-field">
        <label>{{ $t('sidebar.groupDialog.parentGroup') }}</label>
        <select v-model="groupDialog.parentId">
          <option :value="0">{{ $t('sidebar.groupDialog.root') }}</option>
          <option v-for="g in groupSelectOptions" :key="g.id" :value="g.id" :disabled="g.disabled">{{ g.label }}</option>
        </select>
      </div>
      <div class="modal-field">
        <label>{{ $t('sidebar.groupDialog.remark') }}</label>
        <textarea v-model="groupDialog.remark" :placeholder="$t('sidebar.groupDialog.remarkPlaceholder')" rows="2"></textarea>
      </div>
      <div class="modal-actions">
        <button class="modal-btn cancel" @click="$emit('cancel')">{{ $t('sidebar.groupDialog.cancel') }}</button>
        <button class="modal-btn primary" @click="$emit('save')">{{ $t('sidebar.groupDialog.save') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface FlatOption { id: number; label: string; disabled?: boolean }
interface GroupDialogState {
  visible: boolean; editingId: number;
  name: string; parentId: number; remark: string
}

defineProps<{
  groupDialog: GroupDialogState
  groupSelectOptions: FlatOption[]
}>()

defineEmits<{
  save: []
  cancel: []
}>()
</script>
