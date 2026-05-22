<template>
  <div class="terminal-wrapper">
    <!-- 连接表单（覆盖层） -->
    <div v-if="!connected" class="connect-overlay">
      <div class="connect-form">
        <h2>SSH 远程连接</h2>
        <div class="form-group">
          <input v-model="host" placeholder="主机地址 (例: 192.168.1.1)" @keyup.enter="connect" />
          <input
            v-model.number="port"
            placeholder="端口"
            type="number"
            style="max-width: 100px"
            @keyup.enter="connect"
          />
        </div>
        <div class="form-group">
          <input v-model="username" placeholder="用户名" @keyup.enter="connect" />
          <input
            v-model="password"
            placeholder="密码"
            type="password"
            @keyup.enter="connect"
          />
        </div>
        <button @click="connect" :disabled="connecting">
          {{ connecting ? '连接中...' : '连接' }}
        </button>
        <p v-if="error" class="error">{{ error }}</p>
      </div>
    </div>

    <!-- 终端（始终渲染，保持布局尺寸） -->
    <div ref="termContainer" class="term-container"></div>

    <!-- 断开按钮 -->
    <button v-if="connected" @click="disconnect" class="disconnect-btn">
      断开连接
    </button>
  </div>
</template>

<script setup>
import { ref, onUnmounted, nextTick } from 'vue';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import '@xterm/xterm/css/xterm.css';

const connected = ref(false);
const connecting = ref(false);
const error = ref('');
const host = ref('');
const port = ref(22);
const username = ref('');
const password = ref('');
const termContainer = ref(null);

let term = null;
let fitAddon = null;
let unlisten = null;

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
  // 确保终端已在 DOM 中打开并完成布局
  if (!term) {
    createTerminal();
  }
  term.open(termContainer.value);
  fitAddon.fit();
}

async function connect() {
  if (!host.value || !username.value || !password.value) {
    error.value = '请填写所有必填字段';
    return;
  }

  connecting.value = true;
  error.value = '';

  try {
    // 1. 先打开终端到可见容器，计算正确的 rows/cols
    ensureTerminalOpen();

    // 2. 注册 SSH 输出监听（在连接之前，避免丢失初始输出）
    unlisten = await listen('ssh-output', (event) => {
      if (term) {
        const data = new Uint8Array(event.payload.data);
        term.write(data);
      }
    });

    // 3. 将用户键盘输入发送到后端
    term.onData((data) => {
      invoke('ssh_write', {
        data: Array.from(new TextEncoder().encode(data)),
      }).catch(() => {});
    });

    // 4. 终端大小变化时通知后端
    term.onResize(({ rows, cols }) => {
      if (connected.value) {
        invoke('ssh_resize', { rows, cols }).catch(() => {});
      }
    });

    // 5. 发起 SSH 连接
    await invoke('ssh_connect', {
      host: host.value,
      port: port.value,
      username: username.value,
      password: password.value,
      rows: term.rows,
      cols: term.cols,
    });

    // 6. 连接成功，隐藏表单
    connected.value = true;

    // 7. 窗口大小变化时重新适配
    const onResize = () => {
      if (fitAddon) fitAddon.fit();
    };
    window.addEventListener('resize', onResize);
  } catch (e) {
    error.value = `连接失败: ${e}`;
    cleanupTerminal();
  } finally {
    connecting.value = false;
  }
}

function cleanupTerminal() {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  if (term) {
    term.dispose();
    term = null;
    fitAddon = null;
  }
}

async function disconnect() {
  try {
    await invoke('ssh_disconnect');
  } catch (_) {
    // ignore
  }

  cleanupTerminal();
  connected.value = false;
}

onUnmounted(() => {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  if (term) {
    term.dispose();
    term = null;
  }
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

/* 连接表单覆盖层 */
.connect-overlay {
  position: absolute;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #1e1e1e;
}

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

button {
  width: 100%;
  padding: 10px;
  background: #007acc;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

button:hover:not(:disabled) {
  background: #005999;
}

.error {
  color: #f44747;
  margin-top: 12px;
  text-align: center;
  font-size: 13px;
}

/* 终端容器 */
.term-container {
  flex: 1;
  padding: 4px;
}

/* 断开按钮 */
.disconnect-btn {
  position: absolute;
  top: 8px;
  right: 10px;
  width: auto;
  padding: 5px 14px;
  background: #c50f1f;
  font-size: 12px;
  z-index: 10;
  border-radius: 4px;
}

.disconnect-btn:hover {
  background: #a00d19;
}
</style>
