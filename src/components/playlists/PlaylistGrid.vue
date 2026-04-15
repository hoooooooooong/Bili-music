<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { NIcon } from "naive-ui";
import { useMessage } from "naive-ui";
import { HeartOutline, AddOutline, MusicalNotesOutline, TimeOutline, CloudUploadOutline, CloudDownloadOutline, LogInOutline } from "@vicons/ionicons5";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { invoke } from "@tauri-apps/api/core";
import { usePlaylistStore } from "@/stores/playlists";
import { useFavoritesStore } from "@/stores/favorites";
import { useSettingsStore } from "@/stores/settings";
import type { FavoritesFolder } from "@/types";

const emit = defineEmits<{
  selectFavorites: [];
  selectPlaylist: [id: string];
  selectSmartPlaylist: [id: string];
  selectBiliFolder: [folder: FavoritesFolder, uid: string];
  create: [];
  import: [];
  importFavorites: [];
}>();

const playlistStore = usePlaylistStore();
const favoritesStore = useFavoritesStore();
const settingsStore = useSettingsStore();
const message = useMessage();

const smartPlaylists = playlistStore.smartPlaylists;

// Bilibili favorites folders (logged-in user)
const biliFolders = ref<FavoritesFolder[]>([]);
const biliUid = ref("");
const biliFoldersLoading = ref(false);

async function fetchBiliFolders() {
  try {
    const loggedIn = await invoke<boolean>("check_login");
    if (!loggedIn) {
      biliFolders.value = [];
      biliUid.value = "";
      return;
    }
    const userInfo = await invoke<{ mid: number }>("get_user_info");
    biliUid.value = String(userInfo.mid);
    biliFoldersLoading.value = true;
    const result = await invoke<{ folders: FavoritesFolder[] }>(
      "fetch_user_favorites_folders",
      { uid: biliUid.value }
    );
    biliFolders.value = result.folders;
  } catch {
    biliFolders.value = [];
  } finally {
    biliFoldersLoading.value = false;
  }
}

onMounted(fetchBiliFolders);

// Refresh Bilibili folders when login state changes
watch(() => settingsStore.sessdata, () => {
  fetchBiliFolders();
});

defineExpose({ refreshBiliFolders: fetchBiliFolders });

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
  <div class="playlist-grid-page">
    <!-- Bilibili favorites folders section -->
    <section v-if="biliFolders.length > 0" class="section">
      <h3 class="section-title">
        <NIcon size="16" color="var(--accent-color)"><LogInOutline /></NIcon>
        B站收藏夹
      </h3>
      <div class="section-grid">
        <button
          v-for="folder in biliFolders"
          :key="'bili-' + folder.id"
          class="playlist-card bili-folder-card"
          @click="emit('selectBiliFolder', folder, biliUid)"
        >
          <div class="card-covers">
            <template v-if="folder.cover">
              <img :src="folder.cover" class="cover-thumb single-cover" loading="lazy" />
            </template>
            <div v-else class="covers-placeholder">
              <span class="placeholder-text">{{ folder.title.charAt(0) }}</span>
            </div>
          </div>
          <div class="card-meta">
            <span class="card-name">{{ folder.title }}</span>
            <span class="card-count">{{ folder.mediaCount }} 首</span>
          </div>
        </button>
      </div>
    </section>

    <!-- Local playlists section -->
    <section class="section">
      <h3 class="section-title">我的歌单</h3>
      <div class="section-grid">
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

        <button class="playlist-card import-card" @click="emit('importFavorites')">
          <div class="card-covers">
            <div class="covers-placeholder">
              <NIcon size="32" color="var(--accent-color)"><LogInOutline /></NIcon>
            </div>
          </div>
          <div class="card-meta">
            <span class="card-name" style="color: var(--accent-color)">导入B站收藏</span>
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
    </section>
  </div>
</template>

<style scoped>
.playlist-grid-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 16px 20px 20px;
  overflow-y: auto;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  font-weight: 600;
  color: var(--app-text);
  margin: 0;
}

.section-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 12px;
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

.bili-folder-card .card-name {
  color: var(--accent-color);
}
</style>
