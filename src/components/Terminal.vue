<template>
  <div class="terminal-wrapper">
    <!-- 标签栏 -->
    <div v-if="connected" class="tab-bar">
      <div class="tab active">
        <svg viewBox="0 0 24 24" width="12" height="12" fill="#3fb950">
          <circle cx="12" cy="12" r="4"/>
        </svg>
        <span>{{ mode === 'local' ? $t('terminal.local') : host }}</span>
      </div>
      <div class="tab-actions">
        <button v-if="mode !== 'local'" @click="showFileTransfer = true" class="tab-btn" :title="$t('terminal.uploadFiles')">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="currentColor">
            <path d="M9 16h6v-6h4l-7-7-7 7h4zm-4 2h14v2H5z"/>
          </svg>
        </button>
        <button @click="disconnect" class="tab-btn close-btn" :title="$t('terminal.disconnect')">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- 欢迎页 -->
    <div v-if="!connected && !prefill && mode !== 'local'" class="welcome">
      <div class="welcome-content">
        <div class="welcome-logo">
          <svg viewBox="0 0 64 64" width="64" height="64" fill="none">
            <rect x="4" y="14" width="56" height="36" rx="4" stroke="currentColor" stroke-width="2.5"/>
            <path d="M16 28h8M28 28h8M40 28h8M16 36h6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            <circle cx="52" cy="20" r="3" fill="currentColor"/>
          </svg>
        </div>
        <h1>TerminalZ</h1>
        <p class="welcome-sub">{{ $t('terminal.welcome.subtitle') }}</p>
        <div class="welcome-hint">
          <span class="hint-arrow">←</span>
          <span>{{ $t('terminal.welcome.hint') }}</span>
        </div>
        <div class="welcome-shortcuts">
          <div class="sc-item">
            <kbd>Hosts</kbd>
            <span>{{ $t('terminal.welcome.scHosts') }}</span>
          </div>
          <div class="sc-item">
            <kbd>Ctrl</kbd> + <kbd>U</kbd>
            <span>{{ $t('terminal.welcome.scUpload') }}</span>
          </div>
          <div class="sc-item">
            <kbd>Settings</kbd>
            <span>{{ $t('terminal.welcome.scTheme') }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 连接表单覆盖层 -->
    <div v-if="!connected && prefill && mode !== 'local'" class="connect-overlay">
      <div class="connect-form">
        <h2>{{ $t('terminal.form.title') }}</h2>
        <div class="form-group">
          <input v-model="name" :placeholder="$t('terminal.form.namePlaceholder')" @keyup.enter="doConnect" />
        </div>
        <div class="form-group">
          <input v-model="host" :placeholder="$t('terminal.form.hostPlaceholder')" @keyup.enter="doConnect" />
          <input v-model.number="port" :placeholder="$t('terminal.form.portPlaceholder')" type="number" style="max-width:100px" @keyup.enter="doConnect" />
        </div>
        <div class="form-group">
          <input v-model="username" :placeholder="$t('terminal.form.usernamePlaceholder')" @keyup.enter="doConnect" />
          <div class="password-wrap">
            <input
              v-model="password"
              :placeholder="$t('terminal.form.passwordPlaceholder')"
              :type="showPassword ? 'text' : 'password'"
              @keyup.enter="doConnect"
            />
            <button class="eye-btn" type="button" @click="showPassword = !showPassword" tabindex="-1">
              <svg v-if="showPassword" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/>
              </svg>
              <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                <path d="M12 7c2.76 0 5 2.24 5 5 0 .65-.13 1.26-.36 1.83l2.92 2.92c1.51-1.26 2.7-2.89 3.43-4.75-1.73-4.39-6-7.5-11-7.5-1.4 0-2.74.25-3.98.7l2.16 2.16C10.74 7.13 11.35 7 12 7zM2 4.27l2.28 2.28.46.46C3.08 8.3 1.78 10.02 1 12c1.73 4.39 6 7.5 11 7.5 1.55 0 3.03-.3 4.38-.84l.42.42L19.73 22 21 20.73 3.27 3 2 4.27zM7.53 9.8l1.55 1.55c-.05.21-.08.43-.08.65 0 1.66 1.34 3 3 3 .22 0 .44-.03.65-.08l1.55 1.55c-.67.33-1.41.53-2.2.53-2.76 0-5-2.24-5-5 0-.79.2-1.53.53-2.2zm4.31-.78l3.15 3.15.02-.16c0-1.66-1.34-3-3-3l-.17.01z"/>
              </svg>
            </button>
          </div>
        </div>
        <div class="btn-row">
          <button class="btn-connect" @click="doConnect" :disabled="connecting">
            {{ connecting ? $t('terminal.form.connecting') : $t('terminal.form.connect') }}
          </button>
          <button class="btn-save" @click="doSave" :disabled="!canSave">{{ $t('terminal.form.save') }}</button>
        </div>
        <p v-if="error" class="error">{{ error }}</p>
      </div>
    </div>

    <!-- 终端 -->
    <div ref="termContainer" class="term-container"></div>

    <!-- 文件传输面板 -->
    <FileTransfer v-if="showFileTransfer" @close="showFileTransfer = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import '@xterm/xterm/css/xterm.css';
import FileTransfer from './FileTransfer.vue';

const { t } = useI18n();

interface HostPrefill { id?: number; name?: string; host: string; port: number; username: string; password: string }

const props = defineProps<{ prefill: HostPrefill | null; mode: 'ssh' | 'local' | null }>()
const emit = defineEmits<{ 'connection-change': [connected: boolean] }>()

const connected = ref(false);
const connecting = ref(false);
const error = ref('');
const name = ref('');
const host = ref('');
const port = ref(22);
const username = ref('');
const password = ref('');
const showPassword = ref(false);
const termContainer = ref<HTMLDivElement | null>(null);
const editingId = ref(0);
const showFileTransfer = ref(false);

let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let unlisten: (() => void) | null = null;

const canSave = computed(() => host.value && username.value);

// ---- 外部 prefill ---

watch(() => props.prefill, (val: HostPrefill | null) => {
  if (val) {
    name.value = val.name || '';
    host.value = val.host || '';
    port.value = val.port || 22;
    username.value = val.username || '';
    password.value = val.password || '';
    editingId.value = val.id || 0;
    error.value = '';
  }
}, { deep: true })

onMounted(() => {});

// ---- 本地终端自动连接 ----

watch(() => props.mode, (newMode) => {
  if (newMode === 'local' && !connected.value && !connecting.value) {
    doConnect();
  }
});

// ---- 终端操作 ----

function createTerminal(): void {
  term = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: readTerminalTheme(),
    allowProposedApi: true,
  });

  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
}

