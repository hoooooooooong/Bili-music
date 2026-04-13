import { defineStore } from "pinia";
import { ref } from "vue";
import type { Song, SearchResponse } from "@/types";
import { invoke } from "@tauri-apps/api/core";

export const useSearchStore = defineStore("search", () => {
  const keyword = ref("");
  const results = ref<Song[]>([]);
  const page = ref(1);
  const total = ref(0);
  const pageSize = ref(10);
  const loading = ref(false);
  const hasMore = ref(false);
  const error = ref("");

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

  function clear() {
    keyword.value = "";
    results.value = [];
    page.value = 1;
    total.value = 0;
    error.value = "";
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
    search,
    loadMore,
    clear,
  };
});
