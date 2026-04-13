<script setup lang="ts">
import { ref, watch } from "vue";
import AppHeader from "@/components/AppHeader.vue";
import WelcomeSection from "@/components/search/WelcomeSection.vue";
import ResultGrid from "@/components/search/ResultGrid.vue";
import PlaylistsTab from "@/components/playlists/PlaylistsTab.vue";
import { useSearchStore } from "@/stores/search";
import { usePlayerStore } from "@/stores/player";
import { useHistoryStore } from "@/stores/history";
import { useKeyboardShortcuts } from "@/composables/useKeyboardShortcuts";
import { NIcon } from "naive-ui";
import { HeartOutline, TimeOutline, SearchOutline } from "@vicons/ionicons5";
import type { Song } from "@/types";

const searchStore = useSearchStore();
const player = usePlayerStore();
const historyStore = useHistoryStore();
const activeTab = ref<"search" | "favorites" | "history">("search");

useKeyboardShortcuts();

watch(() => searchStore.keyword, (kw) => {
  if (kw && activeTab.value !== "search") {
    activeTab.value = "search";
  }
});

function handleSearch(keyword: string) {
  activeTab.value = "search";
  searchStore.search(keyword);
}

function playSong(song: Song) {
  player.playSong(song);
}
</script>

<template>
  <div class="home-page">
    <AppHeader />

    <div class="tab-bar">
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'search' }"
        @click="activeTab = 'search'"
      >
        <NIcon size="15"><SearchOutline /></NIcon>
        搜索
      </button>
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'favorites' }"
        @click="activeTab = 'favorites'"
      >
        <NIcon size="15"><HeartOutline /></NIcon>
        收藏
      </button>
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'history' }"
        @click="activeTab = 'history'"
      >
        <NIcon size="15"><TimeOutline /></NIcon>
        历史
      </button>
    </div>

    <div v-show="activeTab === 'search'" class="tab-content">
      <WelcomeSection
        v-if="!searchStore.keyword && !searchStore.loading"
        @search="handleSearch"
      />
      <ResultGrid v-else />
    </div>

    <div v-show="activeTab === 'favorites'">
      <PlaylistsTab />
    </div>

    <div v-show="activeTab === 'history'" class="history-section">
      <div
        v-if="historyStore.history.length === 0"
        class="empty-state"
      >
        <p>还没有播放历史</p>
      </div>
      <div v-else class="history-list">
        <button
          v-for="entry in historyStore.history.slice(0, 50)"
          :key="entry.song.bvid + entry.playedAt"
          class="history-item"
          @click="playSong(entry.song)"
        >
          <div class="history-cover-wrap">
            <img
              :src="entry.song.coverUrl || `bili-cover://${entry.song.bvid}`"
              :alt="entry.song.title"
              class="history-cover"
              loading="lazy"
              @load="($event.target as HTMLImageElement).classList.add('loaded')"
            />
          </div>
          <div class="history-info">
            <p class="history-title">{{ entry.song.title }}</p>
            <p class="history-author">{{ entry.song.author }}</p>
          </div>
          <span class="history-count">x{{ entry.playCount }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-page {
  display: flex;
  flex-direction: column;
}

.tab-content {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.tab-bar {
  display: flex;
  gap: 4px;
  padding: 8px 20px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 13px;
  color: var(--text-secondary);
}

.tab-btn:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.tab-btn.active {
  background: var(--accent-light);
  color: var(--accent-color);
  font-weight: 500;
}

.history-section {
  padding: 0 20px 20px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-secondary);
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  text-align: left;
  width: 100%;
}

.history-item:hover {
  background: var(--card-hover);
}

.history-cover-wrap {
  width: 40px;
  height: 40px;
  border-radius: 6px;
  flex-shrink: 0;
  overflow: hidden;
  background: var(--card-hover);
  animation: skeleton-pulse 1.5s ease-in-out infinite;
}

.history-cover-wrap.loaded {
  animation: none;
}

.history-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0;
  transition: opacity 0.3s;
}

.history-cover.loaded {
  opacity: 1;
}

@keyframes skeleton-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.history-info {
  flex: 1;
  min-width: 0;
}

.history-title {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.history-author {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.history-count {
  font-size: 11px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}
</style>
