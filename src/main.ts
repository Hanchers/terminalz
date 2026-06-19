import { createApp } from "vue";
import App from "./App.vue";
import { initTheme } from "./themes/index";
import { i18n } from "./i18n";
import "./styles/dialog.css";

initTheme();
createApp(App).use(i18n).mount("#app");
