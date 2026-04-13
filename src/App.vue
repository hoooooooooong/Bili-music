<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from "vue";
import { darkTheme, zhCN, dateZhCN, type GlobalThemeOverrides } from "naive-ui";
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
} from "naive-ui";
import { listen } from "@tauri-apps/api/event";
import TitleBar from "./components/TitleBar.vue";
import MiniPlayer from "./components/player/MiniPlayer.vue";
import FullPlayer from "./components/player/FullPlayer.vue";
import PlaylistPanel from "./components/player/PlaylistPanel.vue";
import { useSettingsStore } from "./stores/settings";
import { usePlayerStore } from "./stores/player";
import { useMediaSession } from "./composables/useMediaSession";

const settingsStore = useSettingsStore();
const playerStore = usePlayerStore();
const { init: initMediaSession, updateMetadata, updatePlaybackState } =
  useMediaSession();

const showFullPlayer = ref(false);
const showPlaylist = ref(false);

const theme = computed(() => {
  if (settingsStore.theme === "dark") return darkTheme;
  return null;
});

const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: "#fb7299",
    primaryColorHover: "#fc85a5",
    primaryColorPressed: "#e5617f",
  },
};

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

onMounted(() => {
  settingsStore.loadSettings();

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

  _unlisteners = [
    () => { unlistenTrayPlayPause.then((fn) => fn()); },
    () => { unlistenTrayNext.then((fn) => fn()); },
    () => { unlistenTrayPrev.then((fn) => fn()); },
    () => { unlistenGlobalPlayPause.then((fn) => fn()); },
    () => { unlistenGlobalNext.then((fn) => fn()); },
    () => { unlistenGlobalPrev.then((fn) => fn()); },
  ];

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
