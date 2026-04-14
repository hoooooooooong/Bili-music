import { createApp } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import App from "./App.vue";
import router from "./router";
import "./styles/global.css";
import "./styles/theme.css";
import { getCurrentWindow } from "@tauri-apps/api/window";

const windowLabel = getCurrentWindow().label;

// Add window-specific class for transparent windows
if (windowLabel === "desktop-lyrics" || windowLabel === "mini-player") {
  document.documentElement.classList.add("transparent-window");
}

const app = createApp(App);
const pinia = createPinia();
pinia.use(piniaPluginPersistedstate);

app.use(pinia);
app.use(router);
app.mount("#app");
