<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
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
} from "@vicons/ionicons5";
import { usePlayerStore } from "@/stores/player";
import { useFavoritesStore } from "@/stores/favorites";
import { useLyricOffsetsStore } from "@/stores/lyricOffsets";
import { usePlayerControls } from "@/composables/usePlayerControls";
import { useLyrics } from "@/composables/useLyrics";
import { formatDuration } from "@/utils/formatters";
import SpinningDisc from "./SpinningDisc.vue";
import ScrollingLyrics from "./ScrollingLyrics.vue";
import AudioVisualizer from "./AudioVisualizer.vue";

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

function handleKeydown(e: KeyboardEvent) {
  if (e.code === "Escape") emit("close");
}

onMounted(() => window.addEventListener("keydown", handleKeydown));
onUnmounted(() => window.removeEventListener("keydown", handleKeydown));
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

.fp-ctrl .loop-badge {
  position: absolute;
  top: 4px;
  right: 2px;
  font-size: 9px;
  font-weight: 700;
  line-height: 1;
  color: #fb7299;
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
  background-color: #fb7299 !important;
}
.full-player :deep(.n-slider-handle) {
  background-color: #fb7299 !important;
  border-color: #fb7299 !important;
}
.full-player :deep(.n-slider-dot) {
  background-color: #fb7299 !important;
}
</style>
