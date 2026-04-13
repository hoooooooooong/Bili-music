<script setup lang="ts">
import { ref, computed } from "vue";
import { NIcon, NSpin } from "naive-ui";
import {
  PlayOutline,
  PauseOutline,
  HeartOutline,
  Heart,
  DownloadOutline,
  CheckmarkCircleOutline,
} from "@vicons/ionicons5";
import { usePlayerStore } from "@/stores/player";
import { useFavoritesStore } from "@/stores/favorites";
import { useDownloadStore } from "@/stores/download";
import AddToPlaylistDropdown from "../playlists/AddToPlaylistDropdown.vue";
import SongContextMenu from "../SongContextMenu.vue";
import type { Song } from "@/types";
import { formatPlayCount } from "@/utils/formatters";

const props = defineProps<{
  song: Song;
  songList?: Song[];
  selectMode?: boolean;
  selected?: boolean;
}>();

const emit = defineEmits<{
  "toggle-select": [bvid: string];
}>();

const player = usePlayerStore();
const favorites = useFavoritesStore();
const downloadStore = useDownloadStore();

const isPlaying = computed(
  () => player.currentSong?.bvid === props.song.bvid && player.isPlaying
);

const isCurrentSong = computed(
  () => player.currentSong?.bvid === props.song.bvid
);

const isFav = computed(() => favorites.isFavorite(props.song.bvid));

const downloadTask = computed(() => {
  for (const task of downloadStore.tasks.values()) {
    if (task.bvid === props.song.bvid) return task;
  }
  return null;
});

const contextMenu = ref<{ x: number; y: number } | null>(null);

function togglePlay() {
  if (isCurrentSong.value) {
    player.togglePlay();
  } else {
    player.playSong(props.song, props.songList);
  }
}

function toggleFavorite(e: Event) {
  e.stopPropagation();
  favorites.toggle(props.song);
}

async function startDownload(e: Event) {
  e.stopPropagation();
  if (downloadTask.value) return;
  try {
    await downloadStore.startDownload(props.song.bvid);
  } catch (err: any) {
    console.error("Download failed:", err);
  }
}

function showContextMenu(e: MouseEvent) {
  e.preventDefault();
  e.stopPropagation();
  contextMenu.value = { x: e.clientX, y: e.clientY };
}
</script>

<template>
  <div
    class="song-card"
    :class="{ playing: isCurrentSong, selected: selectMode && selected }"
    @click="selectMode ? emit('toggle-select', song.bvid) : togglePlay()"
    @contextmenu.prevent="showContextMenu"
  >
    <div v-if="selectMode" class="select-checkbox" @click.stop="emit('toggle-select', song.bvid)">
      <div class="checkbox-inner" :class="{ checked: selected }">
        <svg v-if="selected" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
      </div>
    </div>
    <div class="card-cover">
      <img
        :src="song.coverUrl"
        :alt="song.title"
        loading="lazy"
        @load="($event.target as HTMLImageElement).classList.add('loaded'); ($event.target as HTMLImageElement).parentElement!.classList.add('loaded')"
        @error="($event.target as HTMLImageElement).src = `bili-cover://${song.bvid}`"
      />
      <div class="cover-overlay">
        <div class="play-btn" :class="{ active: isPlaying }">
          <NIcon size="28">
            <PauseOutline v-if="isPlaying" />
            <PlayOutline v-else />
          </NIcon>
        </div>
      </div>
      <span class="duration">{{ song.duration }}</span>
    </div>
    <div class="card-info">
      <h3 class="card-title">{{ song.title }}</h3>
      <p class="card-author">{{ song.author }}</p>
      <span class="card-plays">
        <span v-if="isCurrentSong" class="playing-indicator">
          <span class="bar" :class="{ paused: !isPlaying }"></span>
          <span class="bar" :class="{ paused: !isPlaying }"></span>
          <span class="bar" :class="{ paused: !isPlaying }"></span>
        </span>
        {{ formatPlayCount(song.playCount) }} 播放
      </span>
    </div>
    <div class="card-actions" @click.stop>
      <button
        class="action-btn"
        :class="{ fav: isFav }"
        @click="toggleFavorite"
        :title="isFav ? '取消收藏' : '收藏'"
      >
        <NIcon size="18">
          <Heart v-if="isFav" />
          <HeartOutline v-else />
        </NIcon>
      </button>
      <AddToPlaylistDropdown :song="song" />
      <button
        class="action-btn"
        :disabled="!!downloadTask"
        @click="startDownload"
        :title="downloadTask ? '下载中' : '下载'"
      >
        <NIcon size="18">
          <CheckmarkCircleOutline
            v-if="downloadTask?.status === 'done'"
          />
          <NSpin
            v-else-if="
              downloadTask?.status === 'downloading' ||
              downloadTask?.status === 'converting'
            "
            :size="18"
          />
          <DownloadOutline v-else />
        </NIcon>
      </button>
    </div>

    <SongContextMenu
      v-if="contextMenu"
      :song="song"
      :song-list="songList"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @close="contextMenu = null"
    />
  </div>
</template>

<style scoped>
.song-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: var(--card-bg);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}

.song-card:hover {
  background: var(--card-hover);
  border-color: var(--border-color);
}

.song-card.playing {
  border-color: var(--accent-color);
  background: var(--accent-light);
}

.song-card.selected {
  border-color: var(--accent-color);
  background: var(--accent-light);
}

.select-checkbox {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  cursor: pointer;
}

.checkbox-inner {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
  color: white;
}

.checkbox-inner.checked {
  background: var(--accent-color);
  border-color: var(--accent-color);
}

.card-cover {
  position: relative;
  width: 52px;
  height: 52px;
  border-radius: 8px;
  overflow: hidden;
  flex-shrink: 0;
  background: var(--skeleton-bg, var(--card-hover));
  animation: skeleton-pulse 1.5s ease-in-out infinite;
}

.card-cover.loaded {
  animation: none;
}

.card-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0;
  transition: opacity 0.3s;
}

.card-cover img.loaded {
  opacity: 1;
}

@keyframes skeleton-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.cover-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.15s;
}

.song-card:hover .cover-overlay,
.song-card.playing .cover-overlay {
  opacity: 1;
}

.play-btn {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--accent-color);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.duration {
  position: absolute;
  bottom: 4px;
  right: 4px;
  padding: 1px 4px;
  background: rgba(0, 0, 0, 0.7);
  border-radius: 3px;
  font-size: 10px;
  color: white;
}

.card-info {
  flex: 1;
  min-width: 0;
}

.card-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--app-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 2px;
}

.card-author {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 2px;
}

.card-plays {
  font-size: 11px;
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  gap: 5px;
}

.playing-indicator {
  display: inline-flex;
  align-items: flex-end;
  gap: 1.5px;
  height: 12px;
}

.playing-indicator .bar {
  width: 2.5px;
  border-radius: 1px;
  background: var(--accent-color);
  animation: equalizer 0.8s ease-in-out infinite;
}

.playing-indicator .bar:nth-child(1) { animation-delay: 0s; }
.playing-indicator .bar:nth-child(2) { animation-delay: 0.2s; }
.playing-indicator .bar:nth-child(3) { animation-delay: 0.4s; }

.playing-indicator .bar.paused {
  animation-play-state: paused;
}

@keyframes equalizer {
  0%, 100% { height: 3px; }
  50% { height: 12px; }
}

.card-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.action-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--text-secondary);
  transition: all 0.15s;
}

.action-btn:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.action-btn.fav {
  color: var(--accent-color);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
