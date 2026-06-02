import { ref, type Ref } from 'vue'

export interface Theme {
  id: string
  name: string
  icon: string
}

export const themes: Theme[] = [
  { id: 'default', name: 'Dark',  icon: '🌙' },
  { id: 'light',   name: 'Light', icon: '☀️' },
]

export const currentTheme: Ref<string> = ref('default')

export function initTheme(): void {
  currentTheme.value = 'default'
  document.documentElement.setAttribute('data-theme', 'default')
}

export function setTheme(themeId: string): void {
  const theme = themes.find(t => t.id === themeId)
  if (!theme) {
    console.warn(`[Theme] Unknown theme: ${themeId}`)
    return
  }
  currentTheme.value = themeId
  document.documentElement.setAttribute('data-theme', themeId)
}

export function toggleTheme(): void {
  const idx = themes.findIndex(t => t.id === currentTheme.value)
  const next = themes[(idx + 1) % themes.length]
  setTheme(next.id)
}
