<template>
  <div class="terminal-wrapper">
    <!-- 连接表单覆盖层 -->
    <div v-if="!connected" class="connect-overlay">
      <div class="overlay-content">
        <!-- 左侧：表单 -->
        <div class="connect-form">
          <h2>SSH 远程连接</h2>
          <div class="form-group">
            <input v-model="name" placeholder="连接名称 (可选)" @keyup.enter="doConnect" />
          </div>
          <div class="form-group">
            <input v-model="host" placeholder="主机地址 (例: 192.168.1.1)" @keyup.enter="doConnect" />
            <input
              v-model.number="port"
              placeholder="端口"
              type="number"
              style="max-width: 100px"
              @keyup.enter="doConnect"
            />
          </div>
          <div class="form-group">
            <input v-model="username" placeholder="用户名" @keyup.enter="doConnect" />
            <input
              v-model="password"
              placeholder="密码"
              type="password"
              @keyup.enter="doConnect"
            />
          </div>
          <div class="btn-row">
            <button class="btn-connect" @click="doConnect" :disabled="connecting">
              {{ connecting ? '连接中...' : '连接' }}
            </button>
            <button class="btn-save" @click="doSave" :disabled="!canSave">💾 保存</button>
          </div>
          <p v-if="error" class="error">{{ error }}</p>
        </div>

        <!-- 右侧：已保存的连接 -->
        <div v-if="savedList.length > 0" class="saved-list">
          <h3>📋 已保存的连接</h3>
          <div
            v-for="item in savedList"
            :key="item.id"
            class="saved-item"
            @click="loadConfig(item)"
          >
            <div class="saved-info">
              <span class="saved-name">{{ item.name || item.host }}</span>
              <span class="saved-detail">{{ item.username }}@{{ item.host }}:{{ item.port }}</span>
            </div>
            <button class="btn-del" @click.stop="doDelete(item.id)">✕</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 终端 -->
    <div ref="termContainer" class="term-container"></div>

    <!-- 工具栏 -->
    <div v-if="connected" class="toolbar">
      <button @click="showFileTransfer = true" class="upload-btn" title="Upload Files">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <path d="M9 16h6v-6h4l-7-7-7 7h4zm-4 2h14v2H5z"/>
        </svg>
        Upload
      </button>
      <button @click="disconnect" class="disconnect-btn">
        断开连接
      </button>
    </div>

    <!-- 文件传输面板 -->
    <FileTransfer v-if="showFileTransfer" @close="showFileTransfer = false" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import '@xterm/xterm/css/xterm.css';
import FileTransfer from './FileTransfer.vue';

const connected = ref(false);
const connecting = ref(false);
const error = ref('');
const name = ref('');
const host = ref('');
const port = ref(22);
const username = ref('');
const password = ref('');
const termContainer = ref(null);
const savedList = ref([]);
const editingId = ref(0);
const showFileTransfer = ref(false);

let term = null;
let fitAddon = null;
let unlisten = null;

const canSave = computed(() => host.value && username.value);

// ---- 初始化 ----

onMounted(async () => {
  await loadSavedList();
});

// ---- 终端操作 ----

function createTerminal() {
  term = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: {
      background: '#1e1e1e',
      foreground: '#d4d4d4',
      cursor: '#ffffff',
    },
    allowProposedApi: true,
  });

  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
}

function ensureTerminalOpen() {
  if (!term) createTerminal();
  term.open(termContainer.value);
  fitAddon.fit();
}

async function doConnect() {
  if (!host.value || !username.value || !password.value) {
    error.value = '请填写主机地址、用户名和密码';
    return;
  }

  connecting.value = true;
  error.value = '';

  try {
    ensureTerminalOpen();

    unlisten = await listen('ssh-output', (event) => {
      if (term) {
        const data = new Uint8Array(event.payload.data);
        term.write(data);
      }
    });

    term.onData((data) => {
      invoke('ssh_write', {
        data: Array.from(new TextEncoder().encode(data)),
      }).catch(() => {});
    });

    term.onResize(({ rows, cols }) => {
      if (connected.value) {
        invoke('ssh_resize', { rows, cols }).catch(() => {});
      }
    });

    await invoke('ssh_connect', {
      host: host.value,
      port: port.value,
      username: username.value,
      password: password.value,
      rows: term.rows,
      cols: term.cols,
    });

    connected.value = true;

    window.addEventListener('resize', () => fitAddon?.fit());
  } catch (e) {
    error.value = `连接失败: ${e}`;
    cleanupTerminal();
  } finally {
    connecting.value = false;
  }
}

function cleanupTerminal() {
  if (unlisten) { unlisten(); unlisten = null; }
  if (term) { term.dispose(); term = null; fitAddon = null; }
}

