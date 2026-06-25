<template>
  <div v-if="hostDialog.visible" class="modal-overlay" @click.self="$emit('cancel')">
    <div class="modal-box">
      <div class="modal-title">{{ hostDialog.editingId ? $t('sidebar.hostDialog.edit') : $t('sidebar.hostDialog.new') }}</div>
      <div class="modal-field">
        <label>{{ $t('sidebar.hostDialog.name') }}</label>
        <input v-model="form.name" :placeholder="$t('sidebar.hostDialog.namePlaceholder')" @keyup.enter="$emit('save', form)" />
      </div>
      <div class="modal-field">
        <label>{{ $t('sidebar.hostDialog.host') }}</label>
        <input v-model="form.host" :placeholder="$t('sidebar.hostDialog.hostPlaceholder')" @keyup.enter="$emit('save', form)" />
      </div>
      <div class="modal-row">
        <div class="modal-field small">
          <label>{{ $t('sidebar.hostDialog.port') }}</label>
          <input v-model.number="form.port" type="number" placeholder="22" />
        </div>
        <div class="modal-field">
          <label>{{ $t('sidebar.hostDialog.username') }}</label>
          <input v-model="form.username" :placeholder="$t('sidebar.hostDialog.usernamePlaceholder')" @keyup.enter="$emit('save', form)" />
        </div>
      </div>
      <div class="modal-field">
        <label>{{ $t('sidebar.hostDialog.password') }}</label>
        <div class="password-wrap">
          <input
            v-model="form.password"
            :type="showHostPwd ? 'text' : 'password'"
            :placeholder="hostDialog.editingId ? $t('sidebar.hostDialog.passwordKeepPlaceholder') : $t('sidebar.hostDialog.passwordPlaceholder')"
            @keyup.enter="$emit('save', form)"
          />
          <button class="eye-btn" type="button" @click="showHostPwd = !showHostPwd" tabindex="-1">
            <svg v-if="showHostPwd" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
              <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
              <path d="M12 7c2.76 0 5 2.24 5 5 0 .65-.13 1.26-.36 1.83l2.92 2.92c1.51-1.26 2.7-2.89 3.43-4.75-1.73-4.39-6-7.5-11-7.5-1.4 0-2.74.25-3.98.7l2.16 2.16C10.74 7.13 11.35 7 12 7zM2 4.27l2.28 2.28.46.46C3.08 8.3 1.78 10.02 1 12c1.73 4.39 6 7.5 11 7.5 1.55 0 3.03-.3 4.38-.84l.42.42L19.73 22 21 20.73 3.27 3 2 4.27zM7.53 9.8l1.55 1.55c-.05.21-.08.43-.08.65 0 1.66 1.34 3 3 3 .22 0 .44-.03.65-.08l1.55 1.55c-.67.33-1.41.53-2.2.53-2.76 0-5-2.24-5-5 0-.79.2-1.53.53-2.2zm4.31-.78l3.15 3.15.02-.16c0-1.66-1.34-3-3-3l-.17.01z"/>
            </svg>
          </button>
        </div>
      </div>
      <div class="modal-field">
        <label>{{ $t('sidebar.hostDialog.remark') }}</label>
        <textarea v-model="form.remark" :placeholder="$t('sidebar.hostDialog.remarkPlaceholder')" rows="2"></textarea>
      </div>
      <div class="modal-field">
        <label>{{ $t('sidebar.hostDialog.group') }}</label>
        <select v-model="form.groupId">
          <option :value="0">{{ $t('sidebar.hostDialog.noGroup') }}</option>
          <option v-for="g in flatGroupOptions" :key="g.id" :value="g.id">{{ g.label }}</option>
        </select>
      </div>
      <div class="modal-field">
        <label>{{ $t('sidebar.hostDialog.tags') }}</label>
        <div class="tag-checkboxes">
          <label
            v-for="t in allTags"
            :key="t.id"
            class="tag-checkbox-label"
            :style="{ borderColor: t.color, color: form.tagIds.includes(t.id) ? '#fff' : t.color, background: form.tagIds.includes(t.id) ? t.color : 'transparent' }"
          >
            <input type="checkbox" :checked="form.tagIds.includes(t.id)" @change="toggleTag(t.id)" style="display:none" />
            {{ t.name }}
          </label>
          <span v-if="allTags.length === 0" class="tag-none">{{ $t('sidebar.hostDialog.noTags') }}</span>
        </div>
        <button class="btn-tag-add" @click="showQuickTag = !showQuickTag">{{ $t('sidebar.hostDialog.newTag') }}</button>
        <div v-if="showQuickTag" class="quick-tag-row">
          <input v-model="newTag.name" :placeholder="$t('sidebar.hostDialog.tagPlaceholder')" class="tag-name-input" @keyup.enter="saveQuickTag" />
          <label class="color-swatch" :style="{ background: newTag.color }">
            <input v-model="newTag.color" type="color" />
          </label>
          <button class="modal-btn primary" style="padding:4px 10px;font-size:12px" @click="saveQuickTag">{{ $t('sidebar.hostDialog.tagCreate') }}</button>
        </div>
      </div>
      <div class="modal-actions">
        <button class="modal-btn cancel" @click="$emit('cancel')">{{ $t('sidebar.hostDialog.cancel') }}</button>
        <button class="modal-btn primary" @click="$emit('save', form)">{{ $t('sidebar.hostDialog.save') }}</button>
        <button
          class="modal-btn connect"
          :disabled="connecting"
          @click="$emit('save-connect', form)"
        >
          <span v-if="connecting" class="spinner"></span>
          {{ connecting ? $t('sidebar.hostDialog.connecting') : $t('sidebar.hostDialog.saveConnect') }}
        </button>
      </div>
      <p v-if="testError" class="test-fail-row">{{ testError }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n({ useScope: 'global' })

interface Tag { id: number; name: string; color: string }
interface FlatOption { id: number; label: string; disabled?: boolean }
interface HostDialogState {
  visible: boolean; editingId: number;
  name: string; host: string; port: number; username: string; password: string;
  groupId: number; tagIds: number[]; remark: string
}

const props = defineProps<{
  hostDialog: HostDialogState
  allTags: Tag[]
  flatGroupOptions: FlatOption[]
}>()

const emit = defineEmits<{
  save: [form: HostDialogState]
  'save-connect': [form: HostDialogState]
  cancel: []
  saved: []
}>()

const showHostPwd = ref(false)
const showQuickTag = ref(false)
const connecting = ref(false)
const testError = ref('')
const newTag = reactive({ name: '', color: '#3fb950' })

// Local reactive copy — never mutate the prop directly.
const form = reactive<HostDialogState>({
  visible: false, editingId: 0,
  name: '', host: '', port: 22, username: '', password: '',
  groupId: 0, tagIds: [], remark: ''
})

// Sync prop → local form whenever dialog opens or data changes.
watch(() => props.hostDialog, (val) => {
  Object.assign(form, {
    visible: val.visible,
    editingId: val.editingId,
    name: val.name,
    host: val.host,
    port: val.port,
    username: val.username,
    password: val.password,
    groupId: val.groupId,
    tagIds: [...val.tagIds],
    remark: val.remark,
  })
}, { immediate: true, deep: true })

// Allow parent to set connecting / error state.
function setConnecting(v: boolean) { connecting.value = v }
function showError(msg: string) { testError.value = msg }
defineExpose({ setConnecting, showError })

function toggleTag(tagId: number): void {
  const idx = form.tagIds.indexOf(tagId)
  if (idx >= 0) form.tagIds.splice(idx, 1)
  else form.tagIds.push(tagId)
}

async function saveQuickTag(): Promise<void> {
  if (!newTag.name.trim()) return
  try {
    const tag = await invoke<Tag>('save_tag', { name: newTag.name.trim(), color: newTag.color })
    form.tagIds.push(tag.id)
    newTag.name = ''
    showQuickTag.value = false
    emit('saved')
  } catch (e) { alert(t('sidebar.error.failed') + e) }
}
</script>

<style scoped>
.tag-checkboxes { display: flex; flex-wrap: wrap; gap: 6px; }
.tag-checkbox-label {
  display: inline-flex; align-items: center; gap: 2px;
  padding: 2px 8px; border-radius: 12px; font-size: 11px;
  border: 1px solid; cursor: pointer; transition: all 0.15s; user-select: none;
}
.tag-checkbox-label:hover { opacity: 0.85; }
.tag-none { font-size: 11px; color: var(--color-text-tertiary); }
.btn-tag-add {
  margin-top: 6px; padding: 2px 8px; font-size: 11px;
  background: transparent; border: 1px dashed var(--color-border-input);
  border-radius: 4px; color: var(--color-text-tertiary); cursor: pointer;
}
.btn-tag-add:hover { border-color: var(--color-accent); color: var(--color-accent); }
.quick-tag-row { display: flex; gap: 6px; margin-top: 6px; align-items: center; }
.tag-name-input { flex: 1; min-width: 260px; padding: 4px 8px; font-size: 12px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-primary); }

.modal-btn.connect {
  display: flex; align-items: center; gap: 6px;
  padding: 6px 16px; font-size: 12px;
  background: var(--color-accent); color: var(--color-text-white);
  border: 1px solid var(--color-accent); border-radius: 4px;
  cursor: pointer; transition: all 0.15s;
}
.modal-btn.connect:hover:not(:disabled) { opacity: 0.9; }
.modal-btn.connect:disabled { opacity: 0.6; cursor: not-allowed; }
.spinner {
  width: 12px; height: 12px; border: 2px solid rgba(255,255,255,0.3);
  border-top-color: var(--color-text-white); border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
.test-fail-row { margin: 6px 0 0; padding: 0; font-size: 12px; color: var(--color-danger); }
</style>
