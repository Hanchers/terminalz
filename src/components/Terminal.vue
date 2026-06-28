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

    <!-- 连接进度（双击 host 直接建立连接） -->
    <div v-if="!connected && prefill && mode !== 'local'" class="connect-overlay">
      <div class="connect-form" style="text-align:center;max-width:380px">
        <h3>{{ prefill.name || prefill.host }}</h3>
        <p class="connect-sub">{{ prefill.username }}@{{ prefill.host }}:{{ prefill.port }}</p>
        <div class="connect-log">
          <div v-if="connecting && statusLog.length === 0" class="log-line">
            <span class="log-dot"></span>
            <span class="log-msg">{{ $t('terminal.form.connecting') }}…</span>
          </div>
          <div v-for="(s, i) in statusLog" :key="i" class="log-line">
            <span class="log-dot" :class="{ 'dot-ok': s.step === 'done', 'dot-err': s.step === 'err' }"></span>
            <span class="log-msg" :class="{ 'log-ok': s.step === 'done', 'log-err': s.step === 'err' }">{{ s.detail }}</span>
          </div>
        </div>
        <p v-if="!connecting && error" class="error" style="margin-top:12px">{{ error }}</p>
        <button v-if="!connecting && error" class="btn-connect" style="margin-top:8px" @click="doConnect">{{ $t('terminal.form.connect') }}</button>
        <p v-if="connecting && showTimeoutTip" class="timeout-tip">{{ $t('terminal.error.connectingSlow') }}</p>
      </div>
    </div>

    <!-- 终端 -->
    <div ref="termContainer" class="term-container"></div>

    <!-- 文件传输面板 -->
    <FileTransfer v-if="showFileTransfer" @close="showFileTransfer = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import '@xterm/xterm/css/xterm.css';
import FileTransfer from './FileTransfer.vue';

const { t } = useI18n({ useScope: 'global' });

interface HostPrefill { id: number; name?: string; host: string; port: number; username: string; password?: string; remark?: string }
interface Tag { id: number; name: string; color: string }

const props = defineProps<{ prefill: HostPrefill | null; mode: 'ssh' | 'local' | null }>()
const emit = defineEmits<{ 'connection-change': [connected: boolean] }>()

const connected = ref(false);
const connecting = ref(false);
const connectAttempts = ref(0);
const MAX_ATTEMPTS = 3;
const error = ref('');
const connectionId = ref(0);
const name = ref('');
const host = ref('');
const port = ref(22);
const username = ref('');
const remark = ref('');
const tags = ref<Tag[]>([]);
const termContainer = ref<HTMLDivElement | null>(null);
const showFileTransfer = ref(false);
const statusLog = ref<{ step: string; detail: string }[]>([]);
const showTimeoutTip = ref(false);
let connectTimeout: ReturnType<typeof setTimeout> | null = null;
let autoConnectTriggered = false;

let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let unlisten: (() => void) | null = null;

// ---- 外部 prefill ---

// When prefill is set (double-click or Save & Connect), auto-connect
watch(() => props.prefill, async (val: HostPrefill | null) => {
  console.log('[Terminal] prefill watcher:', val, 'mode:', props.mode, 'connected:', connected.value, 'connecting:', connecting.value);
  if (val) {
    connectionId.value = val.id || 0;
    name.value = val.name || '';
    host.value = val.host || '';
    port.value = val.port || 22;
    username.value = val.username || '';
    remark.value = val.remark || '';
    error.value = '';
    // Auto-connect immediately, don't wait for tag loading
    if (!autoConnectTriggered && props.mode === 'ssh' && val.id && val.host && val.username && !connected.value && !connecting.value) {
      console.log('[Terminal] Triggering doConnect for', val.host);
      autoConnectTriggered = true;
      doConnect()
    }
    // load tags in background
    if (val.id) {
      try { tags.value = await invoke<Tag[]>('get_host_tags', { hostId: val.id }) } catch (_) { tags.value = [] }
    } else {
      tags.value = []
    }
  }
}, { deep: true })

onMounted(() => {
  // If prefill is already available on mount, trigger auto-connect
  if (!autoConnectTriggered && props.prefill && props.mode === 'ssh' && props.prefill.id && props.prefill.host && props.prefill.username && !connected.value && !connecting.value) {
    console.log('[Terminal] onMounted: triggering doConnect for', props.prefill.host);
    autoConnectTriggered = true;
    // Use nextTick to ensure terminal container is ready
    nextTick(() => doConnect());
  }
});

// ---- 本地终端自动连接 ----

