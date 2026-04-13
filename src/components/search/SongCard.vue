<script setup lang="ts">
import { computed } from "vue";
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
import type { Song } from "@/types";
import { formatPlayCount } from "@/utils/formatters";

const props = defineProps<{ song: Song; songList?: Song[] }>();

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
</script>

<template>
  <div
    class="song-card"
    :class="{ playing: isCurrentSong }"
    @click="togglePlay"
  >
    <div class="card-cover">
      <img
        :src="song.coverUrl"
        :alt="song.title"
        loading="lazy"
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
      <span class="card-plays">{{ formatPlayCount(song.playCount) }} 播放</span>
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

.card-cover {
  position: relative;
  width: 52px;
  height: 52px;
  border-radius: 8px;
  overflow: hidden;
  flex-shrink: 0;
}

.card-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
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
