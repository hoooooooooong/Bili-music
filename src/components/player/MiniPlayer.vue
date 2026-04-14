<script setup lang="ts">
import { computed, ref } from "vue";
import { NIcon, NSlider } from "naive-ui";
import {
  PlayOutline,
  PauseOutline,
  PlaySkipForwardOutline,
  PlaySkipBackOutline,
  RepeatOutline,
  ShuffleOutline,
  VolumeHighOutline,
  VolumeMuteOutline,
  ListOutline,
  ExpandOutline,
  TimerOutline,
  ContractOutline,
} from "@vicons/ionicons5";
import { usePlayerStore } from "@/stores/player";
import { usePlayerControls } from "@/composables/usePlayerControls";
import { useLyrics } from "@/composables/useLyrics";
import { useWindowManager } from "@/composables/useWindowManager";
import { formatDuration } from "@/utils/formatters";

const emit = defineEmits<{ toggleFull: []; togglePlaylist: [] }>();
const player = usePlayerStore();
const { showVolume, showSleepTimer, sleepTimerDisplay, sleepTimerPresets, setSleepTimerAndClose, clearSleepTimerAndClose, toggleMute } = usePlayerControls();
const { currentLineIndex, lyrics } = useLyrics();

const currentLine = computed(() => {
  if (currentLineIndex.value < 0) return "";
  return lyrics.value[currentLineIndex.value]?.text ?? "";
});

const playModeIcon = computed(() => {
  switch (player.playMode) {
    case "random": return ShuffleOutline;
    case "loop": return RepeatOutline;
    default: return RepeatOutline;
  }
});

const playModeTitle = computed(() => {
  switch (player.playMode) {
    case "sequential": return "顺序播放";
    case "loop": return "单曲循环";
    case "random": return "随机播放";
  }
});

function onWheel(e: WheelEvent) {
  e.preventDefault();
  const delta = e.deltaY > 0 ? -0.05 : 0.05;
  player.setVolume(player.volume + delta);
}

function enterMiniMode() {
  useWindowManager().enterMiniMode();
}
</script>

<template>
  <div v-if="player.currentSong" class="mini-player" @wheel.passive="onWheel">
    <div v-if="player.playError" class="play-error">{{ player.playError }}</div>
    <div class="progress-bar-wrapper">
      <NSlider
        :value="player.progress"
        :step="0.1"
        :tooltip="false"
        :rail-style="{ backgroundColor: 'var(--border-color)' }"
        @update:value="(v: number) => player.seekByPercent(v)"
      />
    </div>

    <div class="mini-player-content">
      <div class="player-left" @click="emit('toggleFull')">
        <div class="mini-cover">
          <img
            :src="player.coverUrl || `bili-cover://${player.currentSong.bvid}`"
            :alt="player.currentSong.title"
          />
        </div>
        <div class="mini-info">
          <p class="mini-title">{{ player.currentSong.title }}</p>
          <div class="mini-lyric-wrapper">
            <Transition name="lyric-slide" mode="out-in">
              <p class="mini-lyric" :key="currentLineIndex">{{ currentLine }}</p>
            </Transition>
          </div>
        </div>
      </div>

      <div class="player-center">
        <button class="ctrl-btn small" @click="player.prev()">
          <NIcon size="18"><PlaySkipBackOutline /></NIcon>
        </button>
        <button class="ctrl-btn play-btn" @click="player.togglePlay()">
          <NIcon size="24">
            <PauseOutline v-if="player.isPlaying" />
            <PlayOutline v-else />
          </NIcon>
        </button>
        <button class="ctrl-btn small" @click="player.next()">
          <NIcon size="18"><PlaySkipForwardOutline /></NIcon>
        </button>
      </div>

      <div class="player-right">
        <div
          class="sleep-timer-wrapper"
          @mouseenter="showSleepTimer = true"
          @mouseleave="showSleepTimer = false"
        >
          <button class="ctrl-btn small">
            <NIcon
              size="16"
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
        <button
          class="ctrl-btn small"
          :title="playModeTitle"
          @click="player.togglePlayMode()"
        >
          <NIcon size="16"><component :is="playModeIcon" /></NIcon>
          <span v-if="player.playMode === 'loop'" class="loop-badge">1</span>
        </button>
        <div
          class="volume-wrapper"
          @mouseenter="showVolume = true"
          @mouseleave="showVolume = false"
        >
          <button
            class="ctrl-btn small"
            @click="toggleMute()"
          >
            <NIcon size="16">
              <VolumeMuteOutline v-if="player.volume === 0" />
              <VolumeHighOutline v-else />
            </NIcon>
          </button>
          <Transition name="fade">
            <div v-if="showVolume" class="volume-slider">
              <NSlider
                :value="player.volume * 100"
                :step="1"
                :min="0"
                :max="100"
                :tooltip="false"
                vertical
                :style="{ height: '80px' }"
                @update:value="(v: number) => player.setVolume(v / 100)"
              />
            </div>
          </Transition>
        </div>
        <button class="ctrl-btn small" @click="emit('togglePlaylist')">
          <NIcon size="16"><ListOutline /></NIcon>
        </button>
        <button class="ctrl-btn small" title="迷你模式" @click="enterMiniMode">
          <NIcon size="16"><ContractOutline /></NIcon>
        </button>
        <button class="ctrl-btn small" @click="emit('toggleFull')">
          <NIcon size="16"><ExpandOutline /></NIcon>
        </button>
      </div>
    </div>
  </div>

  <div v-else class="mini-player-empty">
    <p v-if="player.playError" class="error-text">{{ player.playError }}</p>
    <p v-else>选择一首歌开始播放</p>
  </div>
