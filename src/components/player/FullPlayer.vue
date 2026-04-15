<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from "vue";
import { NIcon, NSlider } from "naive-ui";
import {
  ChevronDownOutline,
  PlayOutline,
  PauseOutline,
  PlaySkipForwardOutline,
  PlaySkipBackOutline,
  RepeatOutline,
  ShuffleOutline,
  VolumeHighOutline,
  VolumeMuteOutline,
  HeartOutline,
  Heart,
  TimerOutline,
  AddOutline,
  RemoveOutline,
  ListOutline,
  CloseOutline,
  TrashOutline,
  ChatbubblesOutline,
  MusicalNotesOutline,
} from "@vicons/ionicons5";
import { usePlayerStore } from "@/stores/player";
import { useFavoritesStore } from "@/stores/favorites";
import { useLyricOffsetsStore } from "@/stores/lyricOffsets";
import { usePlayerControls } from "@/composables/usePlayerControls";
import { useLyrics } from "@/composables/useLyrics";
import { useDragSort } from "@/composables/useDragSort";
import { formatDuration } from "@/utils/formatters";
import { invoke } from "@tauri-apps/api/core";
import type { Song, Danmaku } from "@/types";
import SpinningDisc from "./SpinningDisc.vue";
import ScrollingLyrics from "./ScrollingLyrics.vue";
import AudioVisualizer from "./AudioVisualizer.vue";
import CommentPanel from "./CommentPanel.vue";
import DanmakuLayer from "./DanmakuLayer.vue";

const emit = defineEmits<{ close: [] }>();
const player = usePlayerStore();
const favorites = useFavoritesStore();
const { showVolume, showSleepTimer, sleepTimerDisplay, sleepTimerPresets, setSleepTimerAndClose, clearSleepTimerAndClose, toggleMute } = usePlayerControls();
const {
  currentLineIndex,
  lyrics,
  hasLyrics,
  currentOffset,
  onUserScroll,
  seekToLine,
} = useLyrics();

const lyricOffsets = useLyricOffsetsStore();

function adjustOffset(delta: number) {
  if (!player.currentSong) return;
  lyricOffsets.setOffset(
    player.currentSong.bvid,
    lyricOffsets.getOffset(player.currentSong.bvid) + delta
  );
}

function resetOffset() {
  if (!player.currentSong) return;
  lyricOffsets.clearOffset(player.currentSong.bvid);
}

function formatOffset(v: number): string {
  return (v > 0 ? "+" : "") + v.toFixed(1) + "s";
}

const isFav = computed(() =>
  player.currentSong
    ? favorites.isFavorite(player.currentSong.bvid)
    : false
);

const bgStyle = computed(() => {
  if (!player.currentSong) return {};
  const cover = player.currentSong.coverUrl || `bili-cover://${player.currentSong.bvid}`;
  return {
    backgroundImage: `url(${cover})`,
  };
});

const showPlaylist = ref(false);
const showComments = ref(false);
const danmakuEnabled = ref(localStorage.getItem("danmaku-enabled") === "true");
const nowPlaying = ref(0);
let nowPlayingTimer: ReturnType<typeof setInterval> | null = null;
const danmakuList = ref<Danmaku[]>([]);
const fpListRef = ref<HTMLElement | null>(null);

const { dragIndex: fpDragIndex, getItemStyle: fpGetItemStyle, onMouseDown: fpOnMouseDown, isDragging: fpIsDragging } = useDragSort({
  listRef: fpListRef,
  itemSelector: ".fp-playlist-item",
  ghostClass: "fp-playlist-item fp-drag-ghost",
  skipSelector: ".fp-item-remove",
  onDrop: (from, to) => player.movePlaylistItem(from, to),
});

function fpPlaySong(song: Song, index: number) {
  if (fpIsDragging()) return;
  player.currentIndex = index;
  player.playSong(song);
}

watch(showPlaylist, (v) => {
  if (v) showComments.value = false;
});

watch(showComments, (v) => {
  if (v) showPlaylist.value = false;
});

watch(danmakuEnabled, (v) => {
  localStorage.setItem("danmaku-enabled", v ? "true" : "false");
});

watch(
  () => player.currentSong?.bvid,
  async (bvid) => {
    console.log("[danmaku] bvid changed:", bvid, "enabled:", danmakuEnabled.value);
    danmakuList.value = [];
    if (!bvid) return;
    if (!danmakuEnabled.value) return;
    try {
      console.log("[danmaku] fetching for bvid:", bvid);
      const res = await invoke<{ danmaku: Danmaku[] }>("get_danmaku", { bvid });
      console.log("[danmaku] received:", res.danmaku.length, "items", res.danmaku.length > 0 ? res.danmaku.slice(0, 3) : "");
      danmakuList.value = res.danmaku;
    } catch (e) {
      console.error("[danmaku] fetch error:", e);
      danmakuList.value = [];
    }
  }
);

