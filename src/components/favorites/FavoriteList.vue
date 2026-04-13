<script setup lang="ts">
import { ref, computed } from "vue";
import SongCard from "../search/SongCard.vue";
import { useFavoritesStore } from "@/stores/favorites";
import { useDragSort } from "@/composables/useDragSort";

const favorites = useFavoritesStore();
const sortedFavorites = computed(() => [...favorites.favorites].reverse());

const listRef = ref<HTMLElement | null>(null);

function toStoreIndex(visualIndex: number) {
  return favorites.favorites.length - 1 - visualIndex;
}

const { dragIndex, getItemStyle, onMouseDown } = useDragSort({
  listRef,
  itemSelector: ".fav-item",
  ghostClass: "fav-drag-ghost",
  skipSelector: ".card-actions",
  gap: 8,
  onDrop: (from, to) => favorites.moveFavorite(toStoreIndex(from), toStoreIndex(to)),
});
</script>

<template>
  <div class="favorite-list">
    <div v-if="sortedFavorites.length === 0" class="empty-state">
      <p>还没有收藏</p>
      <p class="empty-hint">点击歌曲卡片的爱心图标添加收藏</p>
    </div>
    <div v-else class="song-list" ref="listRef">
      <div
        v-for="(song, index) in sortedFavorites"
        :key="song.bvid"
        class="fav-item"
        :class="{ dragging: dragIndex === index }"
        :style="getItemStyle(index)"
        @mousedown="onMouseDown($event, index)"
      >
        <SongCard :song="song" :song-list="sortedFavorites" />
      </div>
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

.fav-item {
  cursor: grab;
  user-select: none;
}

.fav-item.dragging {
  visibility: hidden;
}
</style>

<style>
.fav-drag-ghost {
  opacity: 0.85;
  background: var(--card-bg);
  border-radius: 10px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  cursor: grabbing;
  padding: 10px 12px;
}
</style>
