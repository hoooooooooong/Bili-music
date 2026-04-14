import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

async function getWindow(label: string) {
  return await WebviewWindow.getByLabel(label);
}

export function useWindowManager() {
  async function showDesktopLyrics() {
    const win = await getWindow("desktop-lyrics");
    if (win) {
      await win.show();
    }
  }

  async function hideDesktopLyrics() {
    const win = await getWindow("desktop-lyrics");
    if (win) {
      await win.hide();
    }
  }

  async function toggleDesktopLyrics() {
    const win = await getWindow("desktop-lyrics");
    if (!win) return;
    if (await win.isVisible()) {
      await win.hide();
    } else {
      await win.show();
    }
  }

  async function enterMiniMode() {
    const [mainWin, miniWin] = await Promise.all([
      getWindow("main"),
      getWindow("mini-player"),
    ]);
    // Don't hide desktop lyrics — keep it visible in mini mode
    if (mainWin) await mainWin.hide();
    if (miniWin) await miniWin.show();
  }

  async function exitMiniMode() {
    const [mainWin, miniWin] = await Promise.all([
      getWindow("main"),
      getWindow("mini-player"),
    ]);
    if (miniWin) await miniWin.hide();
    if (mainWin) {
      await mainWin.show();
      await mainWin.setFocus();
    }
  }

  async function showMainWindow() {
    const [mainWin, miniWin, lyricsWin] = await Promise.all([
      getWindow("main"),
      getWindow("mini-player"),
      getWindow("desktop-lyrics"),
    ]);
    if (miniWin) await miniWin.hide();
    if (lyricsWin) await lyricsWin.hide();
    if (mainWin) {
      await mainWin.show();
      await mainWin.setFocus();
    }
  }

  return {
    showDesktopLyrics,
    hideDesktopLyrics,
    toggleDesktopLyrics,
    enterMiniMode,
    exitMiniMode,
    showMainWindow,
  };
}
