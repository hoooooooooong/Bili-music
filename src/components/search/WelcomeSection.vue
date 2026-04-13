<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NIcon } from "naive-ui";
import { MusicalNotesOutline, FlameOutline, PlayOutline } from "@vicons/ionicons5";
import { invoke } from "@tauri-apps/api/core";
import { usePlayerStore } from "@/stores/player";
import type { Song } from "@/types";

const hotKeywords = [
  "周杰伦",
  "Aimer",
  "YOASOBI",
  "LiSA",
  "米津玄師",
  "Ado",
  "EVA",
  "原神",
];

const emit = defineEmits<{ search: [keyword: string] }>();
const player = usePlayerStore();

const ranking = ref<Song[]>([]);
const loading = ref(false);

onMounted(fetchRanking);

async function fetchRanking() {
  loading.value = true;
  try {
    ranking.value = await invoke<Song[]>("get_hot_ranking");
  } catch {
    ranking.value = [];
  } finally {
    loading.value = false;
  }
}

function playSong(song: Song) {
  player.playSong(song);
}

function playAll() {
  if (ranking.value.length === 0) return;
  player.playSong(ranking.value[0], ranking.value);
}
</script>

<template>
  <div class="welcome-section">
    <div class="welcome-top">
      <div class="welcome-icon">
        <NIcon size="48" color="var(--accent-color)">
          <MusicalNotesOutline />
        </NIcon>
      </div>
      <h2 class="welcome-title">搜索 B 站音乐</h2>
      <p class="welcome-desc">搜索你喜欢的音乐，在线播放或下载为 MP3</p>
      <div class="hot-keywords">
        <button
          v-for="kw in hotKeywords"
          :key="kw"
          class="keyword-tag"
          @click="emit('search', kw)"
        >
          {{ kw }}
        </button>
      </div>
    </div>

    <div class="ranking-section">
      <div class="ranking-header">
        <span class="ranking-title">
          <NIcon size="16"><FlameOutline /></NIcon>
          音乐区热门
        </span>
        <button class="play-all-btn" @click="playAll">
          <NIcon size="14"><PlayOutline /></NIcon>
          播放全部
        </button>
      </div>

      <div v-if="loading" class="ranking-loading">加载中...</div>

      <div v-else-if="ranking.length === 0" class="ranking-empty">
        暂时无法获取排行榜
      </div>

      <div v-else class="ranking-list">
        <button
          v-for="(song, idx) in ranking"
          :key="song.bvid"
          class="ranking-item"
          @click="playSong(song)"
        >
          <span
            class="ranking-rank"
            :class="{ top: idx < 3 }"
          >{{ idx + 1 }}</span>
          <div class="ranking-cover-wrap">
            <img
              :src="song.coverUrl || `bili-cover://${song.bvid}`"
              :alt="song.title"
              class="ranking-cover"
              loading="lazy"
            />
          </div>
          <div class="ranking-info">
            <p class="ranking-song-title">{{ song.title }}</p>
            <p class="ranking-author">{{ song.author }}</p>
          </div>
          <span class="ranking-plays">{{ song.playCountText }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.welcome-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 40px 20px 20px;
  overflow-y: auto;
}

.welcome-top {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  flex-shrink: 0;
}

.welcome-icon {
  margin-bottom: 16px;
}

.welcome-title {
  font-size: 22px;
  font-weight: 600;
  color: var(--app-text);
  margin-bottom: 8px;
}

.welcome-desc {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 32px;
}

.hot-keywords {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: center;
  max-width: 400px;
}

.keyword-tag {
  padding: 6px 16px;
  border-radius: 16px;
  font-size: 13px;
  color: var(--app-text);
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  transition: all 0.15s;
}

.keyword-tag:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
  background: var(--accent-light);
}

.ranking-section {
  width: 100%;
  max-width: 600px;
  margin-top: 36px;
}

.ranking-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.ranking-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 15px;
  font-weight: 600;
  color: var(--app-text);
}

.ranking-title .n-icon {
  color: #fb7299;
}

.play-all-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  border-radius: 14px;
  font-size: 12px;
  color: var(--accent-color);
  background: var(--accent-light);
  border: none;
  cursor: pointer;
}

.play-all-btn:hover {
  filter: brightness(0.9);
}

.ranking-loading,
.ranking-empty {
  text-align: center;
  padding: 32px 0;
  font-size: 13px;
  color: var(--text-tertiary);
}

.ranking-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.ranking-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  text-align: left;
  width: 100%;
  background: none;
  border: none;
}

.ranking-item:hover {
  background: var(--card-hover);
}

.ranking-rank {
  width: 22px;
  flex-shrink: 0;
  font-size: 14px;
  font-weight: 700;
  color: var(--text-tertiary);
  text-align: center;
  font-variant-numeric: tabular-nums;
}

.ranking-rank.top {
  color: #fb7299;
}

.ranking-cover-wrap {
  width: 40px;
  height: 40px;
  border-radius: 6px;
  flex-shrink: 0;
  overflow: hidden;
  background: var(--card-hover);
}

.ranking-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.ranking-info {
  flex: 1;
  min-width: 0;
}

.ranking-song-title {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--app-text);
}

.ranking-author {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ranking-plays {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--text-tertiary);
}
</style>
