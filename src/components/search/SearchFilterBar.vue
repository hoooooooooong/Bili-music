<script setup lang="ts">
import { useSearchStore } from "@/stores/search";
import type { SearchOrder, DurationFilter } from "@/types";

const searchStore = useSearchStore();

interface SortOption {
  value: SearchOrder;
  label: string;
}

interface DurationOption {
  value: DurationFilter;
  label: string;
}

const sortOptions: SortOption[] = [
  { value: "totalrank", label: "综合排序" },
  { value: "pubdate", label: "最新发布" },
  { value: "click", label: "最多播放" },
  { value: "stow", label: "最多收藏" },
  { value: "dm", label: "最多弹幕" },
];

const durationOptions: DurationOption[] = [
  { value: "all", label: "全部" },
  { value: "short", label: "<3分钟" },
  { value: "medium", label: "3-5分钟" },
  { value: "long", label: "5-10分钟" },
  { value: "very-long", label: ">10分钟" },
];
</script>

<template>
  <div class="search-filter-bar">
    <div class="filter-group">
      <span class="filter-label">排序</span>
      <div class="filter-pills">
        <button
          v-for="opt in sortOptions"
          :key="opt.value"
          class="filter-pill"
          :class="{ active: searchStore.sortOrder === opt.value }"
          @click="searchStore.setSortOrder(opt.value)"
        >{{ opt.label }}</button>
      </div>
    </div>
    <div class="filter-group">
      <span class="filter-label">时长</span>
      <div class="filter-pills">
        <button
          v-for="opt in durationOptions"
          :key="opt.value"
          class="filter-pill"
          :class="{ active: searchStore.durationFilter === opt.value }"
          @click="searchStore.setDurationFilter(opt.value)"
        >{{ opt.label }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.search-filter-bar {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 4px 0 8px;
  flex-shrink: 0;
  overflow-x: auto;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.filter-label {
  font-size: 12px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.filter-pills {
  display: flex;
  gap: 6px;
}

.filter-pill {
  padding: 2px 12px;
  border-radius: 14px;
  font-size: 12px;
  color: var(--text-secondary);
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.filter-pill:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.filter-pill.active {
  background: var(--accent-color);
  border-color: var(--accent-color);
  color: #fff;
}
</style>
