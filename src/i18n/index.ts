import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN'
import enUS from './locales/en-US'

export type SupportedLocale = 'zh-CN' | 'en-US'

export const locales: { id: SupportedLocale; name: string; icon: string }[] = [
  { id: 'zh-CN', name: '简体中文', icon: '🇨🇳' },
  { id: 'en-US', name: 'English',   icon: '🇺🇸' },
]

const STORAGE_KEY = 'terminalz-locale'

function getSavedLocale(): SupportedLocale {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved && locales.some(l => l.id === saved)) {
      return saved as SupportedLocale
    }
  } catch (_) {}
  // Detect system language — check full list, not just primary.
  const langs = navigator.languages || [navigator.language]
  const match = langs.find(l => l.startsWith('zh'))
  return match ? 'zh-CN' : 'en-US'
}

export function saveLocale(id: SupportedLocale): void {
  try { localStorage.setItem(STORAGE_KEY, id) } catch (_) {}
}

export const i18n = createI18n({
  legacy: false,
  locale: getSavedLocale(),
  fallbackLocale: 'en-US',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
})
