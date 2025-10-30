import { emit } from "@tauri-apps/api/event";
import { createPinia } from "pinia";
import { createApp } from "vue";
import { i18n } from "./i18n";
import "./styles.css";
import SettingsApp from "./views/SettingsApp.vue";

const app = createApp(SettingsApp);

const pinia = createPinia();
app.use(pinia);
app.use(i18n);

app.mount("#app");

console.log("📦 App mounted, emitting ready event immediately...");

setTimeout(async () => {
  try {
    console.log("📤 Emitting settings-ready event...");
    await emit("settings-ready");
    console.log("✅ Settings window ready!");
  } catch (error) {
    console.error("❌ Failed to emit settings-ready event:", error);
  }
}, 50);