function readTerminalTheme(): { background: string; foreground: string; cursor: string } {
  const style = getComputedStyle(document.documentElement);
  return {
    background: style.getPropertyValue('--terminal-bg').trim(),
    foreground: style.getPropertyValue('--terminal-fg').trim(),
    cursor:    style.getPropertyValue('--terminal-cursor').trim(),
  };
}

function ensureTerminalOpen(): void {
  if (!term) createTerminal();
  term.open(termContainer.value);
  fitAddon.fit();
}

async function doConnect() {
  // ---- 本地终端模式 ----
  if (props.mode === 'local') {
    connecting.value = true;
    error.value = '';
    try {
      ensureTerminalOpen();

      unlisten = await listen<{ data: number[] }>('local-output', (event) => {
        if (term) {
          const data = new Uint8Array(event.payload.data);
          term.write(data);
        }
      });

      term.onData((data) => {
        invoke('local_term_write', {
          data: Array.from(new TextEncoder().encode(data)),
        }).catch(() => {});
      });

      term.onResize(({ rows, cols }) => {
        if (connected.value) {
          invoke('local_term_resize', { rows, cols }).catch(() => {});
        }
      });

      await invoke('local_term_start', {
        rows: term.rows,
        cols: term.cols,
      });

      connected.value = true;
      emit('connection-change', true);

      window.addEventListener('resize', () => fitAddon?.fit());
    } catch (e) {
      error.value = `${t('terminal.error.localStartFailed')}${e}`;
      cleanupTerminal();
    } finally {
      connecting.value = false;
    }
    return;
  }

  // ---- SSH 远程连接 ----
  if (!host.value || !username.value || !password.value) {
    error.value = t('terminal.error.fillRequired');
    return;
  }

  connecting.value = true;
  error.value = '';

  try {
    ensureTerminalOpen();

    unlisten = await listen<{ data: number[] }>('ssh-output', (event) => {
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
    emit('connection-change', true);

    window.addEventListener('resize', () => fitAddon?.fit());
  } catch (e) {
    error.value = `${t('terminal.error.connectFailed')}${e}`;
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
  if (props.mode === 'local') {
    try { await invoke('local_term_close'); } catch (_) {}
  } else {
    try { await invoke('ssh_disconnect'); } catch (_) {}
  }
  cleanupTerminal();
  connected.value = false;
  emit('connection-change', false);
}

// ---- 数据库操作 ----

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
    const saved = await invoke<{ id: number }>('save_connection', { config });
    editingId.value = saved.id;
    error.value = t('terminal.error.saved');
    setTimeout(() => { error.value = ''; }, 1500);
  } catch (e) {
    error.value = `${t('terminal.error.saveFailed')}${e}`;
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
  background: var(--color-bg-primary);
}

/* ---- 欢迎页 ---- */
.welcome {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-primary);
  z-index: 10;
}

.welcome-content {
  text-align: center;
  max-width: 420px;
}

.welcome-logo {
  color: var(--color-accent);
  margin-bottom: 16px;
  opacity: 0.7;
}

.welcome h1 {
  font-size: 28px;
  font-weight: 700;
  color: var(--color-text-primary);
  letter-spacing: 1px;
  margin-bottom: 6px;
}

.welcome-sub {
  font-size: 14px;
  color: var(--color-text-secondary);
  margin-bottom: 32px;
}

.welcome-hint {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: var(--color-bg-panel);
  border: 1px solid var(--color-border-primary);
  border-radius: 8px;
  font-size: 13px;
  color: var(--color-text-secondary);
  margin-bottom: 36px;
}

.hint-arrow {
  font-size: 16px;
  color: var(--color-accent);
  animation: hintPulse 2s ease-in-out infinite;
}

@keyframes hintPulse {
  0%, 100% { opacity: 1; transform: translateX(0); }
  50% { opacity: 0.4; transform: translateX(-4px); }
}

.welcome-shortcuts {
  display: flex;
  flex-direction: column;
  gap: 8px;
  text-align: left;
}

.sc-item {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.sc-item kbd {
  display: inline-block;
  padding: 2px 7px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border-primary);
  border-radius: 4px;
  font-family: monospace;
  font-size: 11px;
  color: var(--color-text-secondary);
  min-width: 24px;
  text-align: center;
}

/* ---- 标签栏 ---- */
.tab-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 32px;
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border-tab);
  padding: 0 8px;
  flex-shrink: 0;
}