async function disconnect() {
  try { await invoke('ssh_disconnect'); } catch (_) {}
  cleanupTerminal();
  connected.value = false;
}

// ---- 数据库操作 ----

async function loadSavedList() {
  try {
    savedList.value = await invoke('list_connections');
  } catch (_) {
    savedList.value = [];
  }
}

function loadConfig(item) {
  name.value = item.name || '';
  host.value = item.host;
  port.value = item.port;
  username.value = item.username;
  password.value = item.password;
  editingId.value = item.id;
  error.value = '';
}

async function doSave() {
  if (!canSave.value) return;
  try {
    const config = {
      id: editingId.value || 0,
      name: name.value || `${username.value}@${host.value}`,
      host: host.value,
      port: port.value,
      username: username.value,
      password: password.value,
    };
    const saved = await invoke('save_connection', { config });
    editingId.value = saved.id;
    error.value = '已保存 ✓';
    setTimeout(() => { error.value = ''; }, 1500);
    await loadSavedList();
  } catch (e) {
    error.value = `保存失败: ${e}`;
  }
}

async function doDelete(id) {
  try {
    await invoke('delete_connection', { id });
    if (editingId.value === id) {
      editingId.value = 0;
      name.value = '';
    }
    await loadSavedList();
  } catch (e) {
    error.value = `删除失败: ${e}`;
  }
}

// ---- 清理 ----

onUnmounted(() => {
  if (unlisten) { unlisten(); unlisten = null; }
  if (term) { term.dispose(); term = null; }
});
</script>

<style scoped>
.terminal-wrapper {
  width: 100%;
  height: 100%;
  position: relative;
  display: flex;
  flex-direction: column;
}

/* ---- 覆盖层 ---- */
.connect-overlay {
  position: absolute;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #1e1e1e;
}

.overlay-content {
  display: flex;
  gap: 24px;
  align-items: flex-start;
}

/* ---- 连接表单 ---- */
.connect-form {
  width: 420px;
  padding: 32px;
  background: #252526;
  border-radius: 8px;
  border: 1px solid #3e3e42;
}

.connect-form h2 {
  margin-bottom: 24px;
  text-align: center;
  color: #cccccc;
  font-size: 18px;
}

.form-group {
  display: flex;
  gap: 10px;
  margin-bottom: 14px;
}

.form-group input {
  flex: 1;
  padding: 10px 12px;
  background: #3c3c3c;
  border: 1px solid #555;
  border-radius: 4px;
  color: #d4d4d4;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.form-group input:focus {
  border-color: #007acc;
}

input[type='number']::-webkit-inner-spin-button,
input[type='number']::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.btn-row {
  display: flex;
  gap: 10px;
}

.btn-connect {
  flex: 1;
  padding: 10px;
  background: #007acc;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-connect:hover:not(:disabled) {
  background: #005999;
}

.btn-save {
  width: auto;
  padding: 10px 16px;
  background: #3a3a3a;
  color: #ccc;
  border: 1px solid #555;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.btn-save:hover:not(:disabled) {
  background: #4a4a4a;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.error {
  color: #f44747;
  margin-top: 12px;
  text-align: center;
  font-size: 13px;
}

/* ---- 已保存列表 ---- */
.saved-list {
  width: 280px;
  max-height: 380px;
  overflow-y: auto;
  background: #252526;
  border-radius: 8px;
  border: 1px solid #3e3e42;
  padding: 16px;
}

.saved-list h3 {
  margin-bottom: 12px;
  color: #cccccc;
  font-size: 15px;
}

.saved-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  margin-bottom: 6px;
  background: #2d2d30;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
}

.saved-item:hover {
  background: #37373d;
}

.saved-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.saved-name {
  font-size: 13px;
  color: #e0e0e0;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.saved-detail {
  font-size: 11px;
  color: #888;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.btn-del {
  width: 24px;
  height: 24px;
  padding: 0;
  background: transparent;
  color: #888;
  border: none;
  border-radius: 3px;
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.btn-del:hover {
  background: #c50f1f;
  color: white;
}

/* ---- 终端容器 ---- */
.term-container {
  flex: 1;
  padding: 4px;
}

/* ---- 工具栏 ---- */
.toolbar {
  position: absolute;
  top: 6px;
  right: 10px;
  display: flex;
  gap: 8px;
  z-index: 10;
}

.upload-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  background: #0e639c;
  border: none;
  border-radius: 4px;
  color: white;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s;
}

.upload-btn:hover {
  background: #1177bb;
}

.disconnect-btn {
  padding: 5px 14px;
  background: #c50f1f;
  font-size: 12px;
  border-radius: 4px;
  border: none;
  color: white;
  cursor: pointer;
}

.disconnect-btn:hover {
  background: #a00d19;
}
</style>