watch(danmakuEnabled, async (enabled) => {
  console.log("[danmaku] toggle:", enabled, "currentSong:", player.currentSong?.bvid);
  if (enabled && player.currentSong) {
    try {
      console.log("[danmaku] fetching for bvid:", player.currentSong.bvid);
      const res = await invoke<{ danmaku: Danmaku[] }>("get_danmaku", { bvid: player.currentSong.bvid });
      console.log("[danmaku] received:", res.danmaku.length, "items", res.danmaku.length > 0 ? res.danmaku.slice(0, 3) : "");
      danmakuList.value = res.danmaku;
    } catch (e) {
      console.error("[danmaku] fetch error:", e);
      danmakuList.value = [];
    }
  } else {
    danmakuList.value = [];
  }
});

watch(() => player.currentIndex, async (idx) => {
  if (idx < 0 || !fpListRef.value || !showPlaylist.value) return;
  await nextTick();
  const items = fpListRef.value!.querySelectorAll<HTMLElement>(".fp-playlist-item");
  items[idx]?.scrollIntoView({ behavior: "smooth", block: "nearest" });
});

watch(() => player.currentSong?.bvid, (bvid) => {
  nowPlaying.value = 0;
  fetchNowPlaying();
});

function handleKeydown(e: KeyboardEvent) {
  if (e.code === "Escape") emit("close");
}

async function fetchNowPlaying() {
  if (!player.currentSong) return;
  try {
    nowPlaying.value = await invoke<number>("get_now_playing", { bvid: player.currentSong.bvid });
  } catch {
    nowPlaying.value = 0;
  }
}

onMounted(async () => {
  window.addEventListener("keydown", handleKeydown);
  // Restore danmaku data if enabled on mount
  if (danmakuEnabled.value && player.currentSong) {
    try {
      const res = await invoke<{ danmaku: Danmaku[] }>("get_danmaku", { bvid: player.currentSong.bvid });
      danmakuList.value = res.danmaku;
    } catch {
      danmakuList.value = [];
    }
  }
  // Start now-playing refresh timer
  fetchNowPlaying();
  nowPlayingTimer = setInterval(fetchNowPlaying, 30000);
});
onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
  if (nowPlayingTimer) clearInterval(nowPlayingTimer);
});
</script>

