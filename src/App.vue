<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from "vue";
import { storeToRefs } from "pinia";
import { darkTheme, zhCN, dateZhCN } from "naive-ui";
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
} from "naive-ui";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import TitleBar from "./components/TitleBar.vue";
import MiniPlayer from "./components/player/MiniPlayer.vue";
import FullPlayer from "./components/player/FullPlayer.vue";
import PlaylistPanel from "./components/player/PlaylistPanel.vue";
import DesktopLyrics from "./components/player/DesktopLyrics.vue";
import MiniPlayerWindow from "./components/player/MiniPlayerWindow.vue";
import { useSettingsStore } from "./stores/settings";
import { usePlayerStore } from "./stores/player";
import { useMediaSession } from "./composables/useMediaSession";
import { useThemeColor } from "./composables/useThemeColor";
import { useWindowGeometry } from "./composables/useWindowGeometry";
import { useCrossWindowSync } from "./composables/useCrossWindowSync";
import { useWindowManager } from "./composables/useWindowManager";

const settingsStore = useSettingsStore();
const playerStore = usePlayerStore();
const { init: initMediaSession, updateMetadata, updatePlaybackState } =
  useMediaSession();
const { themeOverrides } = useThemeColor(computed(() => settingsStore.accentColor));

const windowLabel = getCurrentWindow().label;

// Cross-window sync (main window only)
// Must use storeToRefs to keep refs reactive (accessing via reactive store unwraps them)
if (windowLabel === "main") {
  const { currentTime, lyrics, currentSong, isPlaying, duration, coverUrl } = storeToRefs(playerStore);
  useCrossWindowSync(
    currentTime,
    lyrics,
    currentSong,
    isPlaying,
    duration,
    coverUrl,
  );
}

const showFullPlayer = ref(false);
const showPlaylist = ref(false);

const theme = computed(() => {
  if (settingsStore.theme === "dark") return darkTheme;
  return null;
});

function toggleFullPlayer() {
  showFullPlayer.value = !showFullPlayer.value;
}

function togglePlaylist() {
  showPlaylist.value = !showPlaylist.value;
}

function closeFullPlayer() {
  showFullPlayer.value = false;
}

function closePlaylist() {
  showPlaylist.value = false;
}

let _unlisteners: Array<() => void> = [];

onMounted(async () => {
  await settingsStore.loadSettings();

  // Start window geometry tracking (main window only)
  if (windowLabel === "main") {
    const { startTracking } = useWindowGeometry();
    startTracking();

    // Restore desktop lyrics window if it was enabled
    if (settingsStore.desktopLyricsEnabled) {
      useWindowManager().showDesktopLyrics();
    }

    // Listen for mini-player control events
    const unlistenMiniTogglePlay = listen("mini-player:toggle-play", () => {
      playerStore.togglePlay();
    });
    const unlistenMiniNext = listen("mini-player:next", () => {
      playerStore.next();
    });
    const unlistenMiniPrev = listen("mini-player:prev", () => {
      playerStore.prev();
    });
    const unlistenMiniSeek = listen<number>("mini-player:seek", (e) => {
      playerStore.seekByPercent(e.payload);
    });
    const unlistenMiniRestore = listen("mini-player:restore-main", () => {
      useWindowManager().exitMiniMode();
    });

    _unlisteners.push(
      () => { unlistenMiniTogglePlay.then((fn) => fn()); },
      () => { unlistenMiniNext.then((fn) => fn()); },
      () => { unlistenMiniPrev.then((fn) => fn()); },
      () => { unlistenMiniSeek.then((fn) => fn()); },
      () => { unlistenMiniRestore.then((fn) => fn()); },
    );
  }

  // Listen for tray and global shortcut events
  const unlistenTrayPlayPause = listen("tray-play-pause", () => {
    playerStore.togglePlay();
  });
  const unlistenTrayNext = listen("tray-next", () => {
    playerStore.next();
  });
  const unlistenTrayPrev = listen("tray-prev", () => {
    playerStore.prev();
  });
  const unlistenGlobalPlayPause = listen("global-play-pause", () => {
    playerStore.togglePlay();
  });
  const unlistenGlobalNext = listen("global-next", () => {
    playerStore.next();
  });
  const unlistenGlobalPrev = listen("global-prev", () => {
    playerStore.prev();
  });

  _unlisteners.push(
    () => { unlistenTrayPlayPause.then((fn) => fn()); },
    () => { unlistenTrayNext.then((fn) => fn()); },
    () => { unlistenTrayPrev.then((fn) => fn()); },
    () => { unlistenGlobalPlayPause.then((fn) => fn()); },
    () => { unlistenGlobalNext.then((fn) => fn()); },
    () => { unlistenGlobalPrev.then((fn) => fn()); },
  );

  // Initialize media session with handlers
  initMediaSession(
    () => playerStore.togglePlay(),
    () => playerStore.next(),
    () => playerStore.prev(),
  );
});

onBeforeUnmount(() => {
  _unlisteners.forEach((fn) => fn());
  playerStore.cleanup();
});

// Sync media session metadata when song changes
watch(
  () => playerStore.currentSong,
  (song) => {
    updateMetadata(song);
  },
);

// Sync media session playback state
watch(
  () => playerStore.isPlaying,
  (playing) => {
    updatePlaybackState(playing);
  },
);
</script>

<template>
  <!-- Desktop Lyrics Window -->
  <DesktopLyrics v-if="windowLabel === 'desktop-lyrics'" />

  <!-- Mini Player Window -->
  <template v-else-if="windowLabel === 'mini-player'">
    <MiniPlayerWindow />
  </template>

  <!-- Main Window -->
  <template v-else>
    <NConfigProvider :theme="theme" :theme-overrides="themeOverrides" :locale="zhCN" :date-locale="dateZhCN">
      <NMessageProvider>
        <NDialogProvider>
          <NNotificationProvider>
            <div class="app-container">
              <TitleBar />
              <div class="app-content">
                <router-view />
              </div>
              <MiniPlayer
                @toggle-full="toggleFullPlayer"
                @toggle-playlist="togglePlaylist"
              />
              <Transition name="slide-up">
                <FullPlayer v-if="showFullPlayer" @close="closeFullPlayer" />
              </Transition>
              <Transition name="fade">
                <div v-if="showPlaylist" class="playlist-mask" @click="closePlaylist"></div>
              </Transition>
              <Transition name="slide-left">
                <PlaylistPanel
                  v-if="showPlaylist"
                  @close="closePlaylist"
                />
              </Transition>
            </div>
          </NNotificationProvider>
        </NDialogProvider>
      </NMessageProvider>
    </NConfigProvider>
  </template>
</template>

<style>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background: var(--app-bg);
  color: var(--app-text);
}

.app-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding-bottom: 72px;
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}
.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(100%);
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.25s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.playlist-mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  z-index: 149;
}

.slide-left-enter-active {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.slide-left-leave-active {
  transition: transform 0.25s cubic-bezier(0.4, 0, 1, 1);
}
.slide-left-enter-from,
.slide-left-leave-to {
  transform: translateX(100%);
}
</style>
