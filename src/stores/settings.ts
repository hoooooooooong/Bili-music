import { defineStore } from "pinia";
import { ref } from "vue";
import type { AppSettings } from "@/types";
import { invoke } from "@tauri-apps/api/core";

export const useSettingsStore = defineStore("settings", () => {
  const outputDir = ref("");
  const theme = ref<"light" | "dark" | "system">("dark");
  const cacheSize = ref(500);
  const volume = ref(1);
  const loaded = ref(false);

  async function loadSettings() {
    try {
      const settings = await invoke<AppSettings>("get_settings");
      outputDir.value = settings.outputDir;
      theme.value = settings.theme;
      cacheSize.value = settings.cacheSize;
      volume.value = settings.volume;
      applyTheme(theme.value);
      loaded.value = true;
    } catch {
      applyTheme("dark");
      loaded.value = true;
    }
  }

  async function saveSettings() {
    try {
      await invoke("save_settings", {
        settings: {
          outputDir: outputDir.value,
          theme: theme.value,
          cacheSize: cacheSize.value,
          volume: volume.value,
        },
      });
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  }

  function setTheme(t: "light" | "dark" | "system") {
    theme.value = t;
    applyTheme(t);
    saveSettings();
  }

  function applyTheme(t: string) {
    const isDark =
      t === "dark" ||
      (t === "system" &&
        window.matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.setAttribute(
      "data-theme",
      isDark ? "dark" : "light"
    );
  }

  async function pickDirectory(): Promise<string | null> {
    try {
      const dir = await invoke<string | null>("pick_directory");
      if (dir) {
        outputDir.value = dir;
        await saveSettings();
      }
      return dir;
    } catch {
      return null;
    }
  }

  async function checkTools(): Promise<{ ffmpeg: boolean }> {
    return await invoke<{ ffmpeg: boolean }>("check_tools");
  }

  return {
    outputDir,
    theme,
    cacheSize,
    volume,
    loaded,
    loadSettings,
    saveSettings,
    setTheme,
    pickDirectory,
    checkTools,
  };
});
