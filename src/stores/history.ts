import { defineStore } from "pinia";
import { ref } from "vue";
import type { Song } from "@/types";
import { on } from "@/utils/emitter";

interface HistoryEntry {
  song: Song;
  playedAt: number;
  playCount: number;
}

export const useHistoryStore = defineStore(
  "history",
  () => {
    const history = ref<HistoryEntry[]>([]);

    on("song:played", (song: Song) => addSong(song));
    const searchHistory = ref<string[]>([]);

    function addSong(song: Song) {
      const idx = history.value.findIndex((h) => h.song.bvid === song.bvid);
      if (idx >= 0) {
        history.value[idx].playedAt = Date.now();
        history.value[idx].playCount++;
        const [entry] = history.value.splice(idx, 1);
        history.value.unshift(entry);
      } else {
        history.value.unshift({
          song: { ...song },
          playedAt: Date.now(),
          playCount: 1,
        });
      }
      if (history.value.length > 200) {
        history.value = history.value.slice(0, 200);
      }
    }

    function addSearch(keyword: string) {
      const kw = keyword.trim();
      if (!kw) return;
      const idx = searchHistory.value.indexOf(kw);
      if (idx >= 0) {
        searchHistory.value.splice(idx, 1);
      }
      searchHistory.value.unshift(kw);
      if (searchHistory.value.length > 50) {
        searchHistory.value = searchHistory.value.slice(0, 50);
      }
    }

    function clearHistory() {
      history.value = [];
    }

    function clearSearchHistory() {
      searchHistory.value = [];
    }

    function removeSearch(keyword: string) {
      const idx = searchHistory.value.indexOf(keyword);
      if (idx >= 0) {
        searchHistory.value.splice(idx, 1);
      }
    }

    function removeSong(bvid: string) {
      const idx = history.value.findIndex((h) => h.song.bvid === bvid);
      if (idx >= 0) {
        history.value.splice(idx, 1);
      }
    }

    return {
      history,
      searchHistory,
      addSong,
      addSearch,
      clearHistory,
      clearSearchHistory,
      removeSearch,
      removeSong,
    };
  },
  {
    persist: true,
  }
);