<template>
  <div class="full-player" :style="bgStyle">
    <div class="fp-overlay"></div>
    <div class="fp-content">
      <div class="fp-header">
        <button class="fp-btn" @click="emit('close')">
          <NIcon size="22"><ChevronDownOutline /></NIcon>
        </button>
        <div class="fp-header-info">
          <p class="fp-title">{{ player.currentSong?.title }}</p>
          <p class="fp-author">{{ player.currentSong?.author }}</p>
          <p v-if="nowPlaying > 0" class="fp-now-playing">
            <NIcon size="13"><MusicalNotesOutline /></NIcon>
            {{ nowPlaying.toLocaleString() }} 人在听
          </p>
        </div>
        <button
          class="fp-btn"
          @click="
            player.currentSong && favorites.toggle(player.currentSong)
          "
        >
          <NIcon size="20" :color="isFav ? 'var(--accent-color)' : ''">
            <Heart v-if="isFav" />
            <HeartOutline v-else />
          </NIcon>
        </button>
      </div>

      <div class="fp-main">
        <DanmakuLayer
          v-if="danmakuEnabled"
          :danmaku-list="danmakuList"
          :current-time="player.currentTime"
          :duration="player.duration"
          :playing="player.isPlaying"
        />
        <div class="fp-left">
          <div class="fp-cover-area">
            <SpinningDisc />
            <AudioVisualizer />
          </div>
        </div>
        <div class="fp-right">
          <ScrollingLyrics
            :lyrics="lyrics"
            :current-index="currentLineIndex"
            @scroll="onUserScroll"
            @seek="seekToLine"
          />
        </div>

        <Transition name="fp-playlist-slide">
          <div v-if="showPlaylist" class="fp-playlist-panel">
          <div class="fp-playlist-header">
            <h3>播放列表 ({{ player.playlist.length }})</h3>
            <div class="fp-playlist-actions">
              <button
                class="fp-playlist-btn"
                @click="player.clearPlaylist()"
                title="清空"
              >
                <NIcon size="14"><TrashOutline /></NIcon>
              </button>
              <button class="fp-playlist-btn" @click="showPlaylist = false">
                <NIcon size="14"><CloseOutline /></NIcon>
              </button>
            </div>
          </div>
          <div class="fp-playlist-list" ref="fpListRef">
            <div v-if="player.playlist.length === 0" class="fp-playlist-empty">
              <p>播放列表为空</p>
            </div>
            <div
              v-for="(song, index) in player.playlist"
              :key="song.bvid"
              class="fp-playlist-item"
              :class="{ active: index === player.currentIndex, dragging: fpDragIndex === index }"
              :style="fpGetItemStyle(index)"
              @mousedown="fpOnMouseDown($event, index)"
              @click="fpPlaySong(song, index)"
            >
              <span v-if="index === player.currentIndex && player.isPlaying" class="fp-playing-indicator">
                <span class="bar"></span>
                <span class="bar"></span>
                <span class="bar"></span>
              </span>
              <span v-else class="fp-item-index">{{ index + 1 }}</span>
              <div class="fp-item-info">
                <p class="fp-item-title">{{ song.title }}</p>
                <p class="fp-item-author">{{ song.author }}</p>
              </div>
              <div class="fp-item-duration">{{ song.duration }}</div>
              <button class="fp-item-remove" @click.stop="player.removeFromPlaylist(index)">
                <NIcon size="12"><CloseOutline /></NIcon>
              </button>
            </div>
          </div>
        </div>
      </Transition>

      <Transition name="fp-playlist-slide">
        <CommentPanel v-if="showComments" @close="showComments = false" />
      </Transition>
      </div>

      <div class="fp-controls">
        <div class="fp-progress">
          <span class="time">{{ formatDuration(player.currentTime) }}</span>
          <NSlider
            :value="player.progress"
            :step="0.1"
            :tooltip="false"
            :rail-style="{ backgroundColor: 'rgba(255,255,255,0.2)' }"
            @update:value="(v: number) => player.seekByPercent(v)"
          />
          <span class="time">{{
            formatDuration(player.duration)
          }}</span>
        </div>

        <div v-if="hasLyrics" class="fp-buttons fp-offset-row">
          <span class="lyric-offset-label">歌词偏移</span>
          <button class="lyric-offset-btn" @click="adjustOffset(-0.5)">
            <NIcon size="14"><RemoveOutline /></NIcon>
          </button>
          <span class="lyric-offset-value">{{ formatOffset(currentOffset) }}</span>
          <button class="lyric-offset-btn" @click="adjustOffset(0.5)">
            <NIcon size="14"><AddOutline /></NIcon>
          </button>
          <button
            v-if="currentOffset !== 0"
            class="lyric-offset-reset"
            @click="resetOffset()"
          >重置</button>
        </div>

        <div class="fp-buttons">
          <div
            class="sleep-timer-area"
            @mouseenter="showSleepTimer = true"
            @mouseleave="showSleepTimer = false"
          >
            <button class="fp-ctrl">
              <NIcon
                size="20"
                :color="player.sleepTimerRemaining > 0 ? 'var(--accent-color)' : ''"
              >
                <TimerOutline />
              </NIcon>
              <span
                v-if="player.sleepTimerRemaining > 0"
                class="sleep-timer-count"
              >{{ sleepTimerDisplay }}</span>
            </button>
            <Transition name="fade">
              <div v-if="showSleepTimer" class="sleep-timer-menu">
                <p class="sleep-timer-title">定时关闭</p>
                <button
                  v-for="min in sleepTimerPresets"
                  :key="min"
                  class="sleep-timer-option"
                  @click="setSleepTimerAndClose(min)"
                >{{ min }}分钟</button>
                <button
                  v-if="player.sleepTimerRemaining > 0"
                  class="sleep-timer-option cancel"
                  @click="clearSleepTimerAndClose()"
                >取消定时</button>
              </div>
            </Transition>
          </div>
          <button class="fp-ctrl" @click="player.togglePlayMode()">
            <NIcon size="20">
              <ShuffleOutline v-if="player.playMode === 'random'" />
              <RepeatOutline v-else />
            </NIcon>
            <span v-if="player.playMode === 'loop'" class="loop-badge">1</span>
          </button>
          <button class="fp-ctrl" @click="player.prev()">
            <NIcon size="26"><PlaySkipBackOutline /></NIcon>
          </button>
          <button class="fp-play-btn" @click="player.togglePlay()">
            <NIcon size="32">
              <PauseOutline v-if="player.isPlaying" />
              <PlayOutline v-else />
            </NIcon>
          </button>
          <button class="fp-ctrl" @click="player.next()">
            <NIcon size="26"><PlaySkipForwardOutline /></NIcon>
          </button>
          <div
            class="volume-area"
            @mouseenter="showVolume = true"
            @mouseleave="showVolume = false"
          >
            <button
              class="fp-ctrl"
              @click="toggleMute()"
            >
              <NIcon size="20">
                <VolumeMuteOutline v-if="player.volume === 0" />
                <VolumeHighOutline v-else />
              </NIcon>
            </button>
            <Transition name="fade">
              <div v-if="showVolume" class="fp-volume-slider">
                <NSlider
                  :value="player.volume * 100"
                  :step="1"
                  :min="0"
                  :max="100"
                  :tooltip="false"
                  vertical
                  :style="{ height: '100px' }"
                  @update:value="
                    (v: number) => player.setVolume(v / 100)
                  "
                />
              </div>
            </Transition>
          </div>
          <button class="fp-ctrl" @click="showPlaylist = !showPlaylist" title="播放列表">
            <NIcon size="20" :color="showPlaylist ? 'var(--accent-color)' : ''">
              <ListOutline />
            </NIcon>
          </button>
          <button class="fp-ctrl" @click="showComments = !showComments" title="评论">
            <NIcon size="20" :color="showComments ? 'var(--accent-color)' : ''">
              <ChatbubblesOutline />
            </NIcon>
          </button>
          <button
            class="fp-ctrl danmaku-btn"
            :class="{ active: danmakuEnabled }"
            @click="danmakuEnabled = !danmakuEnabled"
            title="弹幕"
          >弹</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.full-player {
  position: fixed;
  inset: 0;
  z-index: 200;
  background-size: cover;
  background-position: center;
  background-color: #1a1a2e;
}

