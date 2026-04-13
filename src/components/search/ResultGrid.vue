<script setup lang="ts">
import { NSpin } from "naive-ui";
import SongCard from "./SongCard.vue";
import { useSearchStore } from "@/stores/search";

const searchStore = useSearchStore();
</script>

<template>
  <div class="result-grid">
    <div
      v-if="searchStore.loading && searchStore.results.length === 0"
      class="loading-state"
    >
      <NSpin size="large" />
      <p>搜索中...</p>
    </div>

    <div
      v-else-if="
        searchStore.error && searchStore.results.length === 0
      "
      class="error-state"
    >
      <p class="error-text">{{ searchStore.error }}</p>
    </div>

    <div
      v-else-if="
        !searchStore.loading &&
        searchStore.results.length === 0 &&
        searchStore.keyword
      "
      class="empty-state"
    >
      <p>未找到相关结果</p>
    </div>

    <template v-else-if="searchStore.results.length > 0">
      <div class="results-header">
        <span>搜索到 {{ searchStore.total }} 个结果</span>
      </div>
      <div class="song-list">
        <SongCard
          v-for="song in searchStore.results"
          :key="song.bvid"
          :song="song"
        />
      </div>
      <div class="load-more">
        <button
          v-if="searchStore.hasMore"
          class="load-more-btn"
          :disabled="searchStore.loading"
          @click="searchStore.loadMore()"
        >
          <NSpin v-if="searchStore.loading" :size="14" />
          {{ searchStore.loading ? "加载中..." : "加载更多" }}
        </button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.result-grid {
  padding: 0 20px 20px;
}

.loading-state,
.empty-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-secondary);
  gap: 12px;
}

.error-text {
  color: #e81123;
}

.results-header {
  font-size: 12px;
  color: var(--text-secondary);
  padding: 8px 0;
}

.song-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.load-more {
  display: flex;
  justify-content: center;
  padding: 16px 0;
}

.load-more-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 24px;
  border-radius: 18px;
  font-size: 13px;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  transition: all 0.15s;
}

.load-more-btn:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.load-more-btn:disabled {
  opacity: 0.5;
}
</style>
