import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Song } from "@/types";

export const useFavoritesStore = defineStore(
  "favorites",
  () => {
    const favorites = ref<Song[]>([]);

    const favoriteIds = computed(
      () => new Set(favorites.value.map((s) => s.bvid))
    );

    function isFavorite(bvid: string): boolean {
      return favoriteIds.value.has(bvid);
    }

    function toggle(song: Song) {
      const idx = favorites.value.findIndex((s) => s.bvid === song.bvid);
      if (idx >= 0) {
        favorites.value.splice(idx, 1);
      } else {
        favorites.value.push({ ...song });
      }
    }

    function add(song: Song) {
      if (!isFavorite(song.bvid)) {
        favorites.value.push({ ...song });
      }
    }

    function remove(bvid: string) {
      const idx = favorites.value.findIndex((s) => s.bvid === bvid);
      if (idx >= 0) {
        favorites.value.splice(idx, 1);
      }
    }

    function moveFavorite(fromIndex: number, toIndex: number) {
      if (fromIndex === toIndex) return;
      const [item] = favorites.value.splice(fromIndex, 1);
      favorites.value.splice(toIndex, 0, item);
    }

    function exportData(): string {
      return JSON.stringify(favorites.value, null, 2);
    }

    function importData(json: string) {
      try {
        const songs = JSON.parse(json) as Song[];
        if (Array.isArray(songs)) {
          for (const song of songs) {
            if (song.bvid && song.title) {
              add(song);
            }
          }
        }
      } catch {
        throw new Error("Invalid JSON data");
      }
    }

    function clearAll() {
      favorites.value = [];
    }

    return {
      favorites,
      favoriteIds,
      isFavorite,
      toggle,
      add,
      remove,
      moveFavorite,
      exportData,
      importData,
      clearAll,
    };
  },
  {
    persist: true,
  }
);
