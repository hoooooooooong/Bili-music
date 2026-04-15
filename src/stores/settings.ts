import { defineStore } from "pinia";
import { ref } from "vue";
import type { AppSettings, AudioFormat, AudioQuality, WindowGeometry } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";

// 存储主题切换按钮的点击坐标
let _themeClickX = 0;
let _themeClickY = 0;

export function setThemeClickOrigin(x: number, y: number) {
  _themeClickX = x;
  _themeClickY = y;
}
export const useSettingsStore = defineStore("settings", () => {
  const outputDir = ref("");
  const theme = ref<"light" | "dark" | "system">("dark");
  const cacheSize = ref(500);
  const volume = ref(1);
  const downloadFormat = ref<AudioFormat>("mp3");
  const downloadQuality = ref<AudioQuality>("high");
  const minimizeToTray = ref(true);
  const autostartEnabled = ref(false);
  const accentColor = ref("#fb7299");
  const windowGeometry = ref<WindowGeometry | null>(null);
  const desktopLyricsEnabled = ref(false);
  const desktopLyricsFontSize = ref(32);
  const desktopLyricsLocked = ref(false);
  const sessdata = ref("");
  const loaded = ref(false);

  async function loadSettings() {
    try {
      const settings = await invoke<AppSettings>("get_settings");
      outputDir.value = settings.outputDir;
      theme.value = settings.theme;
      cacheSize.value = settings.cacheSize;
      volume.value = settings.volume;
      downloadFormat.value = (settings.downloadFormat || "mp3") as AudioFormat;
      downloadQuality.value = (settings.downloadQuality || "high") as AudioQuality;
      minimizeToTray.value = settings.minimizeToTray ?? true;
      autostartEnabled.value = settings.autostartEnabled ?? false;
      accentColor.value = settings.accentColor ?? "#fb7299";
      desktopLyricsEnabled.value = (settings as any).desktopLyricsEnabled ?? false;
      desktopLyricsFontSize.value = (settings as any).desktopLyricsFontSize ?? 32;
      desktopLyricsLocked.value = (settings as any).desktopLyricsLocked ?? false;
      sessdata.value = (settings as any).sessdata ?? "";
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
          downloadFormat: downloadFormat.value,
          downloadQuality: downloadQuality.value,
          minimizeToTray: minimizeToTray.value,
          autostartEnabled: autostartEnabled.value,
          accentColor: accentColor.value,
          desktopLyricsEnabled: desktopLyricsEnabled.value,
          desktopLyricsFontSize: desktopLyricsFontSize.value,
          desktopLyricsLocked: desktopLyricsLocked.value,
          sessdata: sessdata.value,
        },
      });
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  }

  async function setAutostartEnabled(val: boolean) {
    autostartEnabled.value = val;
    try {
      if (val) {
        await enable();
      } else {
        await disable();
      }
    } catch (e) {
      console.error("Failed to set autostart:", e);
      // Revert on failure
      autostartEnabled.value = !val;
      return;
    }
    await saveSettings();
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
    const newTheme = isDark ? "dark" : "light";
    const oldTheme = document.documentElement.getAttribute("data-theme");

    if (oldTheme === newTheme) return;

    // 使用存储的点击坐标作为动画起点
    const x = _themeClickX ?? window.innerWidth / 2;
    const y = _themeClickY ?? window.innerHeight / 2;
    const radius = Math.hypot(
      Math.max(x, innerWidth - x),
      Math.max(y, innerHeight - y)
    );

    if (document.startViewTransition && oldTheme) {
      const transition = document.startViewTransition(() => {
        document.documentElement.setAttribute("data-theme", newTheme);
      });

      transition.ready.then(() => {

        if (isDark) {
          document.documentElement.animate(
            {
              clipPath: [
                `circle(0% at ${x}px ${y}px)`,
                `circle(${radius}px at ${x}px ${y}px)`,
              ],
            },
            {
              duration: 500,
              easing: "ease-out",
              fill: "forwards",
              pseudoElement: "::view-transition-new(root)",
            }
          );
        } else {
          const style = document.createElement("style");
          style.id = "vt-z-index-fix";
          style.textContent = `
            ::view-transition-old(root) { z-index: 9999 !important; }
            ::view-transition-new(root) { z-index: 1 !important; }
          `;
          document.head.appendChild(style);

          document.documentElement.animate(
            {
              clipPath: [
                `circle(${radius}px at ${x}px ${y}px)`,
                `circle(0% at ${x}px ${y}px)`,
              ],
            },
            {
              duration: 500,
              easing: "ease-in",
              fill: "forwards",
              pseudoElement: "::view-transition-old(root)",
            }
          );

          transition.finished.then(() => style.remove());
        }
      });
    } else {
      document.documentElement.setAttribute("data-theme", newTheme);
    }
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
    downloadFormat,
    downloadQuality,
    minimizeToTray,
    autostartEnabled,
    accentColor,
    windowGeometry,
    desktopLyricsEnabled,
    desktopLyricsFontSize,
    desktopLyricsLocked,
    sessdata,
    loaded,
    loadSettings,
    saveSettings,
    setAutostartEnabled,
    setTheme,
    pickDirectory,
    checkTools,
  };
});
