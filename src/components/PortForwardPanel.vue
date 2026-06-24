<template>
  <div class="pf-panel">
    <div class="pf-toolbar">
      <span class="pf-title">Port Forwarding</span>
      <button class="pf-btn">+ New Rule</button>
    </div>
    <div class="pf-scroll">
      <div class="pf-cards">
        <div class="pf-card" v-for="i in 2" :key="i">
          <div class="pf-icon">🔗</div>
          <div class="pf-name">Forward Rule {{ i }}</div>
          <div class="pf-note">localhost:{{ 3000 + i }} → remote:{{ 80 + i }}</div>
          <div class="pf-status">○ Stopped</div>
        </div>
      </div>
      <div class="pf-empty">
        SSH tunnel port forwarding coming soon.<br/>
        Map local ports to remote services through SSH tunnels.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
</script>

<style scoped>
.pf-panel { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.pf-toolbar { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; }
.pf-title { font-size: 11px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.5px; }
.pf-btn { padding: 5px 12px; font-size: 11px; background: transparent; border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; }
.pf-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }
.pf-scroll { flex: 1; overflow-y: auto; padding: 12px; }
.pf-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 8px; margin-bottom: 16px; }
.pf-card { padding: 12px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 6px; cursor: pointer; transition: all 0.12s; }
.pf-card:hover { border-color: var(--color-accent); }
.pf-icon { font-size: 20px; margin-bottom: 4px; }
.pf-name { font-size: 12px; font-weight: 500; color: var(--color-text-primary); }
.pf-note { font-size: 10px; color: var(--color-text-tertiary); margin-top: 2px; }
.pf-status { font-size: 10px; color: var(--color-text-tertiary); margin-top: 4px; }
.pf-empty { padding: 24px; text-align: center; color: var(--color-text-tertiary); font-size: 12px; line-height: 1.6; }
</style>
