<template>
  <div class="kp-panel">
    <div class="kp-toolbar">
      <span class="kp-title">Keychain</span>
      <button class="kp-btn">+ New Entry</button>
    </div>
    <div class="kp-scroll">
      <div class="kp-cards">
        <div class="kp-card" v-for="i in 3" :key="i">
          <div class="kp-icon">🔑</div>
          <div class="kp-name">Shared Credential {{ i }}</div>
          <div class="kp-note">username: admin</div>
        </div>
      </div>
      <div class="kp-empty">
        Shared password management coming soon.<br/>
        Credentials can be shared across multiple hosts.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
</script>

<style scoped>
.kp-panel { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.kp-toolbar { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; }
.kp-title { font-size: 11px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.5px; }
.kp-btn { padding: 5px 12px; font-size: 11px; background: transparent; border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; }
.kp-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }
.kp-scroll { flex: 1; overflow-y: auto; padding: 12px; }
.kp-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 8px; margin-bottom: 16px; }
.kp-card { padding: 12px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 6px; cursor: pointer; transition: all 0.12s; }
.kp-card:hover { border-color: var(--color-accent); }
.kp-icon { font-size: 20px; margin-bottom: 4px; }
.kp-name { font-size: 12px; font-weight: 500; color: var(--color-text-primary); }
.kp-note { font-size: 10px; color: var(--color-text-tertiary); margin-top: 2px; }
.kp-empty { padding: 24px; text-align: center; color: var(--color-text-tertiary); font-size: 12px; line-height: 1.6; }
</style>