.tab {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 12px;
  background: var(--color-bg-primary);
  border-radius: 4px 4px 0 0;
  font-size: 12px;
  color: var(--color-text-heading);
  border: 1px solid var(--color-border-tab);
  border-bottom: 1px solid var(--color-bg-primary);
  margin-bottom: -1px;
}

.tab-actions {
  display: flex;
  gap: 2px;
}

.tab-btn {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.tab-btn:hover {
  background: var(--color-bg-active);
  color: var(--color-text-primary);
}

.close-btn:hover {
  background: var(--color-danger-btn);
  color: var(--color-text-white);
}

/* ---- 覆盖层 ---- */
.connect-overlay {
  position: absolute;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-primary);
}

/* ---- 连接表单 ---- */
.connect-form {
  width: 420px;
  padding: 32px;
  background: var(--color-bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--color-border-tab);
}

.connect-form h2 {
  margin-bottom: 24px;
  text-align: center;
  color: var(--color-text-heading);
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
  background: var(--color-bg-form);
  border: 1px solid var(--color-border-input);
  border-radius: 4px;
  color: var(--color-text-terminal);
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.form-group input:focus {
  border-color: var(--color-btn-primary);
}

input[type='number']::-webkit-inner-spin-button,
input[type='number']::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

/* 密码眼睛按钮 */
.password-wrap {
  position: relative;
  flex: 1;
  display: flex;
}
.password-wrap input {
  flex: 1;
  padding-right: 36px;
}
.eye-btn {
  position: absolute;
  right: 2px;
  top: 50%;
  transform: translateY(-50%);
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--color-text-tertiary);
  cursor: pointer;
}
.eye-btn:hover {
  color: var(--color-text-secondary);
}

.btn-row {
  display: flex;
  gap: 10px;
}

.btn-connect {
  flex: 1;
  padding: 10px;
  background: var(--color-btn-primary);
  color: var(--color-text-white);
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-connect:hover:not(:disabled) {
  background: var(--color-btn-primary-hover);
}

.btn-save {
  width: auto;
  padding: 10px 16px;
  background: var(--color-btn-save);
  color: var(--color-btn-save-text);
  border: 1px solid var(--color-border-input);
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.btn-save:hover:not(:disabled) {
  background: var(--color-btn-save-hover);
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.error {
  color: var(--color-danger);
  margin-top: 12px;
  text-align: center;
  font-size: 13px;
}

/* ---- 终端容器 ---- */
.term-container {
  flex: 1;
  padding: 2px;
}
</style>
