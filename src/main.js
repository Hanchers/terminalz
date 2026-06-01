import { createApp } from "vue";
import App from "./App.vue";
import { initTheme } from "./themes/index.js";

// 初始化主题系统
initTheme();

createApp(App).mount("#app");
