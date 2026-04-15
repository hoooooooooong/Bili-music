<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NIcon, NSpin } from "naive-ui";
import { ArrowBackOutline, PlayOutline, DownloadOutline } from "@vicons/ionicons5";
import { invoke } from "@tauri-apps/api/core";
import SongCard from "../search/SongCard.vue";
import { usePlayerStore } from "@/stores/player";
import { useDownloadStore } from "@/stores/download";
import type { Song, MediaResourcePage } from "@/types";

const props = defineProps<{
  folderId: number;
  folderTitle: string;
  uid: string;
}>();

const emit = defineEmits<{
  back: [];
}>();

const player = usePlayerStore();
const downloadStore = useDownloadStore();

const songs = ref<Song[]>([]);
const page = ref(1);
const total = ref(0);
const hasMore = ref(false);
const loading = ref(false);
const loadingMore = ref(false);

async function fetchPage(p: number) {
  const isLoadMore = p > 1;
  if (isLoadMore) {
    loadingMore.value = true;
  } else {
    loading.value = true;
  }
  try {
    const result = await invoke<MediaResourcePage>(
      "fetch_favorites_folder_videos",
      { uid: props.uid, mediaId: props.folderId, page: p }
    );
    if (p === 1) {
      songs.value = result.videos;
    } else {
      songs.value.push(...result.videos);
    }
    page.value = result.page;
    total.value = result.total;
    hasMore.value = result.hasMore;
  } catch {
    // ignore
  } finally {
    loading.value = false;
    loadingMore.value = false;
  }
}

function loadMore() {
  if (loadingMore.value || !hasMore.value) return;
  fetchPage(page.value + 1);
}

function playAll() {
  if (songs.value.length === 0) return;
  player.playSong(songs.value[0], songs.value);
}

function downloadAll() {
  if (songs.value.length === 0) return;
  downloadStore.batchDownload(songs.value);
}

onMounted(() => fetchPage(1));
</script>

<template>
  <div class="bili-folder-view">
    <div class="list-header">
      <button class="back-btn" @click="emit('back')">
        <NIcon size="18"><ArrowBackOutline /></NIcon>
      </button>
      <div class="header-info">
        <h3 class="header-name">{{ folderTitle }}</h3>
        <span class="header-count">{{ total }} 首</span>
      </div>
      <div class="header-actions">
        <button class="header-btn" @click="playAll" title="播放全部">
          <NIcon size="18"><PlayOutline /></NIcon>
        </button>
        <button class="header-btn" @click="downloadAll" title="下载全部">
          <NIcon size="18"><DownloadOutline /></NIcon>
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading-wrap">
      <NSpin size="24" />
    </div>

    <div v-else-if="songs.length === 0" class="empty-state">
      <p>收藏夹为空</p>
    </div>

    <template v-else>
      <div class="song-list">
        <SongCard
          v-for="song in songs"
          :key="song.bvid"
          :song="song"
        />
      </div>

      <div v-if="hasMore" class="load-more">
        <button
          class="load-more-btn"
          :disabled="loadingMore"
          @click="loadMore"
        >
          <NSpin v-if="loadingMore" :size="14" />
          {{ loadingMore ? "加载中..." : "加载更多" }}
        </button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.bili-folder-view {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.list-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
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

.loading-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 60px 0;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-secondary);
  font-size: 13px;
}

.song-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 20px 20px;
}

.load-more {
  display: flex;
  justify-content: center;
  padding: 12px 0 20px;
  flex-shrink: 0;
}

.load-more-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 24px;
  border-radius: 16px;
  font-size: 13px;
  color: var(--accent-color);
  background: var(--accent-light);
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
}

.load-more-btn:hover:not(:disabled) {
  border-color: var(--accent-color);
}

.load-more-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
