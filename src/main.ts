import { createApp } from "vue";
import App from "./App.vue";
import { initTheme } from "./themes/index";

initTheme();
createApp(App).mount("#app");
