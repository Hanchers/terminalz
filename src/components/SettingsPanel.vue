<template>
  <div class="st-panel">
    <div class="st-header">Settings</div>
    <div class="st-scroll">
      <div class="st-section">
        <div class="st-section-title">Appearance</div>
        <div class="st-group">
          <label class="st-label">Theme</label>
          <div class="st-theme-grid">
            <button
              v-for="t in themes" :key="t.id"
              class="st-theme-card"
              :class="{ active: currentTheme === t.id }"
              @click="applyTheme(t.id)"
            >
              <span class="st-theme-icon">{{ t.icon }}</span>
              <span class="st-theme-name">{{ t.name }}</span>
            </button>
          </div>
        </div>
      </div>

      <div class="st-section">
        <div class="st-section-title">Language</div>
        <div class="st-group">
          <label class="st-label">Display Language</label>
          <div class="st-lang-grid">
            <button
              v-for="l in languageOptions" :key="l.id"
              class="st-lang-card"
              :class="{ active: currentLocale === l.id }"
              @click="switchLang(l.id)"
            >
              <span class="st-lang-icon">{{ l.icon }}</span>
              <span class="st-lang-name">{{ l.name }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { currentTheme, themes, setTheme } from '../themes/index'
import { locales, saveLocale, type SupportedLocale } from '../i18n'

const { locale } = useI18n({ useScope: 'global' })
const currentLocale = computed(() => locale.value)
const languageOptions = locales

function applyTheme(id: string) { setTheme(id) }
function switchLang(id: string) { locale.value = id; saveLocale(id as SupportedLocale) }
</script>

<style scoped>
.st-panel { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.st-header { padding: 12px 16px; font-size: 14px; font-weight: 600; color: var(--color-text-heading); border-bottom: 1px solid var(--color-border-secondary); flex-shrink: 0; }
.st-scroll { flex: 1; overflow-y: auto; padding: 16px; }
.st-section { margin-bottom: 24px; }
.st-section-title { font-size: 11px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 10px; }
.st-group { margin-bottom: 12px; }
.st-label { display: block; font-size: 12px; color: var(--color-text-primary); margin-bottom: 8px; }
.st-theme-grid, .st-lang-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: 8px; }
.st-theme-card, .st-lang-card {
  display: flex; align-items: center; gap: 8px;
  padding: 10px 12px; background: var(--color-bg-input);
  border: 1px solid var(--color-border-input); border-radius: 6px;
  cursor: pointer; transition: all 0.12s; font-size: 12px;
  color: var(--color-text-secondary);
}
.st-theme-card:hover, .st-lang-card:hover { border-color: var(--color-accent); }
.st-theme-card.active, .st-lang-card.active {
  border-color: var(--color-accent); color: var(--color-text-primary);
  background: var(--color-accent-bg);
}
.st-theme-icon, .st-lang-icon { font-size: 16px; flex-shrink: 0; }
.st-theme-name, .st-lang-name { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
