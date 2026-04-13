<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from "vue";
import { NIcon } from "naive-ui";
import {
  PlayOutline,
  HeartOutline,
  Heart,
  DownloadOutline,
  PlaySkipForwardOutline,
  ListOutline,
  CopyOutline,
} from "@vicons/ionicons5";
import { usePlayerStore } from "@/stores/player";
import { useFavoritesStore } from "@/stores/favorites";
import { useDownloadStore } from "@/stores/download";
import { usePlaylistStore } from "@/stores/playlists";
import type { Song } from "@/types";

const props = defineProps<{
  song: Song;
  songList?: Song[];
  x: number;
  y: number;
}>();

const emit = defineEmits<{ close: [] }>();

const player = usePlayerStore();
const favorites = useFavoritesStore();
const downloadStore = useDownloadStore();
const playlistStore = usePlaylistStore();

const isFav = computed(() => favorites.isFavorite(props.song.bvid));

const downloadTask = computed(() => {
  for (const task of downloadStore.tasks.values()) {
    if (task.bvid === props.song.bvid) return task;
  }
  return null;
});

const submenuVisible = ref(false);
const newPlaylistName = ref("");
const submenuInputRef = ref<HTMLInputElement | null>(null);

function play() {
  player.playSong(props.song, props.songList);
  emit("close");
}

function playNext() {
  if (player.currentSong && player.currentIndex >= 0) {
    player.playlist.splice(player.currentIndex + 1, 0, { ...props.song });
  } else {
    player.playlist.push({ ...props.song });
  }
  emit("close");
}

function toggleFavorite() {
  favorites.toggle(props.song);
  emit("close");
}

function download() {
  if (downloadTask.value) return;
  downloadStore.startDownload(props.song.bvid).catch((err: any) => {
    console.error("Download failed:", err);
  });
  emit("close");
}

function showSubmenu() {
  submenuVisible.value = true;
  nextTick(() => submenuInputRef.value?.focus());
}

function hideSubmenu() {
  submenuVisible.value = false;
}

function addToPlaylist(playlistId: string) {
  playlistStore.addSong(playlistId, props.song);
  emit("close");
}

function createAndAdd() {
  const name = newPlaylistName.value.trim();
  if (!name) return;
  const playlist = playlistStore.createPlaylist(name);
  playlistStore.addSong(playlist.id, props.song);
  newPlaylistName.value = "";
  emit("close");
}

async function copyBvid() {
  try {
    await navigator.clipboard.writeText(props.song.bvid);
  } catch {
    const ta = document.createElement("textarea");
    ta.value = props.song.bvid;
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    document.body.removeChild(ta);
  }
  emit("close");
}

function onOverlayClick() {
  emit("close");
}
</script>

<template>
  <Teleport to="body">
    <div class="context-menu-overlay" @mousedown="onOverlayClick" />
    <div
      class="context-menu"
      :style="{ left: x + 'px', top: y + 'px' }"
    >
      <button class="ctx-item" @click="play">
        <NIcon size="16"><PlayOutline /></NIcon>
        <span>播放</span>
      </button>
      <button class="ctx-item" @click="playNext">
        <NIcon size="16"><PlaySkipForwardOutline /></NIcon>
        <span>添加到下一首播放</span>
      </button>
      <div class="ctx-divider"></div>
      <button class="ctx-item" @click="toggleFavorite">
        <NIcon size="16">
          <Heart v-if="isFav" />
          <HeartOutline v-else />
        </NIcon>
        <span>{{ isFav ? '取消收藏' : '收藏' }}</span>
      </button>
      <button
        class="ctx-item"
        :disabled="!!downloadTask"
        @click="download"
      >
        <NIcon size="16"><DownloadOutline /></NIcon>
        <span>{{ downloadTask ? '下载中...' : '下载' }}</span>
      </button>
      <div
        class="ctx-item ctx-submenu-parent"
        @mouseenter="showSubmenu"
        @mouseleave="hideSubmenu"
      >
        <NIcon size="16"><ListOutline /></NIcon>
        <span>添加到歌单</span>
        <span class="ctx-arrow">&#9656;</span>
        <div
          v-if="submenuVisible"
          class="ctx-submenu"
          @mouseenter="showSubmenu"
          @mouseleave="hideSubmenu"
          @mousedown.stop
        >
          <div class="submenu-header">
            <input
              ref="submenuInputRef"
              v-model="newPlaylistName"
              placeholder="新建歌单，回车创建"
              class="submenu-input"
              @keydown.enter="createAndAdd"
              @click.stop
            />
          </div>
          <div
            v-if="playlistStore.playlists.length === 0"
            class="submenu-empty"
          >
            暂无歌单
          </div>
          <button
            v-for="p in playlistStore.playlists"
            :key="p.id"
            class="ctx-item submenu-item"
            @click="addToPlaylist(p.id)"
          >
            <span class="submenu-name">{{ p.name }}</span>
            <span class="submenu-count">{{ p.songs.length }} 首</span>
          </button>
        </div>
      </div>
      <div class="ctx-divider"></div>
      <button class="ctx-item" @click="copyBvid">
        <NIcon size="16"><CopyOutline /></NIcon>
        <span>复制 BVID</span>
      </button>
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu-overlay {
  position: fixed;
  inset: 0;
  z-index: 999;
}

.context-menu {
  position: fixed;
  z-index: 1000;
  min-width: 180px;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
  padding: 4px;
  overflow: visible;
}

.ctx-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 10px;
  border-radius: 6px;
  cursor: pointer;
  text-align: left;
  color: var(--app-text);
  font-size: 13px;
  border: none;
  background: none;
  position: relative;
}

.ctx-item:hover {
  background: var(--card-hover);
}

.ctx-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.ctx-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 6px;
}

.ctx-arrow {
  margin-left: auto;
  font-size: 10px;
  color: var(--text-tertiary);
}

.ctx-submenu-parent {
  position: relative;
  padding-right: 24px;
}

.ctx-submenu {
  position: absolute;
  left: 100%;
  top: -4px;
  min-width: 200px;
  max-width: 280px;
  max-height: 300px;
  overflow-y: auto;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
  padding: 4px;
}

.submenu-header {
  padding: 4px;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 2px;
}

.submenu-input {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--app-bg);
  color: var(--app-text);
  font-size: 12px;
  outline: none;
  box-sizing: border-box;
}

.submenu-input:focus {
  border-color: var(--accent-color);
}

.submenu-input::placeholder {
  color: var(--text-tertiary);
}

.submenu-empty {
  padding: 16px 12px;
  text-align: center;
  font-size: 12px;
  color: var(--text-tertiary);
}

.submenu-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.submenu-name {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.submenu-count {
  font-size: 11px;
  color: var(--text-tertiary);
  flex-shrink: 0;
  margin-left: 8px;
}
</style>
