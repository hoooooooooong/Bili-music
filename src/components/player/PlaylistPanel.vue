<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { NIcon } from "naive-ui";
import { CloseOutline, TrashOutline } from "@vicons/ionicons5";
import { usePlayerStore } from "@/stores/player";
import { useDragSort } from "@/composables/useDragSort";
import type { Song } from "@/types";

const emit = defineEmits<{ close: [] }>();
const player = usePlayerStore();

const listRef = ref<HTMLElement | null>(null);

const { dragIndex, getItemStyle, onMouseDown, isDragging } = useDragSort({
  listRef,
  itemSelector: ".playlist-item",
  ghostClass: "playlist-item drag-ghost",
  skipSelector: ".item-remove",
  onDrop: (from, to) => player.movePlaylistItem(from, to),
});

function playSong(song: Song, index: number) {
  if (isDragging()) return;
  player.currentIndex = index;
  player.playSong(song);
}

watch(() => player.currentIndex, async (idx) => {
  if (idx < 0 || !listRef.value) return;
  await nextTick();
  const items = listRef.value!.querySelectorAll<HTMLElement>(".playlist-item");
  items[idx]?.scrollIntoView({ behavior: "smooth", block: "nearest" });
});
</script>

<template>
  <div class="playlist-panel">
    <div class="panel-header">
      <h3>播放列表 ({{ player.playlist.length }})</h3>
      <div class="panel-actions">
        <button
          class="panel-btn"
          @click="player.clearPlaylist()"
          title="清空"
        >
          <NIcon size="16"><TrashOutline /></NIcon>
        </button>
        <button class="panel-btn" @click="emit('close')">
          <NIcon size="16"><CloseOutline /></NIcon>
        </button>
      </div>
    </div>

    <div class="panel-list" ref="listRef">
      <div v-if="player.playlist.length === 0" class="panel-empty">
        <p>播放列表为空</p>
      </div>
      <div
        v-for="(song, index) in player.playlist"
        :key="song.bvid"
        class="playlist-item"
        :class="{ active: index === player.currentIndex, dragging: dragIndex === index }"
        :style="getItemStyle(index)"
        @mousedown="onMouseDown($event, index)"
        @click="playSong(song, index)"
      >
        <span v-if="index === player.currentIndex && player.isPlaying" class="playing-indicator">
          <span class="bar"></span>
          <span class="bar"></span>
          <span class="bar"></span>
        </span>
        <span v-else class="item-index">{{ index + 1 }}</span>
        <div class="item-info">
          <p class="item-title">{{ song.title }}</p>
          <p class="item-author">{{ song.author }}</p>
        </div>
        <div class="item-duration">{{ song.duration }}</div>
        <button class="item-remove" @click.stop="player.removeFromPlaylist(index)">
          <NIcon size="14"><CloseOutline /></NIcon>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.playlist-panel {
  position: fixed;
  right: 0;
  top: 0;
  bottom: 0;
  width: 340px;
  background: var(--header-bg);
  border-left: 1px solid var(--border-color);
  z-index: 150;
  display: flex;
  flex-direction: column;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.panel-header h3 {
  font-size: 15px;
  font-weight: 600;
}

.panel-actions {
  display: flex;
  gap: 4px;
}

.panel-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  border-radius: 6px;
}

.panel-btn:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.panel-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.panel-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-tertiary);
  font-size: 13px;
}

.playlist-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  cursor: grab;
  user-select: none;
}

.playlist-item:hover {
  background: var(--card-hover);
}

.playlist-item.active {
  background: var(--accent-light);
}

.playlist-item.dragging {
  visibility: hidden;
}

.drag-ghost {
  opacity: 0.85;
  background: var(--card-hover);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  cursor: grabbing;
}

.item-index {
  width: 24px;
  text-align: center;
  font-size: 12px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.playlist-item.active .item-index {
  color: var(--accent-color);
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-title {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.playlist-item.active .item-title {
  color: var(--accent-color);
  font-weight: 500;
}

.item-author {
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-duration {
  font-size: 11px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.item-remove {
  opacity: 0;
  color: var(--text-secondary);
  cursor: pointer;
}

.playlist-item:hover .item-remove {
  opacity: 1;
}

.item-remove:hover {
  color: var(--accent-color);
}

.playing-indicator {
  display: inline-flex;
  align-items: flex-end;
  gap: 1.5px;
  height: 12px;
  width: 24px;
  justify-content: center;
  flex-shrink: 0;
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

@keyframes equalizer {
  0%, 100% { height: 3px; }
  50% { height: 12px; }
}
</style>
