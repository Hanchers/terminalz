<template>
  <div class="ft-overlay" @click.self="$emit('close')">
    <div class="ft-panel">
      <!-- 标题栏 -->
      <div class="ft-titlebar">
        <span class="ft-title">{{ $t('sftp.title') }}</span>
        <button class="ft-close-btn" @click="$emit('close')">&times;</button>
      </div>

      <div class="ft-body">
        <!-- ===== 左侧：本地文件系统 ===== -->
        <div class="ft-column">
          <div class="ft-col-header">
            <span class="ft-col-title">{{ $t('sftp.local') }}</span>
            <div class="ft-col-actions">
              <button class="ft-action-btn" @click="showHiddenLocal = !showHiddenLocal" :title="showHiddenLocal ? $t('sftp.hideDotfiles') : $t('sftp.showDotfiles')">{{ showHiddenLocal ? '👁' : '👁‍🗨' }}</button>
              <button class="ft-action-btn" @click="refreshLocal" :title="$t('sftp.refresh')">↻</button>
              <!-- Actions 下拉 -->
              <div class="ft-dropdown" ref="localDropdownEl">
                <button class="ft-action-btn" @click="toggleDropdown('local')">{{ $t('sftp.actions') }} ▾</button>
                <div v-if="openDropdown === 'local'" class="ft-dropdown-menu">
                  <div class="ft-dropdown-item" @click="promptNewDir('local')">{{ $t('sftp.newFolder') }}</div>
                </div>
              </div>
            </div>
          </div>
          <!-- 路径面包屑 -->
          <div class="ft-path-bar">
            <button class="ft-path-up" @click="navUp('local')" title="Up">↑</button>
            <input
              class="ft-path-input"
              v-model="localPath"
              @keyup.enter="loadLocal(localPath)"
              spellcheck="false"
            />
          </div>
          <!-- 快捷目录 -->
          <div class="ft-quick-dirs">
            <button class="ft-quick-btn" @click="jumpToDir('home')">{{ $t('sftp.home') }}</button>
            <button class="ft-quick-btn" @click="jumpToDir('desktop')">{{ $t('sftp.desktop') }}</button>
            <button class="ft-quick-btn" @click="jumpToDir('download')">{{ $t('sftp.downloads') }}</button>
            <button class="ft-quick-btn" @click="jumpToDir('document')">{{ $t('sftp.documents') }}</button>
          </div>
          <!-- 文件列表 -->
          <div
            class="ft-file-list"
            @click="closeDropdown"
            @contextmenu.prevent="onContextMenu($event, 'local')"
            @dragover.prevent
            @drop.prevent="onDrop('local', $event)"
          >
            <div class="ft-list-header">
              <span class="ft-col-name">{{ $t('sftp.name') }}</span>
              <span class="ft-col-size">{{ $t('sftp.size') }}</span>
            </div>
            <div v-if="filteredLocalFiles.length === 0" class="ft-empty">{{ $t('sftp.empty') }}</div>
            <div
              v-for="f in filteredLocalFiles"
              :key="f.name"
              class="ft-row"
              :class="{ selected: isSelected('local', f.name) }"
              :draggable="!f.is_dir"
              @click="onClick('local', f)"
              @dblclick="onDblClick('local', f)"
              @contextmenu.stop.prevent="onRowContext($event, 'local', f)"
              @dragstart="onDragStart($event, 'local', f.name)"
            >
              <span class="ft-row-name">
                <span class="ft-icon">{{ f.is_dir ? '📁' : '📄' }}</span>
                {{ f.name }}
              </span>
              <span class="ft-row-size">{{ f.is_dir ? '--' : formatSize(f.size) }}</span>
            </div>
          </div>
        </div>

        <!-- ===== 右侧：远程文件系统 ===== -->
        <div class="ft-column">
          <div class="ft-col-header">
            <span class="ft-col-title">{{ $t('sftp.remote') }}</span>
            <div class="ft-col-actions">
              <button class="ft-action-btn" @click="refreshRemote" :title="$t('sftp.refresh')">↻</button>
              <div class="ft-dropdown" ref="remoteDropdownEl">
                <button class="ft-action-btn" @click="toggleDropdown('remote')">{{ $t('sftp.actions') }} ▾</button>
                <div v-if="openDropdown === 'remote'" class="ft-dropdown-menu">
                  <div class="ft-dropdown-item" @click="promptNewDir('remote')">{{ $t('sftp.newFolder') }}</div>
                </div>
              </div>
            </div>
          </div>
          <!-- 路径面包屑 -->
          <div class="ft-path-bar">
            <button class="ft-path-up" @click="navUp('remote')" :title="$t('sftp.up')">↑</button>
            <input
              class="ft-path-input"
              v-model="remotePath"
              @keyup.enter="loadRemote(remotePath)"
              spellcheck="false"
            />
          </div>
          <!-- 文件列表 -->
          <div
            class="ft-file-list"
            @click="closeDropdown"
            @contextmenu.prevent="onContextMenu($event, 'remote')"
            @dragover.prevent
            @drop.prevent="onDrop('remote', $event)"
          >
            <div class="ft-list-header">
              <span class="ft-col-name">{{ $t('sftp.name') }}</span>
              <span class="ft-col-size">{{ $t('sftp.size') }}</span>
            </div>
            <div v-if="remoteFiles.length === 0" class="ft-empty">{{ $t('sftp.empty') }}</div>
            <div
              v-for="f in remoteFiles"
              :key="f.name"
              class="ft-row"
              :class="{ selected: isSelected('remote', f.name) }"
              :draggable="!f.is_dir"
              @click="onClick('remote', f)"
              @dblclick="onDblClick('remote', f)"
              @contextmenu.stop.prevent="onRowContext($event, 'remote', f)"
              @dragstart="onDragStart($event, 'remote', f.name)"
            >
              <span class="ft-row-name">
                <span class="ft-icon">{{ f.is_dir ? '📁' : '📄' }}</span>
                {{ f.name }}
              </span>
              <span class="ft-row-size">{{ f.is_dir ? '--' : formatSize(f.size) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 上下文菜单 -->
      <div
        v-if="ctxMenu.visible"
        class="ft-context-menu"
        :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
      >
        <template v-if="ctxMenu.side === 'local'">
          <div class="ctx-item" @click="uploadSelected">{{ $t('sftp.uploadToRemote') }}</div>
        </template>
        <template v-else>
          <div class="ctx-item" @click="downloadSelected">{{ $t('sftp.downloadToLocal') }}</div>
          <div class="ctx-sep"></div>
          <div class="ctx-item" @click="promptRename">{{ $t('sftp.rename') }}</div>
          <div class="ctx-item ctx-danger" @click="deleteRemote">{{ $t('sftp.delete') }}</div>
          <div class="ctx-sep"></div>
          <div class="ctx-item" @click="promptNewDir('remote')">{{ $t('sftp.newFolder') }}</div>
          <div class="ctx-item" @click="refreshRemote">{{ $t('sftp.refreshList') }}</div>
        </template>
      </div>

      <!-- 多任务进度条 -->
      <div v-if="activeCount > 0" class="ft-progress-area">
        <div v-for="t in transferList" :key="t.file_name" class="ft-progress-row">
          <span class="ft-progress-name">{{ t.displayName }}</span>
          <span class="ft-progress-pct">{{ t.pct }}%</span>
          <div class="ft-progress-inner">
            <div class="ft-progress-fill" :style="{ width: t.pct + '%' }"></div>
          </div>
        </div>
      </div>

      <!-- 状态栏 -->
      <div class="ft-statusbar">{{ statusMsg }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { homeDir, desktopDir, downloadDir, documentDir } from '@tauri-apps/api/path'

const { t } = useI18n({ useScope: 'global' })

interface FileItem { name: string; is_dir: boolean; size: number; modified: string }
interface ProgressPayload { file_name: string; current: number; total: number; percentage: number; status: string }
interface CtxMenu { visible: boolean; x: number; y: number; side: string; file: FileItem | null }
interface SelectState { local: string | null; remote: string | null }

const emit = defineEmits<{ close: [] }>()

const localPath = ref('')
const remotePath = ref('/')
const localFiles = ref<FileItem[]>([])
const remoteFiles = ref<FileItem[]>([])
const showHiddenLocal = ref(false)

// Filter: hide dotfiles by default, toggle to reveal them
const filteredLocalFiles = computed(() => {
  if (showHiddenLocal.value) return localFiles.value
  return localFiles.value.filter(f => !f.name.startsWith('.'))
})

const statusMsg = ref('')
const activeTransfers = reactive<Map<string, ProgressPayload>>(new Map())
const openDropdown = ref<string | null>(null)
let unlisten: UnlistenFn | null = null

const selected = reactive<SelectState>({ local: null, remote: null })
const ctxMenu = reactive<CtxMenu>({ visible: false, x: 0, y: 0, side: '', file: null })

// ---- 多任务进度聚合 ----
const activeCount = computed(() => activeTransfers.size)
const transferList = computed(() => {
  const items: { file_name: string; displayName: string; pct: number }[] = []
  for (const t of activeTransfers.values()) {
    items.push({
      file_name: t.file_name,
      displayName: t.file_name.length > 28 ? t.file_name.slice(0, 25) + '...' : t.file_name,
      pct: Math.round(t.percentage),
    })
  }
  return items
})

// ---- 初始化 ----
onMounted(async () => {
  // Resolve home directory via Tauri path API (avoids macOS sandbox permission issues)
  try { localPath.value = await homeDir() } catch (_) { localPath.value = '/' }
  loadLocal(localPath.value)
  loadRemote(remotePath.value)
  // 监听传输进度 — multi-task tracker, one entry per file.
  unlisten = await listen('sftp-progress', (e) => {
    const p: ProgressPayload = e.payload as ProgressPayload
    activeTransfers.set(p.file_name, p)
    if (p.status === 'completed' || p.status === 'error') {
      setTimeout(() => { activeTransfers.delete(p.file_name) }, 2500)
    }
  })
  // 窗口点击关闭菜单
  window.addEventListener('click', closeCtxMenu)
})

onUnmounted(() => {
  if (unlisten) unlisten()
  window.removeEventListener('click', closeCtxMenu)
})

// ---- 数据加载 ----
async function loadLocal(dir: string): Promise<void> {
  try {
    localFiles.value = await invoke<FileItem[]>('read_local_dir', { path: dir })
    localPath.value = dir
    statusMsg.value = ''
  } catch (e) {
    statusMsg.value = t('sftp.localError') + e
    localFiles.value = []
  }
}

async function loadRemote(dir: string): Promise<void> {
  try {
    remoteFiles.value = await invoke<FileItem[]>('sftp_list_dir', { remotePath: dir })
    remotePath.value = dir
    statusMsg.value = ''
  } catch (e) {
    statusMsg.value = t('sftp.remoteError') + e
    remoteFiles.value = []
  }
}

function refreshLocal(): void { loadLocal(localPath.value) }
function refreshRemote(): void { loadRemote(remotePath.value) }

function navUp(side: string): void {
  if (side === 'local') {
    const p = localPath.value.replace(/\/+$/, '').split('/').slice(0, -1).join('/') || '/'
    loadLocal(p)
  } else {
    const p = remotePath.value.replace(/\/+$/, '').split('/').slice(0, -1).join('/') || '/'
    loadRemote(p)
  }
}

// ---- 点击 / 双击 ----
function onClick(side: string, file: FileItem): void {
  (selected as any)[side] = file.name
}

function isSelected(side: string, name: string): boolean {
  return (selected as any)[side] === name
}

function onDblClick(side: string, file: FileItem): void {
  if (!file.is_dir) return
  if (side === 'local') {
    loadLocal(localPath.value.replace(/\/+$/, '') + '/' + file.name)
  } else {
    loadRemote(remotePath.value.replace(/\/+$/, '') + '/' + file.name)
  }
}

function onDragStart(e: DragEvent, side: string, name: string): void {
  e.dataTransfer!.setData('text/plain', JSON.stringify({ side, name }))
  e.dataTransfer!.effectAllowed = 'move'
}

async function onDrop(targetSide: string, e: DragEvent): Promise<void> {
  let data: { side: string; name: string } | null = null
  try { data = JSON.parse(e.dataTransfer!.getData('text/plain')) } catch (_) { return }
  if (!data || data.side === targetSide) return

  if (data.side === 'local' && targetSide === 'remote') {
    const localFull = localPath.value.replace(/\/+$/, '') + '/' + data.name
    await uploadFile(localFull)
    refreshRemote()
  } else if (data.side === 'remote' && targetSide === 'local') {
    const remoteFull = remotePath.value.replace(/\/+$/, '') + '/' + data.name
    await downloadFile(remoteFull)
    refreshLocal()
  }
  closeCtxMenu()
}

async function uploadFile(localFull: string): Promise<void> {
  try {
    await invoke('sftp_upload', {
      localPaths: [localFull],
      remoteDir: remotePath.value,
    })
    statusMsg.value = t('sftp.uploadDone')
  } catch (e) {
    statusMsg.value = t('sftp.uploadFailed') + e
  }
}

async function downloadFile(remoteFull: string): Promise<void> {
  const name = remoteFull.split('/').pop() || 'download'
  const localFull = localPath.value.replace(/\/+$/, '') + '/' + name
  try {
    await invoke('sftp_download', { remotePath: remoteFull, localPath: localFull })
    statusMsg.value = t('sftp.downloadDone')
  } catch (e) {
    statusMsg.value = t('sftp.downloadFailed') + e
  }
}

function uploadSelected(): void {
  if (!selected.local) return
  const localFull = localPath.value.replace(/\/+$/, '') + '/' + selected.local
  uploadFile(localFull).then(() => refreshRemote())
  closeCtxMenu()
}

function downloadSelected(): void {
  if (!selected.remote) return
  const remoteFull = remotePath.value.replace(/\/+$/, '') + '/' + selected.remote
  downloadFile(remoteFull).then(() => refreshLocal())
  closeCtxMenu()
}

async function deleteRemote(): Promise<void> {
  if (!selected.remote) return
  const full = remotePath.value.replace(/\/+$/, '') + '/' + selected.remote
  if (!confirm(t('sftp.deleteConfirm', { path: full }))) return
  try {
    await invoke('sftp_delete', { remotePath: full })
    statusMsg.value = t('sftp.deleted')
    refreshRemote()
  } catch (e) { statusMsg.value = t('sftp.deleteFailed') + e }
  closeCtxMenu()
}

async function promptRename(): Promise<void> {
  if (!selected.remote) return
  const oldName = selected.remote
  const newName = prompt(t('sftp.renamePrompt'), oldName)
  if (!newName || newName === oldName) { closeCtxMenu(); return }
  const oldFull = remotePath.value.replace(/\/+$/, '') + '/' + oldName
  const newFull = remotePath.value.replace(/\/+$/, '') + '/' + newName
  try {
    await invoke('sftp_rename', { oldPath: oldFull, newPath: newFull })
    statusMsg.value = t('sftp.renamed')
    refreshRemote()
  } catch (e) { statusMsg.value = t('sftp.renameFailed') + e }
  closeCtxMenu()
}

async function promptNewDir(side: string): Promise<void> {
  closeDropdown()
  const dirName = prompt(t('sftp.folderNamePrompt'), '')
  if (!dirName) return

  if (side === 'local') {
    // 本地目录：暂无后端 API，提示用系统文件管理器
    statusMsg.value = t('sftp.mkdirNotSupported')
    closeCtxMenu()
    return
  } else {
    const full = remotePath.value.replace(/\/+$/, '') + '/' + dirName
    try {
      await invoke('sftp_mkdir', { remotePath: full })
      statusMsg.value = t('sftp.mkdirDone')
      refreshRemote()
    } catch (e) {
      statusMsg.value = t('sftp.mkdirFailed') + e
    }
  }
  closeCtxMenu()
}

// ---- 右键菜单 ----
function onContextMenu(e: MouseEvent, side: string): void {
  ctxMenu.visible = true
  ctxMenu.x = e.clientX
  ctxMenu.y = e.clientY
  ctxMenu.side = side
  ctxMenu.file = null
}

function onRowContext(e: MouseEvent, side: string, file: FileItem): void {
  (selected as any)[side] = file.name
  ctxMenu.visible = true
  ctxMenu.x = e.clientX
  ctxMenu.y = e.clientY
  ctxMenu.side = side
  ctxMenu.file = file
}

function closeCtxMenu(): void { ctxMenu.visible = false }

function toggleDropdown(name: string): void {
  openDropdown.value = openDropdown.value === name ? null : name
}

function closeDropdown(): void { openDropdown.value = null }

function formatSize(bytes: number): string {
  if (!bytes || bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1) + ' ' + units[i]
}

async function jumpToDir(dirName: string): Promise<void> {
  const dirs: Record<string, () => Promise<string>> = {
    home: homeDir,
    desktop: desktopDir,
    download: downloadDir,
    document: documentDir,
  }
  try {
    const dir = await (dirs[dirName] || homeDir)()
    loadLocal(dir)
  } catch (e) {
    statusMsg.value = t('sftp.resolveDirFailed') + e
  }
}
</script>

<style scoped>
/* 遮罩 */
.ft-overlay { position: fixed; inset: 0; z-index: 100; background: var(--shadow-overlay); display: flex; align-items: center; justify-content: center; }
.ft-panel { width: 90vw; height: 85vh; max-width: 1200px; background: var(--color-bg-panel); border: 1px solid var(--color-border-primary); border-radius: 10px; display: flex; flex-direction: column; overflow: hidden; box-shadow: var(--shadow-panel); }
.ft-titlebar { display: flex; align-items: center; justify-content: space-between; padding: 10px 16px; border-bottom: 1px solid var(--color-border-primary); flex-shrink: 0; }
.ft-title { font-size: 14px; font-weight: 600; color: var(--color-text-primary); }
.ft-close-btn { width: 30px; height: 30px; background: transparent; border: none; border-radius: 6px; color: var(--color-text-secondary); font-size: 20px; cursor: pointer; display: flex; align-items: center; justify-content: center; }
.ft-close-btn:hover { background: var(--color-bg-hover-alt); color: var(--color-text-primary); }

.ft-body { flex: 1; display: flex; overflow: hidden; }
.ft-column { flex: 1; display: flex; flex-direction: column; overflow: hidden; min-width: 0; }
.ft-column:first-child { border-right: 1px solid var(--color-border-primary); }

.ft-col-header { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; }
.ft-col-title { font-size: 12px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.5px; }
.ft-col-actions { display: flex; gap: 4px; }
.ft-action-btn { padding: 3px 8px; font-size: 11px; background: transparent; border: 1px solid var(--color-border-secondary); border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; }
.ft-action-btn:hover { background: var(--color-bg-hover); color: var(--color-text-primary); }

.ft-dropdown { position: relative; }
.ft-dropdown-menu { position: absolute; right: 0; top: 100%; z-index: 20; min-width: 140px; background: var(--color-bg-panel); border: 1px solid var(--color-border-primary); border-radius: 6px; box-shadow: var(--shadow-panel); padding: 4px; }
.ft-dropdown-item { padding: 6px 10px; font-size: 12px; color: var(--color-text-primary); border-radius: 4px; cursor: pointer; }
.ft-dropdown-item:hover { background: var(--color-bg-hover); }

.ft-path-bar { display: flex; align-items: center; gap: 4px; padding: 6px 8px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; }
.ft-quick-dirs { display: flex; gap: 4px; padding: 4px 8px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; flex-wrap: wrap; }
.ft-quick-btn { padding: 2px 8px; font-size: 11px; background: var(--color-bg-input); border: 1px solid var(--color-border-secondary); border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; white-space: nowrap; }
.ft-quick-btn:hover { background: var(--color-bg-hover); color: var(--color-text-primary); border-color: var(--color-accent); }
.ft-path-up { width: 28px; height: 28px; font-size: 14px; background: transparent; border: 1px solid var(--color-border-secondary); border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
.ft-path-up:hover { background: var(--color-bg-hover); color: var(--color-text-primary); }
.ft-path-input { flex: 1; padding: 5px 8px; font-size: 12px; font-family: monospace; background: var(--color-bg-input); border: 1px solid var(--color-border-primary); border-radius: 4px; color: var(--color-text-primary); outline: none; }
.ft-path-input:focus { border-color: var(--color-accent); }

.ft-file-list { flex: 1; overflow-y: auto; user-select: none; }
.ft-list-header { display: flex; padding: 5px 10px; font-size: 10px; color: var(--color-text-tertiary); text-transform: uppercase; letter-spacing: 0.5px; border-bottom: 1px solid var(--color-border-secondary); position: sticky; top: 0; background: var(--color-bg-panel); z-index: 1; }
.ft-col-name { flex: 1; }
.ft-col-size { width: 80px; text-align: right; flex-shrink: 0; }
.ft-empty { padding: 32px; text-align: center; color: var(--color-text-tertiary); font-size: 12px; }
.ft-row { display: flex; align-items: center; padding: 4px 10px; font-size: 12px; cursor: pointer; transition: background 0.1s; }
.ft-row:hover { background: var(--color-bg-hover); }
.ft-row.selected { background: var(--color-accent-bg); color: var(--color-text-white); }
.ft-row-name { flex: 1; display: flex; align-items: center; gap: 6px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ft-row-size { width: 80px; text-align: right; flex-shrink: 0; font-size: 11px; color: var(--color-text-secondary); }
.ft-row.selected .ft-row-size { color: var(--color-accent-light); }
.ft-icon { flex-shrink: 0; font-size: 14px; }

.ft-context-menu { position: fixed; z-index: 200; min-width: 160px; background: var(--color-bg-panel); border: 1px solid var(--color-border-primary); border-radius: 6px; box-shadow: var(--shadow-panel); padding: 4px; }
.ctx-item { padding: 6px 10px; font-size: 12px; color: var(--color-text-primary); border-radius: 4px; cursor: pointer; }
.ctx-item:hover { background: var(--color-bg-hover); }
.ctx-danger { color: var(--color-danger); }
.ctx-sep { height: 1px; background: var(--color-border-secondary); margin: 3px 6px; }

.ft-progress-area { border-top: 1px solid var(--color-border-secondary); padding: 4px 12px; flex-shrink: 0; max-height: 120px; overflow-y: auto; }
.ft-progress-row { display: flex; align-items: center; gap: 8px; padding: 3px 0; }
.ft-progress-name { font-size: 11px; color: var(--color-text-secondary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; flex: 1; }
.ft-progress-pct { font-size: 10px; color: var(--color-text-tertiary); width: 32px; text-align: right; flex-shrink: 0; }
.ft-progress-inner { flex: 2; min-width: 100px; height: 4px; background: var(--color-border-secondary); border-radius: 2px; overflow: hidden; }
.ft-progress-fill { height: 100%; background: var(--color-progress); border-radius: 2px; transition: width 0.3s; }

.ft-statusbar { padding: 4px 12px; font-size: 11px; color: var(--color-text-tertiary); border-top: 1px solid var(--color-border-secondary); flex-shrink: 0; }
</style>
