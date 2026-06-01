<template>
  <div class="ft-overlay" @click.self="$emit('close')">
    <div class="ft-panel">
      <div class="ft-header">
        <h3>File Upload</h3>
        <button class="ft-close-btn" @click="$emit('close')">&times;</button>
      </div>

      <!-- 文件选择区 -->
      <div class="ft-select-area">
        <button class="ft-select-btn" @click="pickFiles" :disabled="uploading">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
            <path d="M9 16h6v-6h4l-7-7-7 7h4zm-4 2h14v2H5z"/>
          </svg>
          Select Files
        </button>
        <button
          class="ft-upload-btn"
          @click="startUpload"
          :disabled="selectedFiles.length === 0 || uploading"
        >
          {{ uploading ? 'Uploading...' : `Upload (${selectedFiles.length})` }}
        </button>
      </div>

      <!-- 远程路径输入 -->
      <div class="ft-remote-path">
        <input
          v-model="remoteDir"
          placeholder="Remote directory (e.g. /home/user/)"
          class="ft-path-input"
          :disabled="uploading"
        />
      </div>

      <!-- 文件列表 & 进度 -->
      <div v-if="selectedFiles.length > 0" class="ft-file-list">
        <div v-for="(file, idx) in selectedFiles" :key="idx" class="ft-file-item">
          <div class="ft-file-info">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" class="ft-file-icon">
              <path d="M6 2c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6H6zm7 7V3.5L18.5 9H13z"/>
            </svg>
            <span class="ft-file-name" :title="file.name">{{ file.name }}</span>
            <span class="ft-file-size">{{ formatSize(file.size) }}</span>
            <span v-if="file.status === 'completed'" class="ft-status done">✓</span>
            <span v-else-if="file.status === 'error'" class="ft-status err">✗</span>
            <span v-else-if="file.status === 'uploading'" class="ft-status uploading">↻</span>
          </div>
          <div class="ft-progress-track">
            <div
              class="ft-progress-fill"
              :class="{
                completed: file.status === 'completed',
                error: file.status === 'error',
              }"
              :style="{ width: file.progress + '%' }"
            ></div>
          </div>
        </div>
      </div>

      <!-- 整体进度 -->
      <div v-if="selectedFiles.length > 0" class="ft-overall">
        <div class="ft-overall-info">
          <span>Overall Progress</span>
          <span class="ft-overall-pct">{{ overallProgress }}%</span>
        </div>
        <div class="ft-progress-track overall-track">
          <div
            class="ft-progress-fill"
            :style="{ width: overallProgress + '%' }"
          ></div>
        </div>
      </div>

      <p v-if="resultMsg" class="ft-result" :class="{ error: hasError }">{{ resultMsg }}</p>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'

const emit = defineEmits(['close'])

const remoteDir = ref('/home/')
const uploading = ref(false)
const resultMsg = ref('')
const hasError = ref(false)
let unlisten = null

// 文件状态: { name, size, path, progress, status, current, total }
const selectedFiles = ref([])

const overallProgress = computed(() => {
  if (selectedFiles.value.length === 0) return 0
  const sum = selectedFiles.value.reduce((acc, f) => acc + f.progress, 0)
  return +(sum / selectedFiles.value.length).toFixed(1)
})

async function pickFiles() {
  const files = await open({
    multiple: true,
    title: 'Select files to upload',
  })

  if (!files) return
  const paths = Array.isArray(files) ? files : [files]

  for (const filePath of paths) {
    // 避免重复
    if (selectedFiles.value.some(f => f.path === filePath)) continue
    const name = filePath.split(/[/\\]/).pop() || filePath
    selectedFiles.value.push({
      name,
      path: filePath,
      size: 0,
      progress: 0,
      current: 0,
      total: 0,
      status: 'pending', // pending | uploading | completed | error
    })
  }
  resultMsg.value = ''
  hasError.value = false
}

