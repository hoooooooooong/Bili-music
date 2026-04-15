<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { NIcon } from "naive-ui";
import {
  SearchOutline,
  CloseOutline,
  TimeOutline,
  ArrowUpOutline,
  TrashOutline,
  FlameOutline,
} from "@vicons/ionicons5";
import { invoke } from "@tauri-apps/api/core";
import { useSearchStore } from "@/stores/search";
import { useHistoryStore } from "@/stores/history";
import type { Song } from "@/types";

const searchStore = useSearchStore();
const historyStore = useHistoryStore();

const query = ref("");
const showDropdown = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);
const hotSongs = ref<Song[]>([]);

const recentSearches = computed(() => historyStore.searchHistory.slice(0, 10));
const hotKeywords = computed(() =>
  hotSongs.value.slice(0, 8).map((s) => s.title)
);

onMounted(async () => {
  try {
    hotSongs.value = await invoke<Song[]>("get_hot_ranking");
  } catch {
    hotSongs.value = [];
  }
});

function doSearch(kw?: string) {
  const keyword = (kw || query.value).trim();
  if (!keyword) return;
  query.value = keyword;
  historyStore.addSearch(keyword);
  showDropdown.value = false;
  searchStore.search(keyword);
}

function clearInput() {
  query.value = "";
  searchStore.clear();
  inputRef.value?.focus();
}

function selectHistory(kw: string) {
  query.value = kw;
  doSearch(kw);
}

function clearAllHistory() {
  historyStore.clearSearchHistory();
}

function removeHistoryItem(kw: string, e: MouseEvent) {
  e.stopPropagation();
  historyStore.removeSearch(kw);
}

function onFocus() {
  showDropdown.value = true;
}

function onBlur() {
  setTimeout(() => {
    showDropdown.value = false;
  }, 200);
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") doSearch();
}
</script>

<template>
  <div class="search-bar-wrapper">
    <div class="search-bar">
      <NIcon size="18" class="search-icon"><SearchOutline /></NIcon>
      <input
        ref="inputRef"
        v-model="query"
        type="text"
        placeholder="搜索 B 站音乐..."
        class="search-input"
        @focus="onFocus"
        @blur="onBlur"
        @keydown="handleKeydown"
      />
      <button v-if="query" class="clear-btn" @click="clearInput">
        <NIcon size="14"><CloseOutline /></NIcon>
      </button>
    </div>

    <Transition name="dropdown">
      <div v-if="showDropdown" class="search-dropdown">
        <div v-if="hotKeywords.length > 0" class="hot-section">
          <div class="dropdown-header">
            <span class="dropdown-title">
              <NIcon size="14" class="flame-icon"><FlameOutline /></NIcon>
              热门搜索
            </span>
          </div>
          <div class="hot-tags">
            <button
              v-for="kw in hotKeywords"
              :key="kw"
              class="hot-tag"
              @mousedown.prevent="doSearch(kw)"
            >{{ kw }}</button>
          </div>
          <div v-if="recentSearches.length > 0" class="dropdown-divider"></div>
        </div>

        <template v-if="recentSearches.length > 0">
          <div class="dropdown-header">
            <span class="dropdown-title">
              <NIcon size="14"><TimeOutline /></NIcon>
              搜索历史
            </span>
            <button class="clear-history-btn" @click="clearAllHistory">
              <NIcon size="14"><ArrowUpOutline /></NIcon>
              清空
            </button>
          </div>
          <div class="dropdown-list">
            <button
              v-for="kw in recentSearches"
              :key="kw"
              class="dropdown-item"
              @mousedown.prevent="selectHistory(kw)"
            >
              <span class="dropdown-item-text">{{ kw }}</span>
              <span
                class="dropdown-item-del"
                @mousedown.prevent="removeHistoryItem(kw, $event)"
              ><NIcon size="14"><TrashOutline /></NIcon></span>
            </button>
          </div>
        </template>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.search-bar-wrapper {
  position: relative;
  flex: 1;
  max-width: 480px;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  height: 36px;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 18px;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.search-bar:focus-within {
  border-color: var(--accent-color);
  box-shadow: 0 0 0 2px var(--accent-light);
}

.search-icon {
  color: var(--text-secondary);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  font-size: 14px;
  color: var(--app-text);
  background: transparent;
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

.clear-btn {
  color: var(--text-secondary);
  display: flex;
  align-items: center;
}

.clear-btn:hover {
  color: var(--app-text);
}

.search-dropdown {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  right: 0;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  box-shadow: var(--shadow);
  z-index: 100;
  max-height: 360px;
  overflow-y: auto;
}

.dropdown-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px 6px;
}

.dropdown-title {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--text-secondary);
}

.clear-history-btn {
  display: flex;
  align-items: center;
  gap: 2px;
  font-size: 12px;
  color: var(--text-secondary);
}

.clear-history-btn:hover {
  color: var(--accent-color);
}

.dropdown-list {
  padding: 4px 0;
}

.dropdown-item {
  display: flex;
  align-items: center;
  width: 100%;
  text-align: left;
  padding: 8px 14px;
  font-size: 13px;
  color: var(--app-text);
}

.dropdown-item-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dropdown-item-del {
  flex-shrink: 0;
  display: none;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  margin-left: 8px;
}

.dropdown-item:hover .dropdown-item-del {
  display: flex;
}

.dropdown-item-del:hover {
  color: var(--danger-color, #e5484d);
}

.dropdown-item:hover {
  background: var(--card-hover);
}

.dropdown-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 14px;
}

.flame-icon {
  color: #fb7299;
}

.hot-section {
  padding-bottom: 4px;
}

.hot-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 2px 14px 8px;
}

.hot-tag {
  padding: 3px 12px;
  border-radius: 14px;
  font-size: 12px;
  color: var(--app-text);
  background: var(--card-hover);
  border: 1px solid var(--border-color);
  transition: all 0.15s;
}

.hot-tag:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
  background: var(--accent-light);
}

.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.15s, transform 0.15s;
}
.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
