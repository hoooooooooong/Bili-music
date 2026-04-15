import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Song, SearchResponse, SearchOrder, DurationFilter } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { parseDuration } from "@/utils/formatters";

export const useSearchStore = defineStore("search", () => {
  const keyword = ref("");
  const results = ref<Song[]>([]);
  const page = ref(1);
  const total = ref(0);
  const pageSize = ref(10);
  const loading = ref(false);
  const hasMore = ref(false);
  const error = ref("");
  const sortOrder = ref<SearchOrder>("totalrank");
  const durationFilter = ref<DurationFilter>("all");

  function matchesDurationFilter(song: Song): boolean {
    if (durationFilter.value === "all") return true;
    const secs = parseDuration(song.duration);
    switch (durationFilter.value) {
      case "short": return secs > 0 && secs < 180;
      case "medium": return secs >= 180 && secs <= 300;
      case "long": return secs > 300 && secs <= 600;
      case "very-long": return secs > 600;
      default: return true;
    }
  }

  const filteredResults = computed(() => {
    if (durationFilter.value === "all") return results.value;
    return results.value.filter(matchesDurationFilter);
  });

  async function search(kw: string, p?: number) {
    if (!kw.trim()) return;

    keyword.value = kw;
    loading.value = true;
    error.value = "";
    const searchPage = p ?? page.value;

    try {
      const resp = await invoke<SearchResponse>("search_bilibili", {
        keyword: kw,
        page: searchPage,
        order: sortOrder.value,
      });
      results.value = resp.results;
      page.value = resp.page;
      total.value = resp.total;
      pageSize.value = resp.pageSize;
      hasMore.value = results.value.length < total.value;
    } catch (e: any) {
      error.value =
        typeof e === "string" ? e : e.message || "搜索失败";
      results.value = [];
    } finally {
      loading.value = false;
    }
  }

  async function loadMore() {
    if (loading.value || !hasMore.value) return;
    const nextPage = page.value + 1;
    loading.value = true;

    try {
      const resp = await invoke<SearchResponse>("search_bilibili", {
        keyword: keyword.value,
        page: nextPage,
        order: sortOrder.value,
      });
      results.value.push(...resp.results);
      page.value = resp.page;
      hasMore.value = results.value.length < total.value;
    } catch {
      // silently fail
    } finally {
      loading.value = false;
    }
  }

  function setSortOrder(order: SearchOrder) {
    if (order === sortOrder.value) return;
    sortOrder.value = order;
    if (keyword.value) {
      search(keyword.value, 1);
    }
  }

  function setDurationFilter(filter: DurationFilter) {
    durationFilter.value = filter;
  }

  function clear() {
    keyword.value = "";
    results.value = [];
    page.value = 1;
    total.value = 0;
    error.value = "";
    sortOrder.value = "totalrank";
    durationFilter.value = "all";
  }

  return {
    keyword,
    results,
    page,
    total,
    pageSize,
    loading,
    hasMore,
    error,
    sortOrder,
    durationFilter,
    filteredResults,
    search,
    loadMore,
    setSortOrder,
    setDurationFilter,
    clear,
  };
});
