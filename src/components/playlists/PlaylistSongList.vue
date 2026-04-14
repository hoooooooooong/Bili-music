<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { NIcon } from "naive-ui";
import { useMessage } from "naive-ui";
import {
  ArrowBackOutline, PlayOutline, TrashOutline, DownloadOutline,
  ImageOutline, RemoveCircleOutline, MusicalNoteOutline, CloudDownloadOutline,
  SwapVerticalOutline
} from "@vicons/ionicons5";
import { save, open } from "@tauri-apps/plugin-dialog";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";
import SongCard from "../search/SongCard.vue";
import { usePlaylistStore } from "@/stores/playlists";
import { usePlayerStore } from "@/stores/player";
import { useDownloadStore } from "@/stores/download";
import { useDragSort } from "@/composables/useDragSort";
import type { Playlist } from "@/types";

const props = defineProps<{
  playlist: Playlist;
  readonly?: boolean;
}>();
const emit = defineEmits<{
  back: [];
  deleted: [];
}>();

const playlistStore = usePlaylistStore();
const player = usePlayerStore();
const downloadStore = useDownloadStore();
const message = useMessage();

type SortOrder = "desc" | "asc";
const sortOrder = ref<SortOrder>("desc");

const sortedSongs = computed(() => {
  const songs = [...props.playlist.songs];
  if (sortOrder.value === "desc") {
    songs.reverse();
  }
  return songs;
});

function toggleSortOrder() {
  sortOrder.value = sortOrder.value === "desc" ? "asc" : "desc";
}

function playAll() {
  if (sortedSongs.value.length > 0) {
    player.playSong(sortedSongs.value[0], sortedSongs.value);
  }
}

function deletePlaylist() {
  playlistStore.deletePlaylist(props.playlist.id);
  emit("deleted");
}

function downloadAll() {
  if (props.playlist.songs.length === 0) return;
  downloadStore.batchDownload(props.playlist.songs);
  message.success(`已添加 ${props.playlist.songs.length} 首到下载队列`);
}