watch(() => props.mode, (newMode) => {
  if (newMode === 'local' && !connected.value && !connecting.value) {
    connectAttempts.value = 0; // 切换模式时重置
    doConnect();
  } else if (newMode === 'ssh') {
    connectAttempts.value = 0;
  } else if (!newMode) {
    connectAttempts.value = 0;
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
  if (!termContainer.value) {
    console.warn('[Terminal] termContainer not ready');
    return;
  }
  if (!term) createTerminal();
  term.open(termContainer.value);
  fitAddon.fit();
}

async function doConnect() {
  // ---- 本地终端模式 ----
  if (props.mode === 'local') {
    if (connectAttempts.value >= MAX_ATTEMPTS) {
      error.value = t('terminal.error.maxAttemptsLocal', { n: MAX_ATTEMPTS });
      return;
    }
    connectAttempts.value++;
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
      connectAttempts.value = 0;
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
  // Use props.prefill directly to avoid race between autoConnect/prefill watchers
  const cid = props.prefill?.id || connectionId.value;
  const h = props.prefill?.host || host.value;
  const u = props.prefill?.username || username.value;
  const p = props.prefill?.port || port.value;

  console.log('[Terminal] doConnect SSH', { host: h, username: u, id: cid });

  if (!h || !u) {
    error.value = t('terminal.error.fillRequired');
    console.warn('[Terminal] Missing host/username');
    return;
  }

  if (!cid) {
    error.value = 'No connection ID — please select a saved host.';
    console.warn('[Terminal] Missing connectionId');
    return;
  }

  if (connectAttempts.value >= MAX_ATTEMPTS) {
    error.value = t('terminal.error.maxAttemptsSsh', { n: MAX_ATTEMPTS });
    return;
  }

  connectAttempts.value++;
  connecting.value = true;
  error.value = '';
  showTimeoutTip.value = false;
  statusLog.value = [{ step: 'start', detail: `Connecting to ${h}:${p}…` }];

  // Show a slow-connection tip after 8 seconds if still waiting
  connectTimeout = setTimeout(() => { showTimeoutTip.value = true }, 8000);

  let unlistenSteps: (() => void) | null = null;
  try {
    ensureTerminalOpen();

    unlistenSteps = await listen<{ step: string; detail: string }>('connect-step', (e) => {
      console.log('[Terminal] connect-step:', e.payload);
      statusLog.value.push({ step: e.payload.step, detail: e.payload.detail })
    });

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

    console.log('[Terminal] Invoking ssh_connect, cid:', cid);
    await invoke('ssh_connect', {
      connectionId: cid,
      rows: term.rows,
      cols: term.cols,
    });
    console.log('[Terminal] ssh_connect returned OK');

    connected.value = true;
    connectAttempts.value = 0;
    emit('connection-change', true);
    unlistenSteps?.();

    window.addEventListener('resize', () => fitAddon?.fit());
  } catch (e) {
    console.error('[Terminal] connect error:', e);
    unlistenSteps?.();
    error.value = `${t('terminal.error.connectFailed')}${e}`;
    cleanupTerminal();
  } finally {
    clearTimeout(connectTimeout!);
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
  autoConnectTriggered = false; // Reset for next auto-connect
  emit('connection-change', false);
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
  flex: 1;
  min-height: 0;
  position: relative;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-primary);
  overflow: hidden;
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

.connect-sub { font-size: 13px; color: var(--color-text-secondary); margin: 0 0 16px; font-family: monospace; }

/* ---- 连接日志 ---- */
.connect-log { text-align: left; margin: 0 auto; max-width: 320px; }
.log-line { display: flex; align-items: center; gap: 8px; padding: 3px 0; }
.log-dot {
  width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0;
  background: var(--color-text-tertiary);
  animation: pulse 1s infinite;
}
.log-dot.dot-ok { background: var(--color-success); animation: none; }
.log-dot.dot-err { background: var(--color-danger); animation: none; }
.log-msg { font-size: 12px; color: var(--color-text-secondary); }
.log-ok { color: var(--color-success); }
.log-err { color: var(--color-danger); }
.timeout-tip { margin: 12px 0 0; padding: 8px 12px; font-size: 11px; color: var(--color-text-secondary); background: var(--color-bg-secondary); border-radius: 4px; line-height: 1.5; }

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

.input-readonly {
  opacity: 0.65;
  cursor: default;
}

.form-label {
  display: block;
  font-size: 11px;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}

.remark-field {
  padding: 8px 12px;
  font-size: 13px;
  color: var(--color-text-secondary);
  border-radius: 4px;
  min-height: 20px;
  white-space: pre-wrap;
  word-break: break-word;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  min-height: 20px;
}

.tag-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 8px;
  font-size: 11px;
  color: #fff;
  white-space: nowrap;
}

.tag-none {
  font-size: 11px;
  color: var(--color-text-tertiary);
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
