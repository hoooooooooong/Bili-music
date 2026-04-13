<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { darkTheme, zhCN, dateZhCN, type GlobalThemeOverrides } from "naive-ui";
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
} from "naive-ui";
import TitleBar from "./components/TitleBar.vue";
import MiniPlayer from "./components/player/MiniPlayer.vue";
import FullPlayer from "./components/player/FullPlayer.vue";
import PlaylistPanel from "./components/player/PlaylistPanel.vue";
import { useSettingsStore } from "./stores/settings";

const settingsStore = useSettingsStore();
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

onMounted(() => {
  settingsStore.loadSettings();
});
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

.slide-left-enter-active,
.slide-left-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}
.slide-left-enter-from,
.slide-left-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
