<script setup lang="ts">
import { ref, computed } from "vue";
import { NIcon, useMessage } from "naive-ui";
import { ArrowBackOutline, DownloadOutline, CloudDownloadOutline } from "@vicons/ionicons5";
import { open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeFile } from "@tauri-apps/plugin-fs";
import PlaylistGrid from "./PlaylistGrid.vue";
import PlaylistSongList from "./PlaylistSongList.vue";
import FavoriteList from "../favorites/FavoriteList.vue";
import ImportFavoritesDialog from "./ImportFavoritesDialog.vue";
import { usePlaylistStore } from "@/stores/playlists";
import { useFavoritesStore } from "@/stores/favorites";
import { useDownloadStore } from "@/stores/download";

const playlistStore = usePlaylistStore();
const favoritesStore = useFavoritesStore();
const downloadStore = useDownloadStore();
const message = useMessage();

type View =
  | { kind: "grid" }
  | { kind: "favorites" }
  | { kind: "playlist"; id: string }
  | { kind: "smart"; type: "mostPlayed" | "recentlyPlayed" };

const currentView = ref<View>({ kind: "grid" });
const showImportFavorites = ref(false);

const selectedPlaylist = computed(() => {
  if (currentView.value.kind !== "playlist") return null;
  return playlistStore.getPlaylist(currentView.value.id) ?? null;
});

const selectedSmartPlaylist = computed(() => {
  const view = currentView.value;
  if (view.kind !== "smart") return null;
  const smartId = view.type === "mostPlayed" ? "__smart_most_played__" : "__smart_recently_played__";
  return playlistStore.smartPlaylists.find((p) => p.id === smartId) ?? null;
});

function selectFavorites() {
  currentView.value = { kind: "favorites" };
}

function selectPlaylist(id: string) {
  currentView.value = { kind: "playlist", id };
}

function selectSmartPlaylist(id: string) {
  if (id === "__smart_most_played__") {
    currentView.value = { kind: "smart", type: "mostPlayed" };
  } else if (id === "__smart_recently_played__") {
    currentView.value = { kind: "smart", type: "recentlyPlayed" };
  }
}

function createPlaylist() {
  const name = prompt("请输入歌单名称");
  if (name?.trim()) {
    const playlist = playlistStore.createPlaylist(name.trim());
    currentView.value = { kind: "playlist", id: playlist.id };
  }
}

function onPlaylistDeleted() {
  currentView.value = { kind: "grid" };
}

async function importPlaylists() {
  try {
    const filePath = await open({
      multiple: false,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!filePath) return;
    const json = await readTextFile(filePath as string);
    const count = playlistStore.importPlaylists(json);
    message.success(`成功导入 ${count} 个歌单`);
  } catch (e: any) {
    message.error(e.message || "导入失败");
  }
}

async function exportFavorites() {
  try {
    if (favoritesStore.favorites.length === 0) {
      message.warning("收藏列表为空");
      return;
    }
    const data = {
      version: 1,
      exportedAt: new Date().toISOString(),
      playlists: [{
        name: "收藏",
        songs: favoritesStore.favorites.map((s) => ({
          bvid: s.bvid, title: s.title, author: s.author,
          duration: s.duration, playCount: s.playCount,
          playCountText: s.playCountText, coverUrl: s.coverUrl,
        })),
      }],
    };
    const json = JSON.stringify(data, null, 2);
    const filePath = await save({
      defaultPath: "收藏.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (filePath) {
      await writeFile(filePath, new TextEncoder().encode(json));
      message.success("收藏已导出");
    }
  } catch (e: any) {
    message.error(e.message || "导出失败");
  }
}

function downloadAllFavorites() {
  if (favoritesStore.favorites.length === 0) return;
  downloadStore.batchDownload(favoritesStore.favorites);
  message.success(`已添加 ${favoritesStore.favorites.length} 首到下载队列`);
}
</script>

<template>
  <div class="playlists-tab">
    <div v-if="currentView.kind === 'favorites'" class="favorites-view">
      <div class="list-header">
        <button class="back-btn" @click="currentView = { kind: 'grid' }">
          <NIcon size="18"><ArrowBackOutline /></NIcon>
        </button>
        <div class="header-info">
          <h3 class="header-name">收藏</h3>
        </div>
        <div class="header-actions">
          <button class="header-btn" @click="exportFavorites" title="导出收藏">
            <NIcon size="18"><CloudDownloadOutline /></NIcon>
          </button>
          <button
            class="header-btn"
            :disabled="favoritesStore.favorites.length === 0"
            @click="downloadAllFavorites"
            title="一键下载"
          >
            <NIcon size="18"><DownloadOutline /></NIcon>
          </button>
        </div>
      </div>
      <FavoriteList />
    </div>

    <PlaylistGrid
      v-else-if="currentView.kind === 'grid'"
      @select-favorites="selectFavorites"
      @select-playlist="selectPlaylist"
      @select-smart-playlist="selectSmartPlaylist"
      @create="createPlaylist"
      @import="importPlaylists"
      @import-favorites="showImportFavorites = true"
    />

    <PlaylistSongList
      v-else-if="selectedPlaylist"
      :key="selectedPlaylist.id"
      :playlist="selectedPlaylist"
      @back="currentView = { kind: 'grid' }"
      @deleted="onPlaylistDeleted"
    />

    <PlaylistSongList
      v-else-if="selectedSmartPlaylist"
      :key="selectedSmartPlaylist.id"
      :playlist="selectedSmartPlaylist"
      :readonly="true"
      @back="currentView = { kind: 'grid' }"
    />

    <ImportFavoritesDialog
      v-if="showImportFavorites"
      @close="showImportFavorites = false"
    />
  </div>
</template>

<style scoped>
.playlists-tab {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.favorites-view {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.list-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.back-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.back-btn:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.header-info {
  flex: 1;
  min-width: 0;
}

.header-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--app-text);
}

.header-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.header-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--text-secondary);
}

.header-btn:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.header-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
