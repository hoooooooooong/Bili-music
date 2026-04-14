<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Song, LyricLine } from "@/types";
import { findCurrentLine } from "@/utils/lrc-parser";
import { formatDuration } from "@/utils/formatters";
import { useSettingsStore } from "@/stores/settings";

const settingsStore = useSettingsStore();

const currentSong = ref<Song | null>(null);
const isPlaying = ref(false);
const currentTime = ref(0);
const duration = ref(0);
const coverUrl = ref("");
const lyrics = ref<LyricLine[]>([]);

const progress = computed(() =>
  duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0
);

const currentLineIndex = computed(() => findCurrentLine(lyrics.value, currentTime.value));
const currentLine = computed(() => {
  if (currentLineIndex.value < 0) return "";
  return lyrics.value[currentLineIndex.value]?.text ?? "";
});

let _pollTimer: ReturnType<typeof setInterval> | null = null;

async function pollState() {
  try {
    const state = await invoke<{
      currentTime: number;
      duration: number;
      isPlaying: boolean;
      currentSong: Song | null;
      coverUrl: string;
      lyrics: LyricLine[];
    }>("get_player_state");
    currentTime.value = state.currentTime;
    duration.value = state.duration;
    isPlaying.value = state.isPlaying;
    currentSong.value = state.currentSong;
    coverUrl.value = state.coverUrl;
    lyrics.value = state.lyrics;
  } catch {
    // ignore
  }
}

function startDrag() {
  getCurrentWindow().startDragging();
}

function emitToMain(event: string, payload?: any) {
  invoke("emit_to_main", { event, payload: payload ?? null }).catch(() => {});
}

function restoreMainWindow() {
  emitToMain("mini-player:restore-main");
}

function togglePlay() {
  emitToMain("mini-player:toggle-play");
}

function next() {
  emitToMain("mini-player:next");
}

function prev() {
  emitToMain("mini-player:prev");
}

function seekTo(e: MouseEvent) {
  const bar = (e.currentTarget as HTMLElement);
  const rect = bar.getBoundingClientRect();
  const percent = Math.max(0, Math.min(100, ((e.clientX - rect.left) / rect.width) * 100));
  emitToMain("mini-player:seek", percent);
}

onMounted(() => {
  settingsStore.loadSettings();
  pollState();
  _pollTimer = setInterval(pollState, 500);
});

onBeforeUnmount(() => {
  if (_pollTimer) clearInterval(_pollTimer);
});
</script>

<template>
  <div class="mini-window" @mousedown="startDrag">
    <div v-if="currentSong" class="mini-window-content">
      <!-- Cover -->
      <div class="mw-cover">
        <img
          v-if="coverUrl"
          :src="coverUrl"
          :alt="currentSong.title"
        />
      </div>

      <!-- Info -->
      <div class="mw-info">
        <p class="mw-title">{{ currentSong.title }}</p>
        <p class="mw-lyric">{{ currentLine }}</p>
        <!-- Progress bar -->
        <div class="mw-progress" @mousedown.stop="seekTo">
          <div class="mw-progress-fill" :style="{ width: progress + '%' }"></div>
        <div class="mw-progress-thumb" :style="{ left: progress + '%' }"></div>
        </div>
      </div>

      <!-- Controls -->
      <div class="mw-controls">
        <button class="mw-btn" @mousedown.stop @click="prev">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 6h2v12H6zm3.5 6 8.5 6V6z"/>
          </svg>
        </button>
        <button class="mw-btn mw-play" @mousedown.stop @click="togglePlay">
          <svg v-if="isPlaying" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
          </svg>
          <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M8 5v14l11-7z"/>
          </svg>
        </button>
        <button class="mw-btn" @mousedown.stop @click="next">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/>
          </svg>
        </button>
      </div>

      <!-- Restore button -->
      <button class="mw-btn mw-restore" @mousedown.stop @click="restoreMainWindow" title="恢复主窗口">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z"/>
        </svg>
      </button>
    </div>

    <div v-else class="mini-window-empty">
      <p>未在播放</p>
    </div>
  </div>
</template>

<style scoped>
.mini-window {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--player-bg, rgba(30, 30, 46, 0.95));
  backdrop-filter: blur(20px);
  overflow: hidden;
  user-select: none;
}

.mini-window-content {
  display: flex;
  align-items: center;
  width: 100%;
  height: 100%;
  padding: 0 10px;
  gap: 8px;
}

.mini-window-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  color: rgba(255, 255, 255, 0.4);
  font-size: 13px;
}

.mw-cover {
  width: 44px;
  height: 44px;
  border-radius: 8px;
  overflow: hidden;
  flex-shrink: 0;
}

.mw-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.mw-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.mw-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--app-text, #fff);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

.mw-lyric {
  font-size: 10px;
  color: var(--text-secondary, rgba(255, 255, 255, 0.45));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.mw-progress {
  width: 100%;
  height: 3px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  position: relative;
  cursor: pointer;
  margin-top: 1px;
}

.mw-progress-fill {
  height: 100%;
  border-radius: 2px;
  background: var(--accent-color, #6366f1);
  transition: width 0.3s linear;
}

.mw-progress-thumb {
  position: absolute;
  top: 50%;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent-color, #6366f1);
  transform: translate(-50%, -50%);
  opacity: 0;
  transition: left 0.3s linear, opacity 0.15s;
}

.mw-progress:hover .mw-progress-thumb {
  opacity: 1;
}

.mw-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.mw-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--app-text, rgba(255, 255, 255, 0.8));
  border-radius: 50%;
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
}

.mw-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.mw-play {
  width: 32px;
  height: 32px;
  background: var(--accent-color, #6366f1);
  color: white;
}

.mw-play:hover {
  transform: scale(1.05);
  background: var(--accent-hover, #4f46e5);
}

.mw-restore {
  flex-shrink: 0;
  margin-left: auto;
}
</style>
