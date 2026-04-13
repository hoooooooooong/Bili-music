<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { NSpin, NIcon } from "naive-ui";
import {
  CheckmarkDoneOutline,
  DownloadOutline,
  CloseOutline,
} from "@vicons/ionicons5";
import SongCard from "./SongCard.vue";
import { useSearchStore } from "@/stores/search";
import { useDownloadStore } from "@/stores/download";

const searchStore = useSearchStore();
const downloadStore = useDownloadStore();
const scrollRef = ref<HTMLElement | null>(null);
const itemHeight = ref(80); // 72px card + 8px gap, measured dynamically
const BUFFER = 5; // extra items above/below viewport

// Select mode state
const selectMode = ref(false);
const selectedIds = ref<Set<string>>(new Set());

function toggleSelectMode() {
  selectMode.value = !selectMode.value;
  if (!selectMode.value) {
    selectedIds.value.clear();
  }
}

function toggleSelect(bvid: string) {
  if (selectedIds.value.has(bvid)) {
    selectedIds.value.delete(bvid);
  } else {
    selectedIds.value.add(bvid);
  }
  // Trigger reactivity
  selectedIds.value = new Set(selectedIds.value);
}

function selectAll() {
  // Select all currently visible results that aren't already downloaded/done
  const doneBvids = new Set<string>();
  for (const task of downloadStore.tasks.values()) {
    if (task.status === "done") doneBvids.add(task.bvid);
  }
  for (const song of searchStore.results) {
    if (!doneBvids.has(song.bvid)) {
      selectedIds.value.add(song.bvid);
    }
  }
  selectedIds.value = new Set(selectedIds.value);
}

function clearSelection() {
  selectedIds.value.clear();
}

function batchDownloadSelected() {
  const songs = searchStore.results.filter((s) => selectedIds.value.has(s.bvid));
  if (songs.length === 0) return;
  downloadStore.batchDownload(songs);
  selectMode.value = false;
  selectedIds.value.clear();
}

const selectedCount = computed(() => selectedIds.value.size);

const visibleRange = ref({ start: 0, end: 20 });

const totalHeight = computed(() => {
  const len = searchStore.results.length;
  return len > 0 ? len * itemHeight.value - 8 : 0; // last item has no gap
});

const visibleItems = computed(() => {
  const { start, end } = visibleRange.value;
  return searchStore.results.slice(start, end).map((song, i) => ({
    song,
    originalIndex: start + i,
  }));
});

function measureItemHeight() {
  if (!scrollRef.value) return;
  const first = scrollRef.value.querySelector<HTMLElement>(".song-card");
  if (first) {
    itemHeight.value = first.offsetHeight + 8; // include gap
  }
}

function updateVisibleRange() {
  const el = scrollRef.value;
  if (!el || itemHeight.value === 0) return;
  const scrollTop = el.scrollTop;
  const viewportHeight = el.clientHeight;
  const start = Math.max(0, Math.floor(scrollTop / itemHeight.value) - BUFFER);
  const end = Math.min(
    searchStore.results.length,
    Math.ceil((scrollTop + viewportHeight) / itemHeight.value) + BUFFER
  );
  visibleRange.value = { start, end };
}

// Watch for new results to re-measure and reset
const originalLength = ref(0);
watch(() => searchStore.results.length, async (len) => {
  if (len !== originalLength.value) {
    originalLength.value = len;
    await nextTick();
    measureItemHeight();
    updateVisibleRange();
  }
});

// Re-measure when selectMode toggles (checkbox changes card height)
watch(selectMode, async () => {
  await nextTick();
  measureItemHeight();
  updateVisibleRange();
});

onMounted(() => {
  measureItemHeight();
  updateVisibleRange();
});

onUnmounted(() => {
  scrollRef.value = null;
});
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
        <span v-if="!selectMode">搜索到 {{ searchStore.total }} 个结果</span>
        <template v-else>
          <button class="header-btn" @click="selectAll">
            <NIcon size="16"><CheckmarkDoneOutline /></NIcon>
            全选
          </button>
          <span class="select-info">已选 {{ selectedCount }} 首</span>
          <div class="header-spacer"></div>
          <button class="header-btn" @click="clearSelection">取消选择</button>
          <button class="header-btn" @click="toggleSelectMode">
            <NIcon size="16"><CloseOutline /></NIcon>
            退出
          </button>
        </template>
        <div v-if="!selectMode" class="header-spacer"></div>
        <button v-if="!selectMode" class="header-btn select-mode-btn" @click="toggleSelectMode">
          <NIcon size="16"><CheckmarkDoneOutline /></NIcon>
          多选
        </button>
      </div>
      <div
        class="song-list"
        ref="scrollRef"
        @scroll="updateVisibleRange"
      >
        <div class="scroll-spacer" :style="{ height: totalHeight + 'px' }">
          <div
            v-for="{ song, originalIndex } in visibleItems"
            :key="song.bvid"
            class="song-item"
            :style="{ position: 'absolute', top: originalIndex * itemHeight + 'px', left: 0, right: 0 }"
          >
            <SongCard
              :song="song"
              :song-list="searchStore.results"
              :select-mode="selectMode"
              :selected="selectedIds.has(song.bvid)"
              @toggle-select="toggleSelect"
            />
          </div>
        </div>
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
      <!-- Batch download floating bar -->
      <Teleport to="body">
        <Transition name="float-bar">
          <div v-if="selectMode && selectedCount > 0" class="batch-download-bar">
            <button class="batch-download-btn" @click="batchDownloadSelected">
              <NIcon size="18"><DownloadOutline /></NIcon>
              下载选中 ({{ selectedCount }} 首)
            </button>
          </div>
        </Transition>
      </Teleport>
    </template>
  </div>
</template>

<style scoped>
.result-grid {
  padding: 0 20px 20px;
  display: flex;
  flex-direction: column;
  min-height: 0;
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
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-spacer {
  flex: 1;
}

.header-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  color: var(--text-secondary);
  background: none;
  border: none;
  cursor: pointer;
  transition: all 0.15s;
}

.header-btn:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.select-mode-btn {
  color: var(--accent-color);
}

.select-mode-btn:hover {
  background: var(--accent-light);
}

.select-info {
  font-size: 12px;
  color: var(--accent-color);
  font-weight: 500;
}

/* Batch download floating bar */
.batch-download-bar {
  position: fixed;
  bottom: 80px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 100;
}

.batch-download-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 24px;
  border-radius: 24px;
  font-size: 14px;
  font-weight: 500;
  color: white;
  background: var(--accent-color);
  border: none;
  cursor: pointer;
  box-shadow: 0 4px 16px rgba(251, 114, 153, 0.4);
  transition: all 0.15s;
}

.batch-download-btn:hover {
  background: var(--accent-hover);
  box-shadow: 0 4px 20px rgba(251, 114, 153, 0.5);
}

.float-bar-enter-active,
.float-bar-leave-active {
  transition: all 0.2s ease;
}

.float-bar-enter-from,
.float-bar-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(20px);
}

.song-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.scroll-spacer {
  position: relative;
}

.load-more {
  display: flex;
  justify-content: center;
  padding: 16px 0;
  flex-shrink: 0;
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