.fp-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(40px);
}

.fp-content {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  height: 100vh;
  padding: 16px 24px;
  color: white;
}

.fp-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  flex-shrink: 0;
}

.fp-header-info {
  text-align: center;
  min-width: 0;
  flex: 1;
}

.fp-title {
  font-size: 16px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.fp-author {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.6);
  margin-top: 2px;
}

.fp-now-playing {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.45);
  margin-top: 4px;
}

.fp-now-playing .n-icon {
  color: rgba(255, 255, 255, 0.55);
}

.fp-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.8);
  border-radius: 50%;
}

.fp-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.fp-main {
  position: relative;
  flex: 1;
  display: flex;
  align-items: center;
  gap: 40px;
  padding: 20px 0;
  padding-left: 40px;
  min-height: 0;
}

.fp-left {
  flex-shrink: 0;
}

.fp-cover-area {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.fp-cover-area :deep(.audio-visualizer) {
  position: absolute;
  bottom: -120px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 2;
}

.fp-right {
  flex: 1;
  min-width: 0;
  height: 100%;
}

.fp-offset-row {
  gap: 6px;
  margin-top: 6px;
  justify-content: center;
}

.lyric-offset-label {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.lyric-offset-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.7);
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.08);
}

.lyric-offset-btn:hover {
  background: rgba(255, 255, 255, 0.18);
}

.lyric-offset-value {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.8);
  min-width: 42px;
  text-align: center;
  font-variant-numeric: tabular-nums;
}

.lyric-offset-reset {
  font-size: 12px;
  color: var(--accent-color);
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
}

.lyric-offset-reset:hover {
  background: rgba(255, 255, 255, 0.08);
}

.fp-controls {
  flex-shrink: 0;
  padding: 16px 0 24px;
}

.fp-progress {
  display: flex;
  align-items: center;
  gap: 12px;
}

.fp-progress .time {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  min-width: 40px;
  text-align: center;
}

.fp-buttons {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 24px;
  margin-top: 12px;
}

.fp-ctrl {
  position: relative;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.8);
  border-radius: 50%;
}

.fp-ctrl:hover {
  background: rgba(255, 255, 255, 0.1);
}

.danmaku-btn {
  font-size: 13px;
  font-weight: 700;
  color: rgba(255, 255, 255, 0.6);
  letter-spacing: 0;
}

.danmaku-btn.active {
  color: var(--accent-color);
}

.fp-ctrl .loop-badge {
  position: absolute;
  top: 4px;
  right: 2px;
  font-size: 9px;
  font-weight: 700;
  line-height: 1;
  color: var(--accent-color);
}

