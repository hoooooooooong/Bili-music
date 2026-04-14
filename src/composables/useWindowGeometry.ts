import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { WindowGeometry } from "@/types";
import { useSettingsStore } from "@/stores/settings";

let _debounceTimer: ReturnType<typeof setTimeout> | null = null;

export function useWindowGeometry() {
  let _unlisteners: UnlistenFn[] = [];

  async function startTracking() {
    const window = getCurrentWindow();
    const settingsStore = useSettingsStore();

    const unlistenResize = await window.onResized(async () => {
      const maximized = await window.isMaximized();
      if (maximized) {
        saveGeometry({ ...settingsStore.windowGeometry!, maximized: true } as WindowGeometry);
        return;
      }
      const size = await window.innerSize();
      saveGeometry({
        ...settingsStore.windowGeometry!,
        width: size.width,
        height: size.height,
        maximized: false,
      } as WindowGeometry);
    });

    const unlistenMove = await window.onMoved(async () => {
      const pos = await window.outerPosition();
      saveGeometry({
        ...settingsStore.windowGeometry!,
        x: pos.x,
        y: pos.y,
      } as WindowGeometry);
    });

    _unlisteners = [unlistenResize, unlistenMove];
  }

  function saveGeometry(geometry: WindowGeometry) {
    if (_debounceTimer) clearTimeout(_debounceTimer);
    _debounceTimer = setTimeout(async () => {
      const settingsStore = useSettingsStore();
      settingsStore.windowGeometry = geometry;
      await settingsStore.saveSettings();
    }, 300);
  }

  function stopTracking() {
    if (_debounceTimer) {
      clearTimeout(_debounceTimer);
      _debounceTimer = null;
    }
    _unlisteners.forEach((fn) => fn());
    _unlisteners = [];
  }

  return { startTracking, stopTracking };
}
