/**
 * TerminalZ — 主题管理器
 *
 * 架构：
 *   - CSS 变量定义在 themes/*.css 中
 *   - default.css 使用 :root 定义默认（暗色）变量
 *   - light.css 使用 html[data-theme="light"] 覆盖变量
 *   - setTheme() 通过改变 html 的 data-theme 属性切换
 *   - 所有组件通过 CSS 变量引用颜色，无需 JS 逻辑
 *
 * 添加新主题：
 *   1. 在 src/themes/ 下新建 xxx.css
 *      - 使用 html[data-theme="xxx"] 覆盖需要改变的变量
 *   2. 在下方 themes 数组中注册
 *   3. 在 index.html 中引入新 CSS 文件
 *   4. 通过 setTheme('xxx') 切换
 */

import { ref } from 'vue'

/** 可用主题列表 */
export const themes = [
  { id: 'default', name: 'Dark',  icon: '🌙' },
  { id: 'light',   name: 'Light', icon: '☀️' },
]

/** 当前激活的主题 id */
export const currentTheme = ref('default')

/**
 * 初始化主题系统（在 main.js 中调用一次）
 */
export function initTheme() {
  // 默认使用 dark 主题
  currentTheme.value = 'default'
  document.documentElement.setAttribute('data-theme', 'default')
}

/**
 * 切换主题
 * @param {string} themeId - 主题 id
 */
export function setTheme(themeId) {
  const theme = themes.find(t => t.id === themeId)
  if (!theme) {
    console.warn(`[Theme] Unknown theme: ${themeId}`)
    return
  }
  currentTheme.value = themeId
  document.documentElement.setAttribute('data-theme', themeId)
}

/**
 * 切换到下一个主题（循环）
 */
export function toggleTheme() {
  const idx = themes.findIndex(t => t.id === currentTheme.value)
  const next = themes[(idx + 1) % themes.length]
  setTheme(next.id)
}
