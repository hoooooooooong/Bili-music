<script setup lang="ts">
import { computed } from "vue";
import SongCard from "../search/SongCard.vue";
import { useFavoritesStore } from "@/stores/favorites";

const favorites = useFavoritesStore();
const sortedFavorites = computed(() => [...favorites.favorites].reverse());
</script>

<template>
  <div class="favorite-list">
    <div v-if="sortedFavorites.length === 0" class="empty-state">
      <p>还没有收藏</p>
      <p class="empty-hint">点击歌曲卡片的爱心图标添加收藏</p>
    </div>
    <div v-else class="song-list">
      <SongCard
        v-for="song in sortedFavorites"
        :key="song.bvid"
        :song="song"
      />
    </div>
  </div>
</template>

<style scoped>
.favorite-list {
  padding: 0 20px 20px;
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
}
</style>