async function startUpload() {
  if (selectedFiles.value.length === 0) return
  uploading.value = true
  resultMsg.value = ''
  hasError.value = false

  // 监听上传进度事件
  unlisten = await listen('sftp-progress', (event) => {
    const { file_name, current, total, percentage, status } = event.payload
    const file = selectedFiles.value.find(f => f.name === file_name)
    if (!file) return

    file.progress = Math.round(percentage * 10) / 10
    file.current = current
    file.total = total
    file.status = status
  })

  try {
    const paths = selectedFiles.value.map(f => f.path)
    const result = await invoke('sftp_upload', {
      localPaths: paths,
      remoteDir: remoteDir.value,
    })

    const successCount = result.success.length
    const failCount = result.failed.length
    if (failCount > 0) {
      hasError.value = true
      resultMsg.value = `Uploaded: ${successCount} succeeded, ${failCount} failed`
    } else {
      resultMsg.value = `All ${successCount} file(s) uploaded successfully!`
    }
  } catch (e) {
    hasError.value = true
    resultMsg.value = 'Upload failed: ' + e
  } finally {
    uploading.value = false
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }
}

function formatSize(bytes) {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  if (i === 0) return bytes + ' B'
  return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i]
}

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>

<style scoped>
.ft-overlay {
  position: fixed;
  inset: 0;
  background: var(--shadow-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.ft-panel {
  width: 520px;
  max-height: 80vh;
  background: var(--color-bg-panel);
  border: 1px solid var(--color-border-primary);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: var(--shadow-panel);
}

.ft-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border-secondary);
}

.ft-header h3 {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.ft-close-btn {
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--color-text-secondary);
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ft-close-btn:hover {
  background: var(--color-bg-hover-alt);
  color: var(--color-text-primary);
}

/* 操作按钮 */
.ft-select-area {
  display: flex;
  gap: 10px;
  padding: 16px 20px 8px;
}

.ft-select-btn,
.ft-upload-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 16px;
  border: 1px solid var(--color-border-primary);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.ft-select-btn {
  background: var(--color-bg-hover-alt);
  color: var(--color-text-primary);
}

.ft-select-btn:hover:not(:disabled) {
  background: var(--color-bg-active);
}

.ft-upload-btn {
  background: var(--color-btn-upload);
  color: var(--color-text-white);
  border-color: var(--color-btn-upload-hover);
}

.ft-upload-btn:hover:not(:disabled) {
  background: var(--color-btn-upload-hover);
}

.ft-upload-btn:disabled,
.ft-select-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 远程路径 */
.ft-remote-path {
  padding: 8px 20px;
}

.ft-path-input {
  width: 100%;
  padding: 8px 12px;
  background: var(--color-bg-input);
  border: 1px solid var(--color-border-primary);
  border-radius: 6px;
  color: var(--color-text-primary);
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
}

.ft-path-input:focus {
  border-color: var(--color-accent);
}

/* 文件列表 */
.ft-file-list {
  padding: 8px 20px;
  overflow-y: auto;
  max-height: 300px;
}

.ft-file-item {
  padding: 8px 0;
  border-bottom: 1px solid var(--color-border-secondary);
}

.ft-file-item:last-child {
  border-bottom: none;
}

.ft-file-info {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.ft-file-icon {
  color: var(--color-accent);
  flex-shrink: 0;
}

.ft-file-name {
  flex: 1;
  font-size: 13px;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ft-file-size {
  font-size: 11px;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.ft-status {
  font-size: 14px;
  font-weight: bold;
  flex-shrink: 0;
  width: 16px;
  text-align: center;
}

.ft-status.done { color: var(--color-success); }
.ft-status.err { color: var(--color-danger); }
.ft-status.uploading { color: var(--color-warning); animation: spin 1s linear infinite; }

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* 进度条 */
.ft-progress-track {
  width: 100%;
  height: 6px;
  background: var(--color-border-secondary);
  border-radius: 3px;
  overflow: hidden;
}

.ft-progress-fill {
  height: 100%;
  background: var(--color-progress);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.ft-progress-fill.completed {
  background: var(--color-progress-done);
}

.ft-progress-fill.error {
  background: var(--color-progress-error);
}

/* 整体进度 */
.ft-overall {
  padding: 12px 20px 16px;
  border-top: 1px solid var(--color-border-primary);
}

.ft-overall-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 6px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.ft-overall-pct {
  font-weight: 600;
  color: var(--color-accent);
}

.overall-track {
  height: 8px;
}

/* 结果消息 */
.ft-result {
  padding: 10px 20px 16px;
  font-size: 13px;
  color: var(--color-success);
  border-top: 1px solid var(--color-border-secondary);
  margin: 0;
}

.ft-result.error {
  color: var(--color-danger);
}
</style>