</template>

<style scoped>
.mini-player {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 72px;
  background: var(--player-bg);
  backdrop-filter: blur(20px);
  border-top: 1px solid var(--border-color);
  z-index: 100;
  display: flex;
  flex-direction: column;
}

.play-error {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  background: #e81123;
  color: white;
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 12px;
  white-space: nowrap;
  z-index: 10;
  max-width: 90%;
  white-space: normal;
  word-break: break-all;
}

.progress-bar-wrapper {
  padding: 0 16px;
  margin-top: -6px;
}

.mini-player-content {
  flex: 1;
  display: flex;
  align-items: center;
  padding: 0 12px;
  gap: 12px;
}

.player-left {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
  cursor: pointer;
}

.mini-cover {
  width: 44px;
  height: 44px;
  border-radius: 8px;
  overflow: hidden;
  flex-shrink: 0;
}

.mini-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.mini-info {
  min-width: 0;
  flex: 1;
}

.mini-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--app-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mini-lyric-wrapper {
  height: 16px;
  overflow: hidden;
  margin-top: 2px;
}

.mini-lyric {
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.lyric-slide-enter-active,
.lyric-slide-leave-active {
  transition: all 0.3s ease;
}
.lyric-slide-enter-from {
  opacity: 0;
  transform: translateY(100%);
}
.lyric-slide-leave-to {
  opacity: 0;
  transform: translateY(-100%);
}

.player-center {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ctrl-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--app-text);
  border-radius: 50%;
}

.ctrl-btn.small {
  width: 32px;
  height: 32px;
}

.ctrl-btn.small:hover {
  background: var(--card-hover);
}

.loop-badge {
  position: absolute;
  top: 2px;
  right: 0;
  font-size: 8px;
  font-weight: 700;
  line-height: 1;
  color: var(--accent-color);
}

.ctrl-btn.play-btn {
  width: 40px;
  height: 40px;
  background: var(--accent-color);
  color: white;
}

.ctrl-btn.play-btn:hover {
  transform: scale(1.05);
  background: var(--accent-hover);
}

.player-right {
  display: flex;
  align-items: center;
  gap: 2px;
}

.volume-wrapper {
  position: relative;
}

.volume-slider {
  position: absolute;
  bottom: 36px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 8px 4px;
  box-shadow: var(--shadow);
}

.sleep-timer-wrapper {
  position: relative;
}

.sleep-timer-count {
  position: absolute;
  bottom: -2px;
  font-size: 8px;
  line-height: 1;
  color: var(--accent-color);
  white-space: nowrap;
}

.sleep-timer-menu {
  position: absolute;
  bottom: 36px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 8px 0;
  min-width: 110px;
  box-shadow: var(--shadow);
}

.sleep-timer-title {
  font-size: 11px;
  color: var(--text-secondary);
  padding: 2px 14px 6px;
  margin: 0;
}

.sleep-timer-option {
  display: block;
  width: 100%;
  padding: 5px 14px;
  font-size: 12px;
  color: var(--app-text);
  background: none;
  border: none;
  cursor: pointer;
  text-align: left;
}

.sleep-timer-option:hover {
  background: var(--card-hover);
}

.sleep-timer-option.cancel {
  border-top: 1px solid var(--border-color);
  margin-top: 4px;
  padding-top: 7px;
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

.mini-player :deep(.n-slider-rail__fill) {
  background-color: var(--accent-color) !important;
}
.mini-player :deep(.n-slider-handle) {
  background-color: var(--accent-color) !important;
  border-color: var(--accent-color) !important;
}
.mini-player :deep(.n-slider-dot) {
  background-color: var(--accent-color) !important;
}
</style>
