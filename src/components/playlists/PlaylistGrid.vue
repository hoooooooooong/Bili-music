<script setup lang="ts">
import { NIcon } from "naive-ui";
import { useMessage } from "naive-ui";
import { HeartOutline, AddOutline, MusicalNotesOutline, TimeOutline, CloudUploadOutline, CloudDownloadOutline } from "@vicons/ionicons5";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { usePlaylistStore } from "@/stores/playlists";
import { useFavoritesStore } from "@/stores/favorites";

const emit = defineEmits<{
  selectFavorites: [];
  selectPlaylist: [id: string];
  selectSmartPlaylist: [id: string];
  create: [];
  import: [];
}>();

const playlistStore = usePlaylistStore();
const favoritesStore = useFavoritesStore();
const message = useMessage();

const smartPlaylists = playlistStore.smartPlaylists;

async function exportAll() {
  try {
    if (playlistStore.playlists.length === 0) {
      message.warning("没有歌单可以导出");
      return;
    }
    const json = playlistStore.exportPlaylists();
    const filePath = await save({
      defaultPath: "playlists.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (filePath) {
      await writeFile(filePath, new TextEncoder().encode(json));
      message.success(`已导出 ${playlistStore.playlists.length} 个歌单`);
    }
  } catch (e: any) {
    message.error(e.message || "导出失败");
  }
}
</script>

<template>
  <div class="playlist-grid">
    <button class="playlist-card favorites-card" @click="emit('selectFavorites')">
      <div class="card-covers">
        <template v-if="favoritesStore.favorites.length > 0">
          <img
            v-for="song in favoritesStore.favorites.slice(0, 4)"
            :key="song.bvid"
            :src="song.coverUrl"
            class="cover-thumb"
            loading="lazy"
          />
        </template>
        <div v-else class="covers-placeholder">
          <NIcon size="32" color="var(--text-tertiary)"><HeartOutline /></NIcon>
        </div>
      </div>
      <div class="card-meta">
        <span class="card-name">收藏</span>
        <span class="card-count">{{ favoritesStore.favorites.length }} 首</span>
      </div>
    </button>

    <button
      v-for="sp in smartPlaylists"
      :key="sp.id"
      class="playlist-card smart-card"
      :class="{ 'smart-most': sp.id === '__smart_most_played__', 'smart-recent': sp.id === '__smart_recently_played__' }"
      @click="emit('selectSmartPlaylist', sp.id)"
    >
      <div class="card-covers smart-cover">
        <div class="covers-placeholder">
          <NIcon size="32" color="rgba(255,255,255,0.85)">
            <MusicalNotesOutline v-if="sp.id === '__smart_most_played__'" />
            <TimeOutline v-else />
          </NIcon>
        </div>
      </div>
      <div class="card-meta">
        <span class="card-name">{{ sp.name }}</span>
        <span class="card-count">{{ sp.songs.length }} 首</span>
      </div>
    </button>

    <button
      v-for="p in playlistStore.playlists"
      :key="p.id"
      class="playlist-card"
      @click="emit('selectPlaylist', p.id)"
    >
      <div class="card-covers">
        <template v-if="p.coverUrl">
          <img :src="p.coverUrl" class="cover-thumb single-cover" loading="lazy" />
        </template>
        <template v-else-if="p.songs.length > 0">
          <img
            v-for="song in p.songs.slice(0, 4)"
            :key="song.bvid"
            :src="song.coverUrl"
            class="cover-thumb"
            loading="lazy"
          />
        </template>
        <div v-else class="covers-placeholder">
          <span class="placeholder-text">{{ p.name.charAt(0) }}</span>
        </div>
      </div>
      <div class="card-meta">
        <span class="card-name">{{ p.name }}</span>
        <span class="card-count">{{ p.songs.length }} 首</span>
      </div>
    </button>

    <button class="playlist-card create-card" @click="emit('create')">
      <div class="card-covers">
        <div class="covers-placeholder">
          <NIcon size="32" color="var(--text-tertiary)"><AddOutline /></NIcon>
        </div>
      </div>
      <div class="card-meta">
        <span class="card-name">新建歌单</span>
      </div>
    </button>

    <button class="playlist-card import-card" @click="exportAll">
      <div class="card-covers">
        <div class="covers-placeholder">
          <NIcon size="32" color="var(--text-tertiary)"><CloudDownloadOutline /></NIcon>
        </div>
      </div>
      <div class="card-meta">
        <span class="card-name">导出全部</span>
      </div>
    </button>

    <button class="playlist-card import-card" @click="emit('import')">
      <div class="card-covers">
        <div class="covers-placeholder">
          <NIcon size="32" color="var(--text-tertiary)"><CloudUploadOutline /></NIcon>
        </div>
      </div>
      <div class="card-meta">
        <span class="card-name">导入歌单</span>
      </div>
    </button>
  </div>
</template>

<style scoped>
.playlist-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 12px;
  padding: 16px 20px 20px;
}

.playlist-card {
  display: flex;
  flex-direction: column;
  background: var(--card-bg);
  border-radius: 10px;
  border: 1px solid transparent;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s;
  overflow: hidden;
}

.playlist-card:hover {
  border-color: var(--border-color);
  background: var(--card-hover);
}

.card-covers {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  gap: 2px;
  aspect-ratio: 1;
  background: var(--card-hover);
  overflow: hidden;
}

.single-cover {
  grid-column: 1 / -1;
  grid-row: 1 / -1;
}

.cover-thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.covers-placeholder {
  grid-column: 1 / -1;
  grid-row: 1 / -1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--card-hover);
}

.placeholder-text {
  font-size: 28px;
  font-weight: 600;
  color: var(--text-tertiary);
}

.card-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 10px;
}

.card-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--app-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-count {
  font-size: 11px;
  color: var(--text-tertiary);
}

.create-card .card-name,
.import-card .card-name {
  color: var(--text-secondary);
}

.smart-cover {
  background: transparent;
}

.smart-card .covers-placeholder {
  background: transparent;
}

.smart-most .smart-cover {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.smart-recent .smart-cover {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}
</style>
