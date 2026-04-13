<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import { NIcon } from "naive-ui";
import { AddOutline } from "@vicons/ionicons5";
import { usePlaylistStore } from "@/stores/playlists";
import type { Song } from "@/types";

const props = defineProps<{ song: Song }>();

const playlistStore = usePlaylistStore();
const open = ref(false);
const containerRef = ref<HTMLElement | null>(null);
const newName = ref("");
const inputRef = ref<HTMLInputElement | null>(null);

function toggle(e: Event) {
  e.stopPropagation();
  open.value = !open.value;
  if (open.value) {
    nextTick(() => inputRef.value?.focus());
  }
}

function addToPlaylist(playlistId: string) {
  playlistStore.addSong(playlistId, props.song);
  open.value = false;
}

function createAndAdd() {
  const name = newName.value.trim();
  if (!name) return;
  const playlist = playlistStore.createPlaylist(name);
  playlistStore.addSong(playlist.id, props.song);
  newName.value = "";
  open.value = false;
}

function onClickOutside(e: MouseEvent) {
  if (containerRef.value && !containerRef.value.contains(e.target as Node)) {
    open.value = false;
  }
}

onMounted(() => document.addEventListener("mousedown", onClickOutside));
onUnmounted(() => document.removeEventListener("mousedown", onClickOutside));
</script>

<template>
  <div class="add-to-playlist" ref="containerRef">
    <button class="action-btn" @click="toggle" title="添加到歌单">
      <NIcon size="18"><AddOutline /></NIcon>
    </button>
    <div v-if="open" class="dropdown">
      <div class="dropdown-header">
        <input
          ref="inputRef"
          v-model="newName"
          placeholder="新建歌单，回车创建"
          class="new-playlist-input"
          @keydown.enter="createAndAdd"
          @click.stop
        />
      </div>
      <div v-if="playlistStore.playlists.length === 0" class="dropdown-empty">
        暂无歌单
      </div>
      <button
        v-for="p in playlistStore.playlists"
        :key="p.id"
        class="dropdown-item"
        @click="addToPlaylist(p.id)"
      >
        <span class="playlist-name">{{ p.name }}</span>
        <span class="playlist-count">{{ p.songs.length }} 首</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.add-to-playlist {
  position: relative;
  display: flex;
  align-items: center;
}

.add-to-playlist .action-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--text-secondary);
  transition: all 0.15s;
  flex-shrink: 0;
}

.add-to-playlist .action-btn:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.dropdown {
  position: absolute;
  right: 0;
  top: 100%;
  z-index: 100;
  min-width: 200px;
  max-width: 280px;
  max-height: 300px;
  overflow-y: auto;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  padding: 4px;
}

.dropdown-header {
  padding: 4px;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 2px;
}

.new-playlist-input {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--app-bg);
  color: var(--app-text);
  font-size: 12px;
  outline: none;
  box-sizing: border-box;
}

.new-playlist-input:focus {
  border-color: var(--accent-color);
}

.new-playlist-input::placeholder {
  color: var(--text-tertiary);
}

.dropdown-empty {
  padding: 16px 12px;
  text-align: center;
  font-size: 12px;
  color: var(--text-tertiary);
}

.dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  text-align: left;
  color: var(--app-text);
  font-size: 13px;
}

.dropdown-item:hover {
  background: var(--card-hover);
}

.playlist-name {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.playlist-count {
  font-size: 11px;
  color: var(--text-tertiary);
  flex-shrink: 0;
  margin-left: 8px;
}
</style>
