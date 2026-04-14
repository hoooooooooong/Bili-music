<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { LyricLine } from "@/types";
import { findCurrentLine } from "@/utils/lrc-parser";
import { useSettingsStore } from "@/stores/settings";

const settingsStore = useSettingsStore();

const lyrics = ref<LyricLine[]>([]);
const currentTime = ref(0);
const locked = ref(false);
const currentRef = ref<HTMLElement | null>(null);
const nextRef = ref<HTMLElement | null>(null);
const isCurrentOverflow = ref(false);
const isNextOverflow = ref(false);

function checkOverflow() {
  if (currentRef.value) {
    isCurrentOverflow.value = currentRef.value.scrollWidth > currentRef.value.clientWidth + 1;
  }
  if (nextRef.value) {
    isNextOverflow.value = nextRef.value.scrollWidth > nextRef.value.clientWidth + 1;
  }
}

let _pollTimer: ReturnType<typeof setInterval> | null = null;
let _settingsTimer: ReturnType<typeof setInterval> | null = null;

const currentLineIndex = computed(() => findCurrentLine(lyrics.value, currentTime.value));
const currentLine = computed(() => {
  if (currentLineIndex.value < 0) return "";
  return lyrics.value[currentLineIndex.value]?.text ?? "";
});
const nextLine = computed(() => {
  const idx = currentLineIndex.value + 1;
  if (idx >= lyrics.value.length) return "";
  return lyrics.value[idx]?.text ?? "";
});

watch([currentLine, nextLine], () => {
  nextTick(checkOverflow);
});

function startDrag() {
  if (locked.value) return;
  getCurrentWindow().startDragging();
}

function onDoubleClick() {
  locked.value = !locked.value;
}

async function pollState() {
  try {
    const state = await invoke<{
      currentTime: number;
      duration: number;
      isPlaying: boolean;
      currentSong: any;
      coverUrl: string;
      lyrics: LyricLine[];
    }>("get_player_state");
    currentTime.value = state.currentTime;
    lyrics.value = state.lyrics;
  } catch {
    // ignore
  }
}

onMounted(() => {
  settingsStore.loadSettings();
  applySettings();
  pollState();
  _pollTimer = setInterval(() => {
    pollState();
    // Refresh settings every 2 seconds to pick up changes from main window
  }, 500);
  _settingsTimer = setInterval(() => {
    settingsStore.loadSettings();
    applySettings();
  }, 2000);
});

function applySettings() {
  locked.value = settingsStore.desktopLyricsLocked;
  const root = document.documentElement;
  root.style.setProperty("--accent-color", settingsStore.accentColor);
}

onBeforeUnmount(() => {
  if (_pollTimer) clearInterval(_pollTimer);
  if (_settingsTimer) clearInterval(_settingsTimer);
});
</script>

<template>
  <div
    class="desktop-lyrics"
    :class="{ locked }"
    @mousedown="startDrag"
    @dblclick="onDoubleClick"
  >
    <div class="lyrics-container">
      <div class="lyric-scroll current" :class="{ scrolling: isCurrentOverflow }">
        <span class="lyric-line current" ref="currentRef" :style="{ fontSize: settingsStore.desktopLyricsFontSize + 'px' }">{{ currentLine }}</span>
      </div>
      <div class="lyric-scroll next" :class="{ scrolling: isNextOverflow }">
        <span class="lyric-line next" ref="nextRef" :style="{ fontSize: (settingsStore.desktopLyricsFontSize - 6) + 'px' }">{{ nextLine }}</span>
      </div>
    </div>
    <div v-if="locked" class="lock-indicator">&#128274;</div>
  </div>
</template>

<style scoped>
.desktop-lyrics {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: move;
  user-select: none;
  position: relative;
}

.desktop-lyrics.locked {
  cursor: default;
}

.lyrics-container {
  text-align: center;
  pointer-events: none;
  max-width: 95%;
}

.lyric-scroll {
  overflow: hidden;
  white-space: nowrap;
}

.lyric-scroll.scrolling .lyric-line.current,
.lyric-scroll.scrolling .lyric-line.next {
  display: inline-block;
  animation: marquee 8s linear infinite;
}

.lyric-line.current {
  font-weight: 600;
  color: var(--accent-color, #fb7299);
  text-shadow: 0 1px 4px rgba(0, 0, 0, 0.8), 0 0 8px rgba(0, 0, 0, 0.5);
}

.lyric-line.next {
  color: rgba(255, 255, 255, 0.55);
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.6);
  margin-top: 4px;
}

@keyframes marquee {
  0%, 20% { transform: translateX(0); }
  80%, 100% { transform: translateX(-30%); }
}

.lock-indicator {
  position: absolute;
  top: 4px;
  right: 8px;
  font-size: 12px;
  opacity: 0.5;
}
</style>