.fp-play-btn {
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-color);
  color: white;
  border-radius: 50%;
}

.fp-play-btn:hover {
  transform: scale(1.05);
}

.volume-area {
  position: relative;
}

.fp-volume-slider {
  position: absolute;
  bottom: 44px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 8px 4px;
}

.sleep-timer-area {
  position: relative;
}

.sleep-timer-count {
  position: absolute;
  bottom: -2px;
  font-size: 9px;
  line-height: 1;
  color: var(--accent-color);
  white-space: nowrap;
}

.sleep-timer-menu {
  position: absolute;
  bottom: 44px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(255, 255, 255, 0.12);
  backdrop-filter: blur(20px);
  border-radius: 10px;
  padding: 8px 0;
  min-width: 120px;
}

.sleep-timer-title {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  padding: 2px 16px 6px;
  margin: 0;
}

.sleep-timer-option {
  display: block;
  width: 100%;
  padding: 6px 16px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
  background: none;
  border: none;
  cursor: pointer;
  text-align: left;
}

.sleep-timer-option:hover {
  background: rgba(255, 255, 255, 0.1);
}

.sleep-timer-option.cancel {
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  margin-top: 4px;
  padding-top: 8px;
  color: var(--accent-color);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.full-player :deep(.n-slider-rail__fill) {
  background-color: var(--accent-color) !important;
}
.full-player :deep(.n-slider-handle) {
  background-color: var(--accent-color) !important;
  border-color: var(--accent-color) !important;
}
.full-player :deep(.n-slider-dot) {
  background-color: var(--accent-color) !important;
}

/* Full player playlist panel */
.fp-playlist-panel {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 320px;
  z-index: 10;
  background: rgba(20, 20, 30, 0.95);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: -4px 0 24px rgba(0, 0, 0, 0.4);
}

.fp-playlist-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  flex-shrink: 0;
}

.fp-playlist-header h3 {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
}

.fp-playlist-actions {
  display: flex;
  gap: 4px;
}

.fp-playlist-btn {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.5);
  border-radius: 6px;
}

.fp-playlist-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
}

.fp-playlist-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.fp-playlist-list::-webkit-scrollbar {
  width: 4px;
}

.fp-playlist-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 2px;
}

.fp-playlist-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 120px;
  color: rgba(255, 255, 255, 0.3);
  font-size: 13px;
}

.fp-playlist-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  cursor: grab;
  user-select: none;
}

.fp-playlist-item:hover {
  background: rgba(255, 255, 255, 0.08);
}

.fp-playlist-item.active {
  background: rgba(255, 255, 255, 0.12);
}

.fp-playlist-item.dragging {
  visibility: hidden;
}

.fp-drag-ghost {
  opacity: 0.85;
  background: rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  cursor: grabbing;
}

.fp-item-index {
  width: 22px;
  text-align: center;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.3);
  flex-shrink: 0;
}

.fp-playlist-item.active .fp-item-index {
  color: var(--accent-color);
}

.fp-item-info {
  flex: 1;
  min-width: 0;
}

.fp-item-title {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.85);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.fp-playlist-item.active .fp-item-title {
  color: var(--accent-color);
  font-weight: 500;
}

.fp-item-author {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.fp-item-duration {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.3);
  flex-shrink: 0;
}

.fp-item-remove {
  opacity: 0;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  flex-shrink: 0;
}

.fp-playlist-item:hover .fp-item-remove {
  opacity: 1;
}

.fp-item-remove:hover {
  color: var(--accent-color);
}

.fp-playing-indicator {
  display: inline-flex;
  align-items: flex-end;
  gap: 1.5px;
  height: 12px;
  width: 22px;
  justify-content: center;
  flex-shrink: 0;
}

.fp-playing-indicator .bar {
  width: 2.5px;
  border-radius: 1px;
  background: var(--accent-color);
  animation: fp-equalizer 0.8s ease-in-out infinite;
}

.fp-playing-indicator .bar:nth-child(1) { animation-delay: 0s; }
.fp-playing-indicator .bar:nth-child(2) { animation-delay: 0.2s; }
.fp-playing-indicator .bar:nth-child(3) { animation-delay: 0.4s; }

@keyframes fp-equalizer {
  0%, 100% { height: 3px; }
  50% { height: 12px; }
}

.fp-playlist-slide-enter-active,
.fp-playlist-slide-leave-active {
  transition: transform 0.25s ease, opacity 0.25s ease;
}

.fp-playlist-slide-enter-from,
.fp-playlist-slide-leave-to {
  transform: translateX(40px);
  opacity: 0;
}
</style>
