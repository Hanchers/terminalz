<template>
  <div class="sp-panel">
    <div class="sp-toolbar">
      <input class="sp-search" placeholder="Search snippets..." />
      <button class="sp-btn">+ New</button>
    </div>
    <div class="sp-scroll">
      <div class="sp-cards">
        <div class="sp-card" v-for="s in demo" :key="s.name">
          <div class="sp-icon">{{ s.icon }}</div>
          <div class="sp-name">{{ s.name }}</div>
          <div class="sp-preview">{{ s.preview }}</div>
        </div>
      </div>
      <div class="sp-empty">
        Code snippet management coming soon.<br/>
        Save and reuse common commands across sessions.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const demo = [
  { icon: '📦', name: 'Docker Deploy', preview: 'docker compose pull && docker compose up -d' },
  { icon: '🔧', name: 'Systemd Restart', preview: 'systemctl restart nginx && systemctl status nginx' },
  { icon: '📊', name: 'Health Check', preview: 'curl -f http://localhost:8080/health || exit 1' },
  { icon: '📝', name: 'Log Rotate', preview: 'find /var/log -name "*.log" -mtime +7 -delete' },
]
</script>

<style scoped>
.sp-panel { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.sp-toolbar { display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; }
.sp-search { flex: 1; padding: 5px 10px; font-size: 12px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-primary); outline: none; }
.sp-search:focus { border-color: var(--color-accent); }
.sp-btn { padding: 5px 12px; font-size: 11px; background: transparent; border: 1px solid var(--color-border-input); border-radius: 4px; color: var(--color-text-secondary); cursor: pointer; }
.sp-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }
.sp-scroll { flex: 1; overflow-y: auto; padding: 12px; }
.sp-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 8px; margin-bottom: 16px; }
.sp-card { padding: 12px; background: var(--color-bg-input); border: 1px solid var(--color-border-input); border-radius: 6px; cursor: pointer; transition: all 0.12s; }
.sp-card:hover { border-color: var(--color-accent); }
.sp-icon { font-size: 20px; margin-bottom: 4px; }
.sp-name { font-size: 12px; font-weight: 500; color: var(--color-text-primary); }
.sp-preview { font-size: 10px; color: var(--color-text-tertiary); margin-top: 4px; font-family: monospace; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.sp-empty { padding: 24px; text-align: center; color: var(--color-text-tertiary); font-size: 12px; line-height: 1.6; }
</style>
