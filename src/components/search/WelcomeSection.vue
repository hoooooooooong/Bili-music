<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NIcon } from "naive-ui";
import {
  MusicalNotesOutline,
  FlameOutline,
  PlayOutline,
} from "@vicons/ionicons5";
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

const isLoggedIn = ref(false);
const loginChecked = ref(false);

// Popular videos state
const popular = ref<Song[]>([]);
const popularPage = ref(1);
const popularTotal = ref(0);
const popularLoading = ref(false);
const popularLoadingMore = ref(false);
const POPULAR_PAGE_SIZE = 20;

// Ranking state (for non-logged-in users)
const ranking = ref<Song[]>([]);
const loading = ref(false);

onMounted(async () => {
  try {
    isLoggedIn.value = await invoke<boolean>("check_login");
  } catch {
    isLoggedIn.value = false;
  }
  loginChecked.value = true;

  if (isLoggedIn.value) {
    fetchPopular();
  } else {
    fetchRanking();
  }
});

async function fetchPopular() {
  popularLoading.value = true;
  try {
    const resp = await invoke<{
      results: Song[];
      page: number;
      total: number;
      pageSize: number;
    }>("get_popular", {
      page: 1,
      pageSize: POPULAR_PAGE_SIZE,
    });
    popular.value = resp.results;
    popularPage.value = 1;
    popularTotal.value = resp.total;
  } catch {
    popular.value = [];
  } finally {
    popularLoading.value = false;
  }
}

async function loadMorePopular() {
  if (popularLoadingMore.value) return;
  const nextPage = popularPage.value + 1;
  if (nextPage * POPULAR_PAGE_SIZE > popularTotal.value) return;

  popularLoadingMore.value = true;
  try {
    const resp = await invoke<{
      results: Song[];
      page: number;
      total: number;
      pageSize: number;
    }>("get_popular", {
      page: nextPage,
      pageSize: POPULAR_PAGE_SIZE,
    });
    popular.value.push(...resp.results);
    popularPage.value = nextPage;
    popularTotal.value = resp.total;
  } catch {
    // ignore
  } finally {
    popularLoadingMore.value = false;
  }
}

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
  const list = isLoggedIn.value ? popular.value : ranking.value;
  if (list.length === 0) return;
  player.playSong(list[0], list);
}
</script>

<template>
  <div class="welcome-section">
    <!-- Logged in: Popular videos view -->
    <template v-if="loginChecked && isLoggedIn">
      <div class="popular-header">
        <h2 class="popular-title">
          <NIcon size="20"><MusicalNotesOutline /></NIcon>
          音乐区
        </h2>
        <button class="play-all-btn" @click="playAll">
          <NIcon size="14"><PlayOutline /></NIcon>
          播放全部
        </button>
      </div>

      <div v-if="popularLoading" class="popular-loading">加载中...</div>

      <div v-else-if="popular.length === 0" class="popular-empty">
        暂时无法获取热门视频
      </div>

      <template v-else>
        <div class="popular-grid">
          <div
            v-for="song in popular"
            :key="song.bvid"
            class="popular-card"
            @click="playSong(song)"
          >
            <div class="popular-cover-wrap">
              <img
                :src="song.coverUrl"
                :alt="song.title"
                class="popular-cover"
                loading="lazy"
                @error="($event.target as HTMLImageElement).src = `bili-cover://${song.bvid}`"
              />
              <div class="popular-cover-overlay">
                <div class="popular-play-icon">
                  <NIcon size="24" color="white"><PlayOutline /></NIcon>
                </div>
              </div>
              <span class="popular-duration">{{ song.duration }}</span>
            </div>
            <div class="popular-info">
              <p class="popular-card-title">{{ song.title }}</p>
              <p class="popular-card-author">{{ song.author }}</p>
              <span class="popular-card-plays">{{ song.playCountText }} 播放</span>
            </div>
          </div>
        </div>

        <div
          v-if="popularPage * POPULAR_PAGE_SIZE < popularTotal"
          class="popular-load-more"
        >
          <button
            class="load-more-btn"
            :disabled="popularLoadingMore"
            @click="loadMorePopular"
          >
            {{ popularLoadingMore ? "加载中..." : "加载更多" }}
          </button>
        </div>
      </template>
    </template>

    <!-- Not logged in: Original welcome + ranking view -->
    <template v-else-if="loginChecked && !isLoggedIn">
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
    </template>
  </div>
</template>

<style scoped>
.welcome-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 20px 20px;
  overflow-y: auto;
  width: 100%;
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

/* Popular videos grid styles */
.popular-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  margin-bottom: 16px;
  flex-shrink: 0;
}

.popular-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 18px;
  font-weight: 600;
  color: var(--app-text);
  margin: 0;
}

.popular-title .n-icon {
  color: var(--accent-color);
}

.popular-loading,
.popular-empty {
  text-align: center;
  padding: 60px 0;
  font-size: 13px;
  color: var(--text-tertiary);
}

.popular-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
  width: 100%;
}

.popular-card {
  display: flex;
  flex-direction: column;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.15s;
  overflow: hidden;
  background: var(--card-bg);
  border: 1px solid transparent;
}

.popular-card:hover {
  background: var(--card-hover);
  border-color: var(--border-color);
  transform: translateY(-1px);
}

.popular-cover-wrap {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
  overflow: hidden;
  background: var(--skeleton-bg, var(--card-hover));
}

.popular-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.popular-cover-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.15s;
}

.popular-card:hover .popular-cover-overlay {
  opacity: 1;
}

.popular-play-icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--accent-color);
  display: flex;
  align-items: center;
  justify-content: center;
}

.popular-duration {
  position: absolute;
  bottom: 4px;
  right: 4px;
  padding: 1px 5px;
  background: rgba(0, 0, 0, 0.7);
  border-radius: 3px;
  font-size: 11px;
  color: white;
}

.popular-info {
  padding: 8px 10px;
  min-width: 0;
}

.popular-card-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--app-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 2px;
}

.popular-card-author {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 2px;
}

.popular-card-plays {
  font-size: 11px;
  color: var(--text-tertiary);
}

.popular-load-more {
  display: flex;
  justify-content: center;
  padding: 20px 0 10px;
}

.load-more-btn {
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