async function exportPlaylist() {
  try {
    let json: string;
    if (props.readonly) {
      const data = {
        version: 1,
        exportedAt: new Date().toISOString(),
        playlists: [{
          name: props.playlist.name,
          songs: props.playlist.songs.map((s) => ({
            bvid: s.bvid, title: s.title, author: s.author,
            duration: s.duration, playCount: s.playCount,
            playCountText: s.playCountText, coverUrl: s.coverUrl,
          })),
        }],
      };
      json = JSON.stringify(data, null, 2);
    } else {
      json = playlistStore.exportPlaylist(props.playlist.id);
    }
    const filePath = await save({
      defaultPath: `${props.playlist.name}.json`,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (filePath) {
      await writeFile(filePath, new TextEncoder().encode(json));
      message.success("歌单已导出");
    }
  } catch (e: any) {
    message.error(e.message || "导出失败");
  }
}

const showCoverMenu = ref(false);
const coverMenuRef = ref<HTMLElement | null>(null);

function toggleCoverMenu() {
  showCoverMenu.value = !showCoverMenu.value;
}

function closeCoverMenu() {
  showCoverMenu.value = false;
}

async function useFirstSongCover() {
  if (props.playlist.songs.length > 0) {
    playlistStore.setPlaylistCover(props.playlist.id, props.playlist.songs[0].coverUrl);
    message.success("封面已设置为第一首歌曲封面");
  } else {
    message.warning("歌单中没有歌曲");
  }
  closeCoverMenu();
}

async function selectImageFile() {
  try {
    const filePath = await open({
      multiple: false,
      filters: [{ name: "图片", extensions: ["png", "jpg", "jpeg", "webp", "gif"] }],
    });
    if (!filePath) return;
    const data = await readFile(filePath as string);
    const ext = (filePath as string).split(".").pop()?.toLowerCase() || "png";
    const mimeMap: Record<string, string> = { png: "image/png", jpg: "image/jpeg", jpeg: "image/jpeg", webp: "image/webp", gif: "image/gif" };
    const mime = mimeMap[ext] || "image/png";
    const binaryStr = Array.from(new Uint8Array(data)).map((b) => String.fromCharCode(b)).join("");
    const base64 = btoa(binaryStr);
    const dataUrl = `data:${mime};base64,${base64}`;
    playlistStore.setPlaylistCover(props.playlist.id, dataUrl);
    message.success("封面已更新");
  } catch (e: any) {
    message.error(e.message || "设置封面失败");
  }
  closeCoverMenu();
}

function removeCover() {
  playlistStore.clearPlaylistCover(props.playlist.id);
  message.success("封面已移除");
  closeCoverMenu();
}

function onClickOutside(e: MouseEvent) {
  if (coverMenuRef.value && !coverMenuRef.value.contains(e.target as Node)) {
    closeCoverMenu();
  }
}

onMounted(() => {
  document.addEventListener("mousedown", onClickOutside);
});

onBeforeUnmount(() => {
  document.removeEventListener("mousedown", onClickOutside);
});

const listRef = ref<HTMLElement | null>(null);

function toStoreIndex(visualIndex: number) {
  return props.playlist.songs.length - 1 - visualIndex;
}

const { dragIndex, getItemStyle, onMouseDown } = useDragSort({
  listRef,
  itemSelector: ".fav-item",
  ghostClass: "fav-drag-ghost",
  skipSelector: ".card-actions",
  gap: 8,
  onDrop: (from, to) => playlistStore.moveSong(props.playlist.id, toStoreIndex(from), toStoreIndex(to)),
});
</script>

<template>
  <div class="playlist-song-list">
    <div class="list-header">
      <button class="back-btn" @click="emit('back')">
        <NIcon size="18"><ArrowBackOutline /></NIcon>
      </button>
      <div class="header-info">
        <h3 class="header-name">
          {{ playlist.name }}
          <button v-if="!readonly" class="cover-edit-btn" @click="toggleCoverMenu" title="设置封面">
            <NIcon size="14"><ImageOutline /></NIcon>
          </button>
        </h3>
        <span class="header-count">{{ playlist.songs.length }} 首</span>
      </div>
      <div class="header-actions">
        <button
          class="header-btn"
          :disabled="sortedSongs.length === 0"
          @click="toggleSortOrder"
          :title="sortOrder === 'desc' ? '按时间降序（最新在前）' : '按时间升序（最早在前）'"
        >
          <NIcon size="18"><SwapVerticalOutline /></NIcon>
        </button>
        <button
          class="header-btn"
          :disabled="sortedSongs.length === 0"
          @click="playAll"
          title="全部播放"
        >
          <NIcon size="18"><PlayOutline /></NIcon>
        </button>
        <button
          class="header-btn"
          :disabled="sortedSongs.length === 0"
          @click="exportPlaylist"
          title="导出歌单"
        >
          <NIcon size="18"><CloudDownloadOutline /></NIcon>
        </button>
        <button
          class="header-btn"
          :disabled="sortedSongs.length === 0"
          @click="downloadAll"
          title="一键下载"
        >
          <NIcon size="18"><DownloadOutline /></NIcon>
        </button>
        <button v-if="!readonly" class="header-btn danger" @click="deletePlaylist" title="删除歌单">
          <NIcon size="18"><TrashOutline /></NIcon>
        </button>
      </div>

      <!-- Cover menu dropdown -->
      <div v-if="showCoverMenu && !readonly" class="cover-menu" ref="coverMenuRef">
        <button class="cover-menu-item" @click="useFirstSongCover">
          <NIcon size="16"><MusicalNoteOutline /></NIcon>
          <span>使用第一首歌曲封面</span>
        </button>
        <button class="cover-menu-item" @click="selectImageFile">
          <NIcon size="16"><ImageOutline /></NIcon>
          <span>选择图片文件</span>
        </button>
        <button v-if="playlist.coverUrl" class="cover-menu-item danger" @click="removeCover">
          <NIcon size="16"><RemoveCircleOutline /></NIcon>
          <span>移除封面</span>
        </button>
      </div>
    </div>

    <div v-if="sortedSongs.length === 0" class="empty-state">
      <p>歌单是空的</p>
      <p class="empty-hint">在搜索结果中点击 + 按钮添加歌曲</p>
    </div>

    <div v-else class="song-list" ref="listRef">
      <div
        v-for="(song, index) in sortedSongs"
        :key="song.bvid"
        class="fav-item"
        :class="{ dragging: dragIndex === index }"
        :style="getItemStyle(index)"
        @mousedown="readonly ? undefined : onMouseDown($event, index)"
      >
        <SongCard :song="song" :song-list="sortedSongs" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.playlist-song-list {
  display: flex;
  flex-direction: column;
}

.list-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
  position: relative;
}

.back-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.back-btn:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.header-info {
  flex: 1;
  min-width: 0;
}

.header-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--app-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: flex;
  align-items: center;
  gap: 4px;
}

.cover-edit-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 4px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.cover-edit-btn:hover {
  background: var(--card-hover);
  color: var(--text-secondary);
}

.header-count {
  font-size: 12px;
  color: var(--text-tertiary);
}

.header-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.header-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--text-secondary);
}

.header-btn:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.header-btn.danger:hover {
  color: #e5484d;
}

.header-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.cover-menu {
  position: absolute;
  top: 100%;
  right: 20px;
  margin-top: 4px;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 4px;
  min-width: 180px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
}

.cover-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border-radius: 5px;
  font-size: 13px;
  color: var(--app-text);
  text-align: left;
  border: none;
  background: none;
  cursor: pointer;
}

.cover-menu-item:hover {
  background: var(--card-hover);
}

.cover-menu-item.danger {
  color: #e5484d;
}

.cover-menu-item.danger:hover {
  background: rgba(229, 72, 77, 0.1);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-secondary);
  gap: 8px;
}

.empty-hint {
  font-size: 12px;
  color: var(--text-tertiary);
}

.song-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0 20px 20px;
}

.fav-item {
  cursor: grab;
  user-select: none;
}

.fav-item.dragging {
  visibility: hidden;
}
</style>
